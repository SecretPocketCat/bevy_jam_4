use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::GameState;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum GameAction {
    Move,
    MoveDir,
}

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<GameAction>::default());
    }
}

pub fn get_input_bundle() -> InputManagerBundle<GameAction> {
    InputManagerBundle::<GameAction> {
        // Stores "which actions are currently activated"
        action_state: ActionState::default(),
        // Describes how to convert from player inputs into those actions
        input_map: InputMap::default()
            // Configure the left stick as a dual-axis
            .insert(DualAxis::left_stick(), GameAction::MoveDir)
            .insert(VirtualDPad::dpad(), GameAction::MoveDir)
            .insert(VirtualDPad::arrow_keys(), GameAction::MoveDir)
            .insert(VirtualDPad::wasd(), GameAction::MoveDir)
            .insert(GamepadButtonType::South, GameAction::Move)
            .insert(KeyCode::Space, GameAction::Move)
            .build(),
    }
}
