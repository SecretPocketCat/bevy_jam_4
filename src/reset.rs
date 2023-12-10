use bevy::{ecs::system::SystemId, prelude::*, window::PrimaryWindow};
use bevy_tweening::{Animator, EaseFunction};
use leafwing_input_manager::prelude::*;

use crate::{
    animation::{get_relative_scale_anim, get_scale_tween, DespawnOnTweenCompleted},
    input::GameAction,
    loading::MainCam,
    map::spawn_grid,
    GameState,
};

#[derive(Component)]
pub struct ResettableGrid;

#[derive(Resource)]
pub struct RegisteredSystems {
    pub reset: SystemId,
    pub spawn_board: SystemId,
}

#[derive(Component)]
pub struct Resettable;

pub struct ResetPlugin;
impl Plugin for ResetPlugin {
    fn build(&self, app: &mut App) {
        let systems = RegisteredSystems {
            reset: app.world.register_system(reset_board),
            spawn_board: app.world.register_system(spawn_grid),
        };

        app.insert_resource(systems);
    }
}

fn reset_board(
    mut cmd: Commands,
    reset_q: Query<Entity, With<ResettableGrid>>,
    systems: Res<RegisteredSystems>,
) {
    for e in reset_q.iter() {
        cmd.entity(e).despawn_recursive();
    }

    cmd.run_system(systems.spawn_board);
}

pub fn tween_reset(
    mut cmd: Commands,
    reset_q: Query<Entity, Or<(With<Resettable>, With<ResettableGrid>)>>,
) {
    for e in reset_q.iter() {
        cmd.entity(e).try_insert((
            get_relative_scale_anim(Vec2::ZERO.extend(1.), 350),
            DespawnOnTweenCompleted,
        ));
    }
}
