use bevy::{prelude::*, window::PrimaryWindow};
use leafwing_input_manager::prelude::*;

use crate::{input::GameAction, loading::MainCam, reset::RegisteredSystems, GameState};

#[derive(Component)]
pub struct PersistReset;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum DebugAction {
    Reset,
}

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<DebugAction>::default())
            .init_resource::<ActionState<DebugAction>>()
            .insert_resource(
                InputMap::default()
                    .insert(KeyCode::Escape, DebugAction::Reset)
                    .insert(KeyCode::R, DebugAction::Reset)
                    .build(),
            )
            .add_systems(Update, handle_input);
    }
}

fn handle_input(
    mut cmd: Commands,
    input: Res<ActionState<DebugAction>>,
    systems: Res<RegisteredSystems>,
) {
    if input.just_pressed(DebugAction::Reset) {
        cmd.run_system(systems.reset);
    }
}
