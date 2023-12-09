use crate::{
    animation::{
        delay_tween, get_relative_rotation_tween, get_relative_scale_anim,
        get_relative_translation_anim, get_scale_anim, get_scale_tween, get_translation_anim,
        get_translation_tween, DespawnOnTweenCompleted,
    },
    cooldown::{Cooldown, Rotating},
    input::GameAction,
    loading::TextureAssets,
    map::{HexData, WorldLayout, WorldMap, HEX_SIZE, HEX_SIZE_INNER, HEX_WIDTH},
    math::{asymptotic_smoothing, asymptotic_smoothing_with_delta_time},
    mouse::CursorPosition,
    GameState,
};
use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
    ui::debug,
    utils::{info, HashMap, HashSet},
    window::PrimaryWindow,
};
use bevy_mod_picking::prelude::*;
use bevy_tweening::{Animator, EaseFunction, Tracks};
use hexx::Hex;
use leafwing_input_manager::prelude::*;
use rand::{distributions::WeightedIndex, prelude::*};
use std::{f32::consts::E, marker::PhantomData, ops::Add};
use strum::IntoEnumIterator;

#[derive(Debug, Clone)]
struct RouteHexBlueprint {
    connected_sides: [bool; 6],
    atlas_index: usize,
    weight: u8,
}

#[derive(Debug, Resource)]
struct HexBlueprints {
    hexes: Vec<RouteHexBlueprint>,
    weighted_index: WeightedIndex<u8>,
    size_weighted_index: WeightedIndex<u8>,
}

impl Default for HexBlueprints {
    fn default() -> Self {
        // these go clockwise from the top-right edge (pointy hexes)
        let blueprints: Vec<_> = [
            ([false, true, true, false, false, false], 7),
            ([false, true, false, true, false, false], 7),
            ([false, true, false, false, true, false], 7),
            ([false, true, true, true, false, false], 3),
            ([false, true, false, true, true, false], 3),
            ([false, true, true, false, true, false], 3),
            ([false, true, false, true, false, true], 3),
            ([false, true, true, true, true, false], 1),
            ([false, true, true, false, true, true], 1),
            ([true, true, true, true, true, true], 0),
        ]
        .into_iter()
        .enumerate()
        .map(
            |(atlas_index, (connected_sides, weight))| RouteHexBlueprint {
                connected_sides,
                weight,
                atlas_index,
            },
        )
        .collect();

        let weighted_index = WeightedIndex::new(blueprints.iter().map(|h| h.weight)).unwrap();

        Self {
            hexes: blueprints,
            weighted_index,
            // todo: tweak when triples work properly
            size_weighted_index: WeightedIndex::new([1, 2 /*3*/]).unwrap(),
            // size_weighted_index: WeightedIndex::new([2, 3, 1]).unwrap(),
        }
    }
}

#[derive(Component)]
struct Piece {
    hexes: HashMap<Hex, PieceHexData>,
    target_hex: Option<Hex>,
}

#[derive(Component)]
pub struct PieceHexData {
    entity: Entity,
    side_index: u8,
    pub data: HexData,
}

#[derive(Component, Deref, DerefMut)]
struct InitialPosition(Vec3);

pub struct PiecePlugin;
impl Plugin for PiecePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HexBlueprints>()
            .init_resource::<HoveredPiece>()
            .add_plugins(DefaultPickingPlugins)
            .add_systems(
                Update,
                (
                    spawn_piece,
                    drag_piece,
                    drag_piece_end,
                    rotate_piece,
                    over_piece,
                    out_piece,
                )
                    .distributive_run_if(
                        in_state(GameState::Playing).and_then(resource_exists::<WorldMap>()),
                    ),
            );
    }
}

fn spawn_piece(
    mut cmd: Commands,
    map_layout: Res<WorldLayout>,
    blueprints: Res<HexBlueprints>,
    piece_q: Query<&Piece>,
    sprites: Res<TextureAssets>,
) {
    if piece_q.iter().len() < 1 {
        let mut rng = thread_rng();

        for y in [-250., 0., 250.] {
            let size = blueprints.size_weighted_index.sample(&mut rng) + 1;
            let mut hexes = HashMap::with_capacity(3);

            for i in 0..size {
                let mut blueprint =
                    (&blueprints.hexes[blueprints.weighted_index.sample(&mut rng)]).clone();

                // randomize rotation
                let rotation_side = (0..6).choose(&mut rng).unwrap();

                if rotation_side > 0 {
                    blueprint.connected_sides.rotate_left(rotation_side);
                }

                let mut blueprint = Some(&blueprint);
                let mut side = Some(rotation_side);

                if i > 0 {
                    let prev: &PieceHexData = hexes.values().last().unwrap();
                    let mut connected = false;

                    if rng.gen_bool(0.5) {
                        blueprint.take();
                    } else {
                        connected = rng.gen_bool(0.5);
                    }

                    if i == 1 {
                        side = prev.data.connections().map_or(None, |connected_sides| {
                            connected_sides
                                .iter()
                                .enumerate()
                                .find(|(side, conn)| {
                                    **conn == connected
                                        && blueprint.map_or(true, |bp| {
                                            bp.connected_sides[get_opposite_side_index(*side)]
                                                == connected
                                        })
                                })
                                .map(|(side, _)| side)
                        });
                    } else {
                        panic!("Size {i} is invalid");
                    }
                }

                if side.is_none() {
                    break;
                }

                let hex = if i == 0 {
                    Hex::ZERO
                } else {
                    side.map_or(Hex::ZERO, |side| Hex::new(1, -1).rotate_cw(side as u32))
                };

                let entity = cmd
                    .spawn((
                        SpriteSheetBundle {
                            transform: Transform {
                                translation: map_layout.hex_to_world_pos(hex).extend(0.),
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
                        PickableBundle::default(),
                    ))
                    .id();

                hexes.insert(
                    hex,
                    PieceHexData {
                        entity,
                        side_index: side.unwrap_or(0) as u8,
                        data: blueprint.map_or(HexData::Empty, |bp| HexData::Route {
                            connections: bp.connected_sides.clone(),
                        }),
                    },
                );
            }

            let children: Vec<_> = hexes.values().map(|d| d.entity).collect();

            // todo: raise z to prevent z-fighting
            let pos = Vec3::new(y, 400., 1.);
            cmd.spawn(SpatialBundle::from_transform(
                Transform::from_translation(pos).with_scale(Vec2::ZERO.extend(1.)),
            ))
            .insert((
                Piece {
                    hexes,
                    target_hex: None,
                },
                InitialPosition(pos),
                Animator::new(delay_tween(
                    get_scale_tween(None, Vec3::ONE, 300, EaseFunction::BackOut),
                    150,
                )),
            ))
            .push_children(&children);
        }
    }
}

fn get_side_index(index: i8) -> usize {
    index.wrapping_rem_euclid(6) as usize
}

fn get_opposite_side_index(side: usize) -> usize {
    get_side_index((side + 3) as i8)
}

fn drag_piece(
    mut cmd: Commands,
    mut ev_r: EventReader<Pointer<Drag>>,
    target_q: Query<(&Parent, &Transform), Without<Piece>>,
    mut piece_q: Query<(&mut Transform, &InitialPosition, &mut Piece)>,
    map: Res<WorldMap>,
    map_layout: Res<WorldLayout>,
    cursor_pos: Res<CursorPosition>,
) {
    for ev in ev_r.read() {
        if let Ok((parent, target_t)) = target_q.get(ev.target) {
            if let Ok((mut piece_t, initial_pos, mut piece)) = piece_q.get_mut(parent.get()) {
                let target_hex =
                    map_layout.world_pos_to_hex(cursor_pos.0 - target_t.translation.truncate());

                if let Some(hex) = piece.target_hex {
                    if target_hex == hex {
                        continue;
                    }
                }

                if piece.hexes.keys().all(|h| {
                    map.get(&(target_hex + *h))
                        .map_or(false, |map_hex| map_hex.placed.is_none())
                }) {
                    piece.target_hex = Some(target_hex);

                    cmd.entity(parent.get()).insert(get_translation_anim(
                        None,
                        map_layout
                            .hex_to_world_pos(target_hex)
                            .extend(piece_t.translation.z),
                        120,
                        EaseFunction::QuadraticOut,
                    ));
                } else {
                    piece.target_hex.take();
                    piece_t.translation.x = initial_pos.x + ev.distance.x;
                    piece_t.translation.y = initial_pos.y - ev.distance.y;
                }
            }
        }
    }
}

fn drag_piece_end(
    mut cmd: Commands,
    mut ev_r: EventReader<Pointer<DragEnd>>,
    parent_q: Query<&Parent>,
    children_q: Query<&Children>,
    mut piece_q: Query<(Entity, &Transform, &mut InitialPosition, &Piece)>,
    mut map: ResMut<WorldMap>,
    map_layout: Res<WorldLayout>,
) {
    let mut placed_piece = None;

    for ev in ev_r.read() {
        if let Ok(parent) = parent_q.get(ev.target) {
            if let Ok((_, t, mut initial_pos, piece)) = piece_q.get_mut(parent.get()) {
                if let Some(hex) = piece.target_hex {
                    initial_pos.0 = map_layout.hex_to_world_pos(hex).extend(t.translation.z);

                    // place hexes
                    map.place_piece(hex, &piece.hexes);

                    // stop hexes from being pickable
                    if let Ok(children) = children_q.get(parent.get()) {
                        for child in children.iter() {
                            cmd.entity(*child).insert(Pickable::IGNORE);
                        }
                    }

                    // remove piece to spawn new ones
                    cmd.entity(parent.get()).remove::<Piece>();
                    placed_piece = Some(parent.get());
                } else {
                    cmd.entity(parent.get()).insert(get_translation_anim(
                        None,
                        initial_pos.0,
                        250,
                        EaseFunction::QuadraticOut,
                    ));
                }
            }
        }
    }

    if let Some(placed_e) = placed_piece {
        // only remove last piece
        if piece_q.iter().len() <= 2 {
            for (e, ..) in piece_q.iter() {
                if e == placed_e {
                    continue;
                }

                cmd.entity(e).insert((
                    get_scale_anim(None, Vec3::ZERO, 300, EaseFunction::BackIn),
                    DespawnOnTweenCompleted,
                ));
            }
        }
    }
}

struct HoveredPieceEntities {
    piece_e: Entity,
    hex_e: Entity,
}

#[derive(Resource, Default, Deref, DerefMut)]
struct HoveredPiece(Option<HoveredPieceEntities>);

fn over_piece(
    mut ev_r: EventReader<Pointer<Over>>,
    parent_q: Query<&Parent>,
    mut hovered: ResMut<HoveredPiece>,
) {
    for ev in ev_r.read() {
        if let Ok(parent) = parent_q.get(ev.target) {
            hovered.0 = Some(HoveredPieceEntities {
                piece_e: parent.get(),
                hex_e: ev.target,
            });
        }
    }
}

fn out_piece(
    mut ev_r: EventReader<Pointer<Out>>,
    parent_q: Query<&Parent>,
    mut hovered: ResMut<HoveredPiece>,
) {
    for ev in ev_r.read() {
        if let Ok(parent) = parent_q.get(ev.target) {
            if let Some(e) = &hovered.0 {
                if e.piece_e == parent.get() {
                    hovered.take();
                }
            }
        }
    }
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
        if let Some(e) = &hovered.0 {
            if let Ok(mut piece) = piece_q.get_mut(e.piece_e) {
                let center_hex = *piece
                    .hexes
                    .iter()
                    .find(|(_, data)| data.entity == e.hex_e)
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

                        if let HexData::Route { connections, .. } = &mut piece_hex_data.data {
                            if clockwise {
                                connections.rotate_right(1);
                            } else {
                                connections.rotate_left(1);
                            }
                        }

                        cmd.entity(piece_hex_data.entity)
                            .insert(Animator::new(Tracks::new([
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
                            ])));

                        (rotated_hex, piece_hex_data)
                    })
                    .collect();

                cmd.entity(e.piece_e).insert(Cooldown::<Rotating>::new(300));

                // todo: do this properly
                // this will at least prevent placing the piece, but rotating without movement will mean the piece will stay in the same place
                piece.target_hex.take();
            }
        }
    }
}
