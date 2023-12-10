use crate::GameState;
use bevy::{prelude::*, transform::commands};
use bevy_asset_loader::prelude::*;
use bevy_trauma_shake::Shake;

#[derive(Component)]
pub struct MainCam;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(LoadingState::new(GameState::Loading).continue_to_state(
            if cfg!(debug_assertions) {
                GameState::Game
            } else {
                GameState::Menu
            },
        ))
        // .add_collection_to_loading_state::<_, AudioAssets>(GameState::Loading)
        .add_collection_to_loading_state::<_, TextureAssets>(GameState::Loading)
        .add_systems(OnEnter(GameState::Loading), spawn_cam);
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)

// #[derive(AssetCollection, Resource)]
// pub struct AudioAssets {
//     #[asset(path = "audio/flying.ogg")]
//     pub flying: Handle<AudioSource>,
// }

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
    #[asset(path = "textures/github.png")]
    pub github: Handle<Image>,
    #[asset(texture_atlas(
        tile_size_x = 82.,
        tile_size_y = 92.,
        columns = 7,
        rows = 2,
        padding_x = 29.,
        padding_y = 7.,
    ))]
    #[asset(path = "tiles/tiles.png")]
    pub tiles: Handle<TextureAtlas>,
}

fn spawn_cam(mut cmd: Commands) {
    cmd.spawn((Camera2dBundle::default(), MainCam, Shake::default()));
}
