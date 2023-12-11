#![allow(clippy::type_complexity)]
#![allow(unused_imports)]

mod animation;
mod cooldown;
mod debug;
mod ecs;
mod game_over;
mod input;
mod loading;
mod map;
mod map_completion;
mod math;
mod menu;
mod mouse;
mod piece;
mod reset;
mod score;

use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::piece::PiecePlugin;
use animation::AnimationPlugin;
use bevy::prelude::*;
use bevy_trauma_shake::TraumaPlugin;
use cooldown::CooldownPlugin;
use ecs::EcsPlugin;
use game_over::GameOverPlugin;
use input::InputPlugin;
use map::MapPlugin;
use map_completion::MapCompletionPlugin;
use mouse::CursorPlugin;
use reset::ResetPlugin;
use score::ScorePlugin;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    Loading,
    Tutorial,
    Game,
    GameOver,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>().add_plugins((
            LoadingPlugin,
            MenuPlugin,
            MapPlugin,
            PiecePlugin,
            InputPlugin,
            CursorPlugin,
            AnimationPlugin,
            CooldownPlugin,
            ResetPlugin,
            MapCompletionPlugin,
            ScorePlugin,
            EcsPlugin,
            TraumaPlugin,
            GameOverPlugin,
        ));

        if cfg!(debug_assertions) {
            app.add_plugins(debug::DebugPlugin);
        }
    }
}
