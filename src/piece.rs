use std::marker::PhantomData;

use crate::{
    animation::{get_relative_translation_anim, get_translation_anim},
    cooldown::{Cooldown, Rotating},
    input::GameAction,
    map::{HexData, Ingredient, WorldLayout, WorldMap, HEX_SIZE, HEX_SIZE_INNER, HEX_WIDTH},
    math::{asymptotic_smoothing, asymptotic_smoothing_with_delta_time},
    mouse::CursorPosition,
    GameState,
};
use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
    utils::{info, HashMap},
    window::PrimaryWindow,
};
use bevy_mod_picking::prelude::*;
use bevy_tweening::EaseFunction;
use hexx::Hex;
use leafwing_input_manager::prelude::*;
use rand::{distributions::WeightedIndex, prelude::*};
use strum::IntoEnumIterator;

#[derive(Debug, Resource)]
struct PieceBlueprints {
    blueprints: Vec<Vec<Hex>>,
    weighted_index: WeightedIndex<u8>,
    colors: [Color; 5],
}

impl Default for PieceBlueprints {
    fn default() -> Self {
        let blueprints = vec![
            (vec![Hex::ZERO], 2),
            (vec![Hex::ZERO, Hex::X], 4),
            (vec![Hex::ZERO, Hex::X, Hex::Y], 3),
            (vec![Hex::ZERO, Hex::X, Hex::ONE], 2),
            (vec![Hex::ZERO, Hex::X, Hex::new(-1, 1)], 2),
            (vec![Hex::ZERO, Hex::X, Hex::X * 2], 2),
            (vec![Hex::ZERO, Hex::X, Hex::Y, Hex::ONE], 1),
            // (vec![Hex::ZERO, Hex::X, Hex::X * 2, Hex::X * 3], 1),
            (vec![Hex::ZERO, Hex::X, Hex::X * 2, Hex::Y], 1),
            (vec![Hex::ZERO, Hex::X, Hex::X * 2, Hex::ONE], 1),
        ];

        let weights: Vec<_> = blueprints.iter().map(|(_, weight)| *weight).collect();
        let weighted_index = WeightedIndex::new(weights).unwrap();

        Self {
            blueprints: blueprints.into_iter().map(|bp| bp.0).collect(),
            weighted_index,
            colors: [
                Color::CRIMSON,
                Color::ORANGE,
                Color::YELLOW_GREEN,
                Color::BLACK,
                Color::NAVY,
            ],
        }
    }
}

#[derive(Component)]
struct Piece {
    hexes: HashMap<Hex, HexData>,
    target_hex: Option<Hex>,
}

#[derive(Component, Deref, DerefMut)]
struct InitialPosition(Vec3);

pub struct PiecePlugin;
impl Plugin for PiecePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PieceBlueprints>()
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
    map: Res<WorldMap>,
    map_layout: Res<WorldLayout>,
    blueprints: Res<PieceBlueprints>,
    piece_q: Query<&Piece>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if piece_q.iter().len() < 1 {
        let mut rng = thread_rng();
        let blueprint = &blueprints.blueprints[blueprints.weighted_index.sample(&mut rng)];

        let colors: Vec<_> = blueprints.colors.choose_multiple(&mut rng, 3).collect();

        let placed: Vec<_> = blueprint
            .iter()
            .map(|h| (*h, **colors.choose(&mut rng).unwrap()))
            .collect();

        // todo: determine distribution for the size

        // randomize rotation (if size > 1)

        let placed: HashMap<_, _> = placed
            .iter()
            .map(|(hex, color)| {
                let entity = cmd
                    .spawn((
                        MaterialMesh2dBundle {
                            mesh: meshes
                                .add(shape::RegularPolygon::new(HEX_SIZE_INNER, 6).into())
                                .into(),
                            material: materials.add(ColorMaterial::from(*color)),
                            transform: Transform::from_translation(
                                map_layout.hex_to_world_pos(*hex).extend(0.),
                            ),
                            ..default()
                        },
                        PickableBundle::default(),
                    ))
                    .id();

                (
                    *hex,
                    HexData {
                        entity,
                        color: *color,
                    },
                )
            })
            .collect();

        let children: Vec<_> = placed.values().map(|hex_data| hex_data.entity).collect();

        let pos = Vec3::new(0., 400., 1.);
        cmd.spawn(SpatialBundle::from_transform(Transform::from_translation(
            pos,
        )))
        .insert((
            Piece {
                hexes: placed,
                target_hex: None,
            },
            InitialPosition(pos),
        ))
        .push_children(&children);
    }
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
    mut piece_q: Query<(&Transform, &mut InitialPosition, &Piece)>,
    mut map: ResMut<WorldMap>,
    map_layout: Res<WorldLayout>,
) {
    for ev in ev_r.read() {
        if let Ok(parent) = parent_q.get(ev.target) {
            if let Ok((t, mut initial_pos, piece)) = piece_q.get_mut(parent.get()) {
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
                    .map(|(hex, data)| {
                        let rotated_hex = if clockwise {
                            hex.cw_around(center_hex)
                        } else {
                            hex.ccw_around(center_hex)
                        };

                        cmd.entity(data.entity).insert(get_translation_anim(
                            None,
                            map_layout.hex_to_world_pos(rotated_hex).extend(0.),
                            350,
                            EaseFunction::BackInOut,
                        ));

                        (rotated_hex, data)
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
