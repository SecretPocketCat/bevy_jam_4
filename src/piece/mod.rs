use crate::{
    animation::{delay_tween, get_relative_rotation_tween, get_scale_tween, get_translation_tween},
    cooldown::{Cooldown, Rotating},
    input::GameAction,
    loading::TextureAssets,
    map::{WorldLayout, WorldMap, HEX_SIZE},
    map_completion::CompletedMap,
    reset::ResettableGrid,
    GameState,
};
use bevy::{prelude::*, sprite::Mesh2dHandle, utils::HashMap};
use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle};
use bevy_tweening::{Animator, EaseFunction, Tracks};
use hexx::Hex;
use leafwing_input_manager::action_state::ActionState;
use rand::prelude::*;

use self::{
    drag::{
        drag_piece, drag_piece_end, dragged, out_piece, over_piece, HoveredPiece, InitialPosition,
    },
    hex::HexBlueprints,
};

mod drag;
mod hex;

pub use hex::PieceHexData;

pub struct PiecePlugin;
impl Plugin for PiecePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HexBlueprints>()
            .init_resource::<HoveredPiece>()
            .add_plugins(DefaultPickingPlugins)
            .add_systems(
                Update,
                (
                    spawn_pieces,
                    dragged,
                    drag_piece,
                    drag_piece_end.after(spawn_pieces),
                    rotate_piece,
                    over_piece.after(out_piece),
                    out_piece,
                )
                    .distributive_run_if(
                        in_state(GameState::Game)
                            .and_then(resource_exists::<WorldMap>())
                            .and_then(not(resource_exists::<CompletedMap>())),
                    ),
            );
    }
}

#[derive(Component)]
pub struct Piece {
    hexes: HashMap<Hex, PieceHexData>,
    target_hex: Option<Hex>,
}

#[derive(Component)]
pub struct PlacedPiece;

fn spawn_pieces(
    mut cmd: Commands,
    map_layout: Res<WorldLayout>,
    map: Res<WorldMap>,
    blueprints: Res<HexBlueprints>,
    piece_q: Query<&Piece>,
    placed_piece_q: Query<(), With<PlacedPiece>>,
    sprites: Res<TextureAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if piece_q.iter().len() < 1 {
        let mut rng = thread_rng();
        let piece_tween_delay = if placed_piece_q.is_empty() { 950 } else { 200 };
        let piece_x = map_layout
            .hex_to_world_pos(Hex::new(map.map_radius as i32 + 4, 0))
            .x;

        for (piece_i, y) in [-220., 0., 220.].iter().enumerate() {
            let size = blueprints.size_weighted_index.sample(&mut rng) + 1;
            let mut hexes = HashMap::with_capacity(3);

            for size_i in 0..size {
                let mut blueprint =
                    blueprints.hexes[blueprints.weighted_index.sample(&mut rng)].clone();

                // randomize rotation
                let rotation_side = (0..6).choose(&mut rng).unwrap();
                if rotation_side > 0 {
                    blueprint.connected_sides.rotate_left(rotation_side);
                }

                let mut blueprint = Some(&blueprint);
                let mut hex = Hex::ZERO;

                if size_i > 0 {
                    let prev: &PieceHexData = hexes.values().last().unwrap();
                    let mut connected = false;

                    if rng.gen_bool(0.65) {
                        blueprint.take();
                    } else {
                        connected = rng.gen_bool(0.75);
                    }

                    if size_i == 1 {
                        let side = prev.connections.and_then(|connected_sides| {
                            connected_sides
                                .iter()
                                .enumerate()
                                .filter(|(side_index, conn)| {
                                    **conn == connected
                                        && blueprint.map_or(true, |bp| {
                                            bp.connected_sides[get_opposite_side_index(*side_index)]
                                                == connected
                                        })
                                })
                                .map(|(side, _)| side)
                                .choose(&mut rng)
                        });

                        match side {
                            Some(side) => {
                                hex = Hex::new(1, -1).rotate_cw(side as u32);
                            }
                            None => break,
                        };
                    } else {
                        panic!("Size {size_i} is invalid");
                    }
                }

                let pos = map_layout.hex_to_world_pos(hex).extend(0.1);

                let entity = cmd
                    .spawn((
                        SpriteSheetBundle {
                            transform: Transform {
                                translation: pos,
                                rotation: Quat::from_rotation_z(
                                    (rotation_side as f32 * 60.).to_radians(),
                                ),
                                ..default()
                            },
                            sprite: TextureAtlasSprite::new(blueprint.map_or(
                                10, // empty hex index
                                |h| h.atlas_index,
                            )),
                            texture_atlas: sprites.tiles.clone(),
                            ..default()
                        },
                        Mesh2dHandle::from(
                            meshes.add(shape::RegularPolygon::new(HEX_SIZE, 6).into()),
                        ),
                        PickableBundle::default(),
                    ))
                    // .with_children(|b| {
                    //     b.spawn(Text2dBundle {
                    //         text: Text::from_section(
                    //             format!("{},{}", hex.x, hex.y),
                    //             TextStyle {
                    //                 font_size: 30.0,
                    //                 color: Color::WHITE,
                    //                 ..default()
                    //             },
                    //         ),
                    //         transform: Transform::from_xyz(0.0, 0.0, 10.0),
                    //         ..default()
                    //     });
                    // })
                    .id();

                hexes.insert(
                    hex,
                    PieceHexData {
                        entity,
                        side_index: rotation_side as u8,
                        connections: blueprint.map(|bp| bp.connected_sides),
                    },
                );
            }

            let children: Vec<_> = hexes.values().map(|d| d.entity).collect();

            let pos = Vec3::new(piece_x, *y, 1.);
            cmd.spawn(SpatialBundle::from_transform(
                Transform::from_translation(pos).with_scale(Vec2::ZERO.extend(1.)),
            ))
            .try_insert((
                Piece {
                    hexes,
                    target_hex: None,
                },
                InitialPosition(pos),
                Animator::new(delay_tween(
                    get_scale_tween(None, Vec3::ONE, 300, EaseFunction::BackOut),
                    piece_tween_delay + piece_i as u64 * 80,
                )),
                ResettableGrid,
            ))
            .push_children(&children);
        }
    }
}

fn get_side_index(index: i8) -> usize {
    index.wrapping_rem_euclid(6) as usize
}

pub fn get_opposite_side_index(side: usize) -> usize {
    get_side_index((side + 3) as i8)
}

fn rotate_piece(
    mut cmd: Commands,
    mut piece_q: Query<&mut Piece, Without<Cooldown<Rotating>>>,
    hovered: Res<HoveredPiece>,
    map_layout: Res<WorldLayout>,
    input: Res<ActionState<GameAction>>,
) {
    let mut rotate_cw = None;

    if input.just_pressed(GameAction::RotateCw) {
        rotate_cw = Some(true);
    } else if input.just_pressed(GameAction::RotateCcw) {
        rotate_cw = Some(false);
    }

    if let Some(clockwise) = rotate_cw {
        if let Some(hovered) = &hovered.0 {
            if let Ok(mut piece) = piece_q.get_mut(hovered.piece_e) {
                let center_hex = *piece
                    .hexes
                    .iter()
                    .find(|(_, data)| data.entity == hovered.hex_e)
                    .unwrap()
                    .0;

                piece.hexes = piece
                    .hexes
                    .drain()
                    .map(|(hex, mut piece_hex_data)| {
                        let rotated_hex = if clockwise {
                            hex.cw_around(center_hex)
                        } else {
                            hex.ccw_around(center_hex)
                        };

                        piece_hex_data.side_index = get_side_index(
                            piece_hex_data.side_index as i8 + (if clockwise { -1 } else { 1 }),
                        ) as u8;

                        if let Some(connections) = &mut piece_hex_data.connections {
                            if clockwise {
                                connections.rotate_right(1);
                            } else {
                                connections.rotate_left(1);
                            }
                        }

                        cmd.entity(piece_hex_data.entity).try_insert((
                            Animator::new(Tracks::new([
                                get_translation_tween(
                                    None,
                                    map_layout.hex_to_world_pos(rotated_hex).extend(0.),
                                    350,
                                    EaseFunction::BackInOut,
                                ),
                                get_relative_rotation_tween(
                                    Quat::from_rotation_z(
                                        (piece_hex_data.side_index as f32 * 60.).to_radians(),
                                    ),
                                    300,
                                ),
                            ])),
                            Cooldown::<Rotating>::new(300),
                        ));

                        (rotated_hex, piece_hex_data)
                    })
                    .collect();

                piece.target_hex.take();
                cmd.entity(hovered.piece_e)
                    .try_insert(Cooldown::<Rotating>::new(300));
            }
        }
    }
}
