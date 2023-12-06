use crate::{
    map::{WorldMap, HEX_SIZE, HEX_SIZE_INNER, HEX_WIDTH},
    math::{asymptotic_smoothing, asymptotic_smoothing_with_delta_time},
    GameState,
};
use bevy::{prelude::*, sprite::MaterialMesh2dBundle, utils::info};
use bevy_eventlistener::prelude::*;
use bevy_mod_picking::prelude::*;
use hexx::Hex;

#[derive(Component, Deref, DerefMut)]
struct Piece(Vec<Hex>);

#[derive(Component, Deref, DerefMut)]
struct InitialPosition(Vec3);

pub struct PiecePlugin;
impl Plugin for PiecePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPickingPlugins).add_systems(
            Update,
            (spawn_piece, drag_piece, drag_piece_end).distributive_run_if(
                in_state(GameState::Playing).and_then(resource_exists::<WorldMap>()),
            ),
        );
    }
}

fn spawn_piece(
    mut cmd: Commands,
    map: Res<WorldMap>,
    piece_q: Query<&Piece>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if piece_q.iter().len() < 1 {
        // create a piece
        // todo: determine size (weighted)
        let size = 2;

        // todo: determine distribution for the size

        // randomize rotation (if size > 1)

        let pos = Vec3::new(0., 400., 1.);
        cmd.spawn(SpatialBundle::from_transform(Transform::from_translation(
            pos,
        )))
        .insert((Piece(vec![Hex::ZERO, Hex::X]), InitialPosition(pos)))
        .with_children(|b| {
            // todo: position properly
            for x in [0., HEX_WIDTH] {
                b.spawn((
                    MaterialMesh2dBundle {
                        mesh: meshes
                            .add(shape::RegularPolygon::new(HEX_SIZE_INNER, 6).into())
                            .into(),
                        material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
                        transform: Transform::from_xyz(x, 0., 0.),
                        ..default()
                    },
                    PickableBundle::default(),
                    On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE), // Disable picking
                    On::<Pointer<DragEnd>>::target_insert(Pickable::default()), // Re-enable picking
                ));
            }
        });
    }
}

fn drag_piece(
    mut ev_r: EventReader<Pointer<Drag>>,
    target_q: Query<(&Parent, &Transform), Without<Piece>>,
    mut piece_q: Query<(&mut Transform, &mut InitialPosition, &Piece)>,
    map: Res<WorldMap>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, cam_transform) = camera_q.single();

    for ev in ev_r.read() {
        if let Ok((parent, target_t)) = target_q.get(ev.target) {
            if let Ok((mut piece_t, mut initial_pos, piece)) = piece_q.get_mut(parent.get()) {
                piece_t.translation.x = initial_pos.x + ev.distance.x;
                piece_t.translation.y = initial_pos.y - ev.distance.y;

                // todo: make this a resource
                let cursor_pos = camera
                    .viewport_to_world_2d(cam_transform, ev.pointer_location.position)
                    .unwrap();

                let hex = map
                    .layout
                    .world_pos_to_hex(cursor_pos - target_t.translation.truncate());

                if piece.iter().all(|h| map.entities.contains_key(&(hex + *h))) {
                    let hex_pos = map
                        .layout
                        .hex_to_world_pos(hex)
                        .extend(piece_t.translation.z);

                    // todo: transition this to the target pos
                    piece_t.translation = hex_pos;
                }
            }
        }
    }
}

fn drag_piece_end(
    mut ev_r: EventReader<Pointer<DragEnd>>,
    parent_q: Query<&Parent>,
    mut piece_q: Query<(&Transform, &mut InitialPosition)>,
) {
    for ev in ev_r.read() {
        if let Ok(parent) = parent_q.get(ev.target) {
            if let Ok((t, mut initial_pos)) = piece_q.get_mut(parent.get()) {
                initial_pos.0 = t.translation;
            }
        }
    }
}
