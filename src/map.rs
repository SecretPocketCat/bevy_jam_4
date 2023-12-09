use crate::{
    animation::{delay_tween, get_scale_anim, get_scale_tween},
    loading::TextureAssets,
    piece::PieceHexData,
    reset::Resettable,
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
use hexx::{shapes, Direction, *};
use rand::{seq::SliceRandom, thread_rng, Rng};
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

#[derive(Debug, Resource)]
pub struct WorldMap {
    pub hexes: HashMap<Hex, MapHex>,
    houses: HashMap<Hex, Option<Vec<Hex>>>,
}

impl WorldMap {
    pub fn place_piece(&mut self, hex: Hex, piece_hexes: &HashMap<Hex, PieceHexData>) {
        let placed_hexes: Vec<_> = piece_hexes
            .iter()
            .map(|(key, val)| (hex + *key, val.connections.clone()))
            .collect();

        // place hexes
        for (hex, hex_data) in placed_hexes.iter() {
            let connections = hex_data.map(|connected_sides| {
                connected_sides
                    .iter()
                    .enumerate()
                    .filter(|(_, connected)| **connected)
                    .map(|(side, _)| Hex::new(1, -1).rotate_cw(side as u32))
                    .collect()
            });

            self.hexes.entry(*hex).and_modify(|map_hex| {
                map_hex.placed = Some(PlacedHex(connections));
            });
        }
    }

    pub fn check_routes(&mut self) {
        // for house_hex in self.houses.iter().filter(|(_, route)| route.is_none()) {}
    }
}

#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct PlacedHex(Option<HashSet<Hex>>);

#[derive(Clone, Debug)]
pub struct MapHex {
    pub entity: Entity,
    pub placed: Option<PlacedHex>,
}

impl MapHex {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            placed: None,
        }
    }
}

pub fn setup_grid(
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
                    // todo: fix this tween?
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
                    Resettable,
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
    let mut wedge_indices = HashSet::with_capacity(count);

    let direction_group = [
        [
            Direction::Top,
            Direction::BottomLeft,
            Direction::BottomRight,
        ],
        [Direction::Bottom, Direction::TopLeft, Direction::TopRight],
    ]
    .choose(&mut rng)
    .unwrap();

    for dir in direction_group {
        'wedge: loop {
            for (i, hex) in Hex::ZERO
                .corner_wedge(((MAP_RADIUS - 2)..=(MAP_RADIUS + 1)).rev(), *dir)
                .enumerate()
            {
                if house_hexes.contains(&hex) || wedge_indices.contains(&i) {
                    continue;
                }

                if rng.gen_bool(0.25) {
                    // todo: tween
                    let entity = cmd
                        .spawn((
                            SpriteSheetBundle {
                                transform: Transform {
                                    translation: layout.hex_to_world_pos(hex).extend(1.),
                                    ..default()
                                },
                                sprite: TextureAtlasSprite::new(11),
                                texture_atlas: sprites.tiles.clone(),
                                ..default()
                            },
                            Resettable,
                        ))
                        .id();

                    hexes
                        .entry(hex)
                        .and_modify(|d| d.placed = Some(PlacedHex::default()))
                        .or_insert_with(|| MapHex {
                            entity,
                            placed: Some(PlacedHex::default()),
                        });
                    house_hexes.insert(hex);
                    wedge_indices.insert(i);

                    break 'wedge;
                }
            }
        }
    }

    cmd.insert_resource(WorldLayout(layout));
    cmd.insert_resource(WorldMap {
        hexes,
        houses: house_hexes.iter().map(|h| (*h, None)).collect(),
    });
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
