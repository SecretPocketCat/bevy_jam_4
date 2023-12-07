use crate::{
    animation::{get_relative_translation_anim, get_translation_anim},
    map::{HexData, Ingredient, PlacedHexes, WorldMap, HEX_SIZE, HEX_SIZE_INNER, HEX_WIDTH},
    math::{asymptotic_smoothing, asymptotic_smoothing_with_delta_time},
    GameState,
};
use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
    utils::{info, HashMap},
};
use bevy_mod_picking::prelude::*;
use bevy_tweening::EaseFunction;
use hexx::Hex;

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
        .insert((
            Piece {
                hexes: [
                    (
                        Hex::ZERO,
                        HexData {
                            ingredient: Ingredient::Ginger,
                            color: Color::ORANGE,
                        },
                    ),
                    (
                        Hex::X,
                        HexData {
                            ingredient: Ingredient::Honey,
                            color: Color::GREEN,
                        },
                    ),
                ]
                .into(),
                target_hex: None,
            },
            InitialPosition(pos),
        ))
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
                ));
            }
        });
    }
}

fn drag_piece(
    mut cmd: Commands,
    mut ev_r: EventReader<Pointer<Drag>>,
    target_q: Query<(&Parent, &Transform), Without<Piece>>,
    mut piece_q: Query<(&mut Transform, &InitialPosition, &mut Piece)>,
    map: Res<WorldMap>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    placed: Res<PlacedHexes>,
) {
    let (camera, cam_transform) = camera_q.single();

    for ev in ev_r.read() {
        if let Ok((parent, target_t)) = target_q.get(ev.target) {
            if let Ok((mut piece_t, initial_pos, mut piece)) = piece_q.get_mut(parent.get()) {
                // todo: make this a resource
                let cursor_pos = camera
                    .viewport_to_world_2d(cam_transform, ev.pointer_location.position)
                    .unwrap();

                let target_hex = map
                    .layout
                    .world_pos_to_hex(cursor_pos - target_t.translation.truncate());

                if let Some(hex) = piece.target_hex {
                    if target_hex == hex {
                        continue;
                    }
                }

                if piece.hexes.keys().all(|h| {
                    map.entities.contains_key(&(target_hex + *h))
                        && !placed.placed.contains_key(&(target_hex + *h))
                }) {
                    piece.target_hex = Some(target_hex);

                    cmd.entity(parent.get()).insert(get_translation_anim(
                        None,
                        map.layout
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
    map: Res<WorldMap>,
    mut placed: ResMut<PlacedHexes>,
) {
    for ev in ev_r.read() {
        if let Ok(parent) = parent_q.get(ev.target) {
            if let Ok((t, mut initial_pos, piece)) = piece_q.get_mut(parent.get()) {
                if let Some(hex) = piece.target_hex {
                    initial_pos.0 = map.layout.hex_to_world_pos(hex).extend(t.translation.z);

                    // place hexes
                    // todo: check for lines
                    placed.placed.extend(
                        piece
                            .hexes
                            .clone()
                            .into_iter()
                            .map(|(key, val)| (hex + key, val)),
                    );

                    // stop hexes from being pickable
                    if let Ok(children) = children_q.get(parent.get()) {
                        for child in children.iter() {
                            cmd.entity(*child).insert(Pickable::IGNORE);
                        }
                    }
                } else {
                    cmd.entity(parent.get()).insert(get_translation_anim(
                        None,
                        initial_pos.0,
                        220,
                        EaseFunction::QuadraticOut,
                    ));
                }
            }
        }
    }
}
