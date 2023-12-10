use crate::{
    ecs::DelayedEvent, input::GameAction, loading::MainCam, reset::RegisteredSystems,
    score::UpdateTimerEv, GameState,
};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_editor_pls::EditorPlugin;
use bevy_trauma_shake::TraumaCommands;
use leafwing_input_manager::prelude::*;

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

        if cfg!(target_arch = "wasm32") {
            app.add_plugins(EditorPlugin::default());
        }
    }
}

fn handle_input(
    mut cmd: Commands,
    input: Res<ActionState<DebugAction>>,
    systems: Res<RegisteredSystems>,
    mut ev_w: EventWriter<UpdateTimerEv>,
) {
    if input.just_pressed(DebugAction::Reset) {
        cmd.run_system(systems.reset);
        cmd.add_trauma(0.7);
        ev_w.send(UpdateTimerEv(-5.));
    }
}
