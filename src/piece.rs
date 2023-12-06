use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    map::{WorldMap, HEX_SIZE, HEX_SIZE_INNER, HEX_WIDTH},
    GameState,
};

#[derive(Component)]
struct Piece;

pub struct PiecePlugin;
impl Plugin for PiecePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_piece
                .run_if(in_state(GameState::Playing).and_then(resource_exists::<WorldMap>())),
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

        cmd.spawn(SpatialBundle::from_transform(Transform::from_xyz(
            0., 0., 0.,
        )))
        .insert(Piece)
        .with_children(|b| {
            // todo: position properly
            for x in [0., HEX_WIDTH] {
                b.spawn(MaterialMesh2dBundle {
                    mesh: meshes
                        .add(shape::RegularPolygon::new(HEX_SIZE_INNER, 6).into())
                        .into(),
                    material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
                    transform: Transform::from_xyz(x, 0., 0.),
                    ..default()
                });
            }
        });
    }
}
