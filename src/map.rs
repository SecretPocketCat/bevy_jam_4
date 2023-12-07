use crate::{
    animation::{delay_tween, get_scale_anim, get_scale_tween},
    GameState,
};
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::HashMap,
    window::PrimaryWindow,
};
use bevy_tweening::{Animator, EaseFunction};
use hexx::{shapes, *};
use strum::EnumIter;

pub const MAP_RADIUS: u32 = 3;
pub const HEX_SIZE: f32 = 50.;
pub const HEX_SIZE_INNER_MULT: f32 = 0.95;
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
    pub fn place_piece(&mut self, hex: Hex, piece_hexes: &HashMap<Hex, HexData>) -> Vec<Vec<Hex>> {
        let cleared_lines = Vec::new();

        let placed_hexes: Vec<_> = piece_hexes
            .iter()
            .map(|(key, val)| (hex + *key, val.clone()))
            .collect();

        // place hexes
        for (hex, placed_hex) in placed_hexes.iter() {
            self.entry(*hex).and_modify(|map_hex| {
                map_hex.placed = Some(placed_hex.clone());
            });
        }

        for (placed_hex, placed_hex_data) in placed_hexes.iter() {
            // line can be cleared twice (ingredient & color)
            let clear_count = 0;

            for line in self.lines(*placed_hex) {
                let (color_match_count, ingredient_match_count) =
                    line.iter().fold((0, 0), |mut acc, h| {
                        if let Some(placed_hex) = self[h].placed.as_ref() {
                            acc.0 += (placed_hex.color == placed_hex_data.color) as usize;
                            acc.1 += (placed_hex.ingredient == placed_hex_data.ingredient) as usize;
                        }

                        acc
                    });

                let cleared_col = color_match_count == line.len();
                let cleared_ingredient = ingredient_match_count == line.len();

                if cleared_col {
                    info!("Cleared color line!");
                }

                if cleared_ingredient {
                    info!("Cleared ingredient line!");
                }
            }

            // check lines for ingredient and/or color matches
        }

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

// todo
#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter)]
pub enum Ingredient {
    Honey,
    Ginger,
    Sugar,
    Spices,
    Frosting,
}

// // todo:
// pub enum HexColor {

// }

#[derive(Component, Clone, Debug)]
pub struct HexData {
    pub ingredient: Ingredient,
    pub color: Color,
    pub entity: Entity,
}

fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let layout = HexLayout {
        hex_size: Vec2::splat(HEX_SIZE),
        orientation: HexOrientation::Pointy,
        ..default()
    };

    // materials
    let default_material = materials.add(Color::WHITE.into());

    // mesh
    let mesh = hexagonal_plane(&layout);
    let mesh_handle = meshes.add(mesh);

    let hexes = shapes::hexagon(Hex::ZERO, MAP_RADIUS)
        .map(|hex| {
            let pos = layout.hex_to_world_pos(hex);
            let hex_len = hex.ulength() as u64;
            let entity = commands
                .spawn((
                    ColorMesh2dBundle {
                        transform: Transform::from_xyz(pos.x, pos.y, 0.0),
                        mesh: mesh_handle.clone().into(),
                        material: default_material.clone(),
                        ..default()
                    },
                    Animator::new(delay_tween(
                        get_scale_tween(
                            Some(Vec2::ZERO.extend(1.)),
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
                // .with_children(|b| {
                //     b.spawn(Text2dBundle {
                //         text: Text::from_section(
                //             format!("{},{}", hex.x, hex.y),
                //             TextStyle {
                //                 font_size: 17.0,
                //                 color: Color::BLACK,
                //                 ..default()
                //             },
                //         ),
                //         transform: Transform::from_xyz(0.0, 0.0, 10.0),
                //         ..default()
                //     });
                // })
                .id();
            (hex, MapHex::new(entity))
        })
        .collect();

    commands.insert_resource(WorldLayout(layout));
    commands.insert_resource(WorldMap(hexes));
}

/// Compute a bevy mesh from the layout
fn hexagonal_plane(hex_layout: &HexLayout) -> Mesh {
    let mesh_info = PlaneMeshBuilder::new(hex_layout)
        .facing(Vec3::Z)
        .with_scale(Vec3::splat(HEX_SIZE_INNER_MULT))
        .build();

    Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs)
        .with_indices(Some(Indices::U16(mesh_info.indices)))
}
