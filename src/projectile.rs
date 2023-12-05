use std::time::Duration;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle, time::common_conditions::on_timer};
use hexx::Hex;

use crate::{agent::AgentCoords, map::WorldMap, GameState};

#[derive(Component)]
struct ProjectileSpawner(hexx::Direction);

#[derive(Component)]
struct Projectile(hexx::Direction);

pub struct ProjectilePlugin;
impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            setup_spawners.run_if(resource_exists_and_changed::<WorldMap>()),
        )
        .add_systems(
            Update,
            move_projectile.run_if(on_timer(Duration::from_millis(500))),
        )
        .add_systems(
            Update,
            spawn_projectile.run_if(on_timer(Duration::from_millis(1500))),
        );
    }
}

fn setup_spawners(mut cmd: Commands, map: Res<WorldMap>) {
    for (x, y, dir) in [
        (-300., 50., hexx::Direction::BottomRight),
        (-350., -50., hexx::Direction::BottomRight),
        (-250., -150., hexx::Direction::TopRight),
        (-200., -250., hexx::Direction::TopRight),
        (-150., -350., hexx::Direction::TopRight),
        (350., 150., hexx::Direction::TopLeft),
        (300., -150., hexx::Direction::TopLeft),
    ] {
        let pos = map
            .layout
            .hex_to_world_pos(map.layout.world_pos_to_hex(Vec2::new(x, y)))
            .extend(0.);

        cmd.spawn(SpatialBundle::from_transform(Transform::from_translation(
            pos,
        )))
        .insert(ProjectileSpawner(dir));
    }
}

fn spawn_projectile(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    spawner_q: Query<(&ProjectileSpawner, &GlobalTransform)>,
    map: Res<WorldMap>,
) {
    for (spawner, t) in spawner_q.iter() {
        cmd.spawn(MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(15.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::ORANGE_RED)),
            transform: Transform::from_translation(t.translation()),
            ..default()
        })
        .insert((
            AgentCoords(map.layout.world_pos_to_hex(t.translation().truncate())),
            Projectile(spawner.0),
        ));
    }
}

fn move_projectile(mut projectile_q: Query<(&mut AgentCoords, &Projectile)>) {
    for (mut projectile_coords, projectile) in projectile_q.iter_mut() {
        projectile_coords.0 = projectile_coords.neighbor(projectile.0);
    }
}
