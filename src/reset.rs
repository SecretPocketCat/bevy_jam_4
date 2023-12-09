use bevy::{ecs::system::SystemId, prelude::*, window::PrimaryWindow};
use leafwing_input_manager::prelude::*;

use crate::{input::GameAction, loading::MainCam, map::setup_grid, GameState};

#[derive(Component)]
pub struct Resettable;

#[derive(Resource)]
pub struct RegisteredSystems {
    pub reset: SystemId,
    pub spawn_board: SystemId,
}

pub struct ResetPlugin;
impl Plugin for ResetPlugin {
    fn build(&self, app: &mut App) {
        let systems = RegisteredSystems {
            reset: app.world.register_system(reset_board),
            spawn_board: app.world.register_system(setup_grid),
        };

        app.insert_resource(systems);
    }
}

fn reset_board(
    mut cmd: Commands,
    reset_q: Query<Entity, With<Resettable>>,
    systems: Res<RegisteredSystems>,
) {
    for e in reset_q.iter() {
        cmd.entity(e).despawn_recursive();
    }

    cmd.run_system(systems.spawn_board);
}
