use std::ops::Mul;

use crate::{
    math::{asymptotic_smoothing, asymptotic_smoothing_with_delta_time, inverse_lerp_clamped},
    score::GameTimer,
    GameState,
};
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
                // GameState::Game
                GameState::Tutorial
            } else {
                GameState::Tutorial
            },
        ))
        // .add_collection_to_loading_state::<_, AudioAssets>(GameState::Loading)
        .add_collection_to_loading_state::<_, TextureAssets>(GameState::Loading)
        .add_collection_to_loading_state::<_, FontAssets>(GameState::Loading)
        .add_systems(OnEnter(GameState::Loading), (spawn_cam, spawn_bg))
        .add_systems(Update, (move_bg));
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
pub struct FontAssets {
    #[asset(path = "fonts/OdibeeSans-Regular.ttf")]
    pub main: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
    #[asset(path = "textures/github.png")]
    pub github: Handle<Image>,
    #[asset(texture_atlas(tile_size_x = 86.6, tile_size_y = 100., columns = 4, rows = 4,))]
    #[asset(path = "tiles/tiles.png")]
    pub tiles: Handle<TextureAtlas>,
}

fn spawn_cam(mut cmd: Commands) {
    cmd.spawn((Camera2dBundle::default(), MainCam, Shake::default()));
}

#[derive(Component)]
struct Bg(f32);

fn spawn_bg(mut cmd: Commands) {
    cmd.spawn((SpriteBundle {
        sprite: Sprite {
            color: Color::rgb_u8(253, 209, 121),
            custom_size: Some(Vec2::new(6000.0, 6000.0)),
            ..default()
        },
        ..default()
    },));

    for i in 0..40 {
        let x = -4000. + i as f32 * 200.;
        cmd.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb_u8(254, 225, 184),
                    custom_size: Some(Vec2::new(100.0, 6000.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(x, 0., 0.))
                    .with_rotation(Quat::from_rotation_z(-30f32.to_radians())),
                ..default()
            },
            Bg(x),
        ));
    }
}

fn move_bg(
    time: Res<Time>,
    mut bg_q: Query<(&mut Transform, &Bg)>,
    timer: Option<Res<GameTimer>>,
    mut speed_t: Local<f32>,
) {
    *speed_t = asymptotic_smoothing_with_delta_time(
        *speed_t,
        30. + timer.map_or(0., |t| {
            if t.finished() {
                0.
            } else {
                t.0.percent() * 300.
            }
        }),
        0.09,
        time.delta_seconds(),
    );

    for (mut t, bg) in bg_q.iter_mut() {
        t.translation.x = time.elapsed_seconds().mul(*speed_t).rem_euclid(1000.) + bg.0;
    }
}
