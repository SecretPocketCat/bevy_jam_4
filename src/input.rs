use bevy::prelude::*;
use leafwing_input_manager::prelude::*;



#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum GameAction {
    Move,
    MoveDir,
    RotateCw,
    RotateCcw,
}

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<GameAction>::default())
            .init_resource::<ActionState<GameAction>>()
            .insert_resource(
                InputMap::default()
                    .insert(DualAxis::left_stick(), GameAction::MoveDir)
                    .insert(VirtualDPad::dpad(), GameAction::MoveDir)
                    .insert(VirtualDPad::arrow_keys(), GameAction::MoveDir)
                    .insert(VirtualDPad::wasd(), GameAction::MoveDir)
                    .insert(GamepadButtonType::South, GameAction::Move)
                    .insert(KeyCode::Space, GameAction::Move)
                    .insert(KeyCode::Q, GameAction::RotateCcw)
                    .insert(MouseWheelDirection::Down, GameAction::RotateCcw)
                    .insert(KeyCode::E, GameAction::RotateCw)
                    .insert(MouseWheelDirection::Up, GameAction::RotateCw)
                    .build(),
            );
    }
}
