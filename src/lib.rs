#![allow(clippy::type_complexity)]
#![allow(unused_imports)]

mod agent;
mod animation;
mod input;
mod loading;
mod map;
mod menu;
mod player;
mod projectile;

use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;
use agent::AgentPlugin;
use animation::AnimationPlugin;
use bevy::prelude::*;
use input::InputPlugin;
use map::MapPlugin;
use projectile::ProjectilePlugin;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>().add_plugins((
            LoadingPlugin,
            MenuPlugin,
            MapPlugin,
            AgentPlugin,
            PlayerPlugin,
            InputPlugin,
            ProjectilePlugin,
            AnimationPlugin,
        ));
    }
}
