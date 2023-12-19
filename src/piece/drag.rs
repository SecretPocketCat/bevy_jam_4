use crate::{
    animation::{get_scale_anim, get_translation_anim, DespawnOnTweenCompleted},
    cooldown::{Cooldown, Rotating},
    loading::MainCam,
    map::{WorldLayout, WorldMap},
    mouse::CursorPosition,
};
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_tweening::EaseFunction;

use super::{Piece, PlacedPiece};

#[derive(Component, Deref, DerefMut)]
pub(super) struct InitialPosition(pub(super) Vec3);

#[derive(Component)]
pub(super) struct Dragged(Drag);

pub(super) struct HoveredPieceEntities {
    pub(super) piece_e: Entity,
    pub(super) hex_e: Entity,
}

#[derive(Resource, Default, Deref, DerefMut)]
pub(super) struct HoveredPiece(pub(super) Option<HoveredPieceEntities>);

pub(super) fn dragged(mut cmd: Commands, mut ev_r: EventReader<Pointer<Drag>>) {
    for ev in ev_r.read() {
        if let Some(mut e_cmd) = cmd.get_entity(ev.target) {
            e_cmd.try_insert(Dragged(ev.event.clone()));
        }
    }
}

pub(super) fn drag_piece(
    mut cmd: Commands,
    mut ev_r: EventReader<Pointer<Drag>>,
    rotated_q: Query<(Entity, &Dragged), With<Cooldown<Rotating>>>,
    target_q: Query<(&Parent, &Transform), Without<Piece>>,
    mut piece_q: Query<(&mut Transform, &InitialPosition, &mut Piece)>,
    map: Res<WorldMap>,
    map_layout: Res<WorldLayout>,
    cursor_pos: Res<CursorPosition>,
    cam_q: Query<&OrthographicProjection, With<MainCam>>,
) {
    let mut to_process: Vec<_> = ev_r
        .read()
        .map(|ev| (ev.target, ev.event.clone()))
        .collect();
    to_process.extend(rotated_q.iter().map(|(e, dragged)| (e, dragged.0.clone())));

    let projection = cam_q.single();

    for (target, drag) in to_process {
        if let Ok((parent, target_t)) = target_q.get(target) {
            if let Ok((mut piece_t, initial_pos, mut piece)) = piece_q.get_mut(parent.get()) {
                let target_hex =
                    map_layout.world_pos_to_hex(cursor_pos.0 - target_t.translation.truncate());

                if let Some(hex) = piece.target_hex {
                    if target_hex == hex {
                        continue;
                    }
                }

                if piece.hexes.keys().all(|h| {
                    map.hexes
                        .get(&(target_hex + *h))
                        .map_or(false, |map_hex| map_hex.placed_hex_e.is_none())
                }) {
                    piece.target_hex = Some(target_hex);

                    cmd.entity(parent.get()).try_insert(get_translation_anim(
                        None,
                        map_layout
                            .hex_to_world_pos(target_hex)
                            .extend(piece_t.translation.z),
                        120,
                        EaseFunction::QuadraticOut,
                    ));
                } else {
                    piece.target_hex.take();
                    piece_t.translation.x = initial_pos.x + drag.distance.x * projection.scale;
                    piece_t.translation.y = initial_pos.y - drag.distance.y * projection.scale;
                }
            }
        }
    }
}

pub(super) fn drag_piece_end(
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
                    cmd.entity(parent.get())
                        .remove::<Piece>()
                        .try_insert(PlacedPiece);

                    // stop hexes from being pickable
                    if let Ok(children) = children_q.get(parent.get()) {
                        for child in children {
                            cmd.entity(*child).try_insert(Pickable::IGNORE);
                        }
                    }

                    // place hexes
                    map.place_piece(hex, &piece.hexes);

                    if let Some(completed_map) = map.get_completed_routes() {
                        cmd.insert_resource(completed_map);

                        return;
                    }

                    // remove piece to spawn new ones
                    placed_piece = Some(parent.get());
                } else {
                    cmd.entity(parent.get()).try_insert(get_translation_anim(
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

                cmd.entity(e).try_insert((
                    get_scale_anim(None, Vec3::ZERO, 300, EaseFunction::BackIn),
                    DespawnOnTweenCompleted,
                ));
            }
        }
    }
}

pub(super) fn over_piece(
    mut ev_r: EventReader<Pointer<Over>>,
    parent_q: Query<&Parent>,
    mut trans_q: Query<&mut Transform>,
    mut hovered: ResMut<HoveredPiece>,
) {
    for ev in ev_r.read() {
        if let Ok(parent) = parent_q.get(ev.target) {
            hovered.0 = Some(HoveredPieceEntities {
                piece_e: parent.get(),
                hex_e: ev.target,
            });

            if let Ok(mut t) = trans_q.get_mut(parent.get()) {
                t.translation.z = 10.;
            }
        }
    }
}

pub(super) fn out_piece(
    mut ev_r: EventReader<Pointer<Out>>,
    parent_q: Query<&Parent>,
    mut trans_q: Query<&mut Transform>,
    mut hovered: ResMut<HoveredPiece>,
) {
    for ev in ev_r.read() {
        if let Ok(parent) = parent_q.get(ev.target) {
            if let Some(e) = &hovered.0 {
                if e.piece_e == parent.get() {
                    if let Ok(mut t) = trans_q.get_mut(e.piece_e) {
                        t.translation.z = 1.;
                    }

                    hovered.take();
                }
            }
        }
    }
}
