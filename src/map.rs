use crate::{
    animation::{delay_tween, get_scale_anim, get_scale_tween},
    loading::TextureAssets,
    piece::PieceHexData,
    GameState,
};
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::MaterialMesh2dBundle,
    utils::{HashMap, HashSet},
    window::PrimaryWindow,
};
use bevy_tweening::{Animator, EaseFunction};
use hexx::{shapes, *};
use rand::{thread_rng, Rng};
use strum::EnumIter;

pub const MAP_RADIUS: u32 = 3;
pub const HEX_SIZE: f32 = 46.;
pub const HEX_SIZE_INNER_MULT: f32 = 0.925;
pub const HEX_SIZE_INNER: f32 = HEX_SIZE * HEX_SIZE_INNER_MULT;

// https://www.redblobgames.com/grids/hexagons/#basics
pub const HEX_WIDTH: f32 = HEX_SIZE * 1.732_050_8; // sqrt of 3
pub const HEX_HEIGHT: f32 = HEX_SIZE * 2.;

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_grid);
    }
}

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct WorldLayout(HexLayout);

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct WorldMap(HashMap<Hex, MapHex>);

impl WorldMap {
    pub fn place_piece(
        &mut self,
        hex: Hex,
        piece_hexes: &HashMap<Hex, PieceHexData>,
    ) -> Vec<Vec<Hex>> {
        let cleared_lines = Vec::new();

        let placed_hexes: Vec<_> = piece_hexes
            .iter()
            .map(|(key, val)| (hex + *key, val.data.clone()))
            .collect();

        // place hexes
        for (hex, hex_data) in placed_hexes.iter() {
            self.entry(*hex).and_modify(|map_hex| {
                map_hex.placed = Some(hex_data.clone());
            });
        }

        // for (placed_hex, placed_hex_data) in placed_hexes.iter() {
        //     // line can be cleared twice (ingredient & color)
        //     let clear_count = 0;

        //     for line in self.lines(*placed_hex) {
        //         let color_match_count: usize = line
        //             .iter()
        //             .map(|h| {
        //                 if let Some(placed_hex) = self[h].placed.as_ref() {
        //                     (placed_hex.color == placed_hex_data.color) as usize
        //                 } else {
        //                     0
        //                 }
        //             })
        //             .sum();

        //         let cleared_col = color_match_count == line.len();

        //         if cleared_col {
        //             info!("Cleared color line!");
        //         }
        //     }

        //     // check lines for ingredient and/or color matches
        // }

        cleared_lines
    }

    pub fn lines(&self, hex: Hex) -> [Vec<Hex>; 3] {
        let radius = MAP_RADIUS as i32;
        let max_range = -radius as i32..=radius;
        let hex_z = hex.z();

        let x = max_range
            .clone()
            .map(|y| Hex::new(hex.x, y))
            .filter(|h| self.contains_key(h))
            .collect();
        let y = max_range
            .clone()
            .map(|x| Hex::new(x, -x - hex_z))
            .filter(|h| self.contains_key(h))
            .collect();
        let z = max_range
            .clone()
            .map(|x| Hex::new(x, hex.y))
            .filter(|h| self.contains_key(h))
            .collect();

        [x, y, z]
    }
}

#[derive(Clone, Debug)]
pub struct MapHex {
    pub entity: Entity,
    pub placed: Option<HexData>,
}

impl MapHex {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            placed: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum HexData {
    House, // todo: color/group?
    Route { connections: [bool; 6] },
    Empty, // decorations, lakes etc.
}

impl HexData {
    pub fn connections(&self) -> Option<&[bool; 6]> {
        if let HexData::Route { connections, .. } = &self {
            Some(connections)
        } else {
            None
        }
    }
}

fn setup_grid(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    sprites: Res<TextureAssets>,
) {
    let mut rng = thread_rng();

    let layout = HexLayout {
        hex_size: Vec2::splat(HEX_SIZE),
        orientation: HexOrientation::Pointy,
        ..default()
    };

    // materials
    let border_material = materials.add(Color::DARK_GRAY.into());
    let default_material = materials.add(Color::ANTIQUE_WHITE.into());

    // mesh
    let mesh = hexagonal_plane(&layout);
    let mesh_handle = meshes.add(mesh);

    let mut hexes: HashMap<Hex, MapHex> = shapes::hexagon(Hex::ZERO, MAP_RADIUS)
        .map(|hex| {
            let pos = layout.hex_to_world_pos(hex);
            let hex_len = hex.ulength() as u64;
            let entity = cmd
                .spawn((
                    ColorMesh2dBundle {
                        transform: Transform::from_xyz(pos.x, pos.y, 0.0),
                        mesh: mesh_handle.clone().into(),
                        material: border_material.clone(),
                        ..default()
                    },
                    Animator::new(delay_tween(
                        get_scale_tween(
                            None,
                            Vec3::ONE,
                            350,
                            if hex_len == MAP_RADIUS as u64 {
                                EaseFunction::BackOut
                            } else {
                                EaseFunction::QuadraticOut
                            },
                        ),
                        hex_len * 80,
                    )),
                ))
                .with_children(|b| {
                    b.spawn(MaterialMesh2dBundle {
                        mesh: meshes
                            .add(shape::RegularPolygon::new(HEX_SIZE_INNER, 6).into())
                            .into(),
                        material: default_material.clone(),
                        transform: Transform::from_xyz(0., 0., 0.1),
                        ..default()
                    });
                    // b.spawn(Text2dBundle {
                    //     text: Text::from_section(
                    //         format!("{},{}", hex.x, hex.y),
                    //         TextStyle {
                    //             font_size: 17.0,
                    //             color: Color::BLACK,
                    //             ..default()
                    //         },
                    //     ),
                    //     transform: Transform::from_xyz(0.0, 0.0, 10.0),
                    //     ..default()
                    // });
                })
                .id();
            (hex, MapHex::new(entity))
        })
        .collect();

    // houses
    let count = 3;
    let mut house_hexes = HashSet::with_capacity(count);
    'houses: loop {
        for i in (0..=MAP_RADIUS).rev() {
            for hex in Hex::ZERO.ring(i) {
                if house_hexes.contains(&hex) {
                    continue;
                }

                if rng.gen_bool(0.2) {
                    hexes.get_mut(&hex).unwrap().placed = Some(HexData::House);
                    house_hexes.insert(hex);

                    cmd.spawn(SpriteSheetBundle {
                        transform: Transform {
                            translation: layout.hex_to_world_pos(hex).extend(1.),
                            ..default()
                        },
                        sprite: TextureAtlasSprite::new(11),
                        texture_atlas: sprites.tiles.clone(),
                        ..default()
                    });

                    if house_hexes.len() >= count {
                        break 'houses;
                    }
                }
            }
        }
    }

    cmd.insert_resource(WorldLayout(layout));
    cmd.insert_resource(WorldMap(hexes));
}

/// Compute a bevy mesh from the layout
fn hexagonal_plane(hex_layout: &HexLayout) -> Mesh {
    let mesh_info = PlaneMeshBuilder::new(hex_layout)
        .facing(Vec3::Z)
        .with_scale(Vec3::splat(1.075))
        .build();

    Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs)
        .with_indices(Some(Indices::U16(mesh_info.indices)))
}
