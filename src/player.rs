use crate::agent::AgentCoords;
use crate::input::{get_input_bundle, GameAction};
use crate::loading::TextureAssets;
use crate::map::{HighlightedHexes, WorldMap};
use crate::GameState;
use bevy::prelude::*;
use hexx::Hex;
use leafwing_input_manager::action_state::ActionState;

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(Update, handle_input.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_player(mut commands: Commands, textures: Res<TextureAssets>) {
    commands
        .spawn(SpriteBundle {
            texture: textures.bevy.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(50.)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
            ..Default::default()
        })
        .insert((Player, AgentCoords::default(), get_input_bundle()));
}

fn handle_input(
    mut commands: Commands,
    map: Res<WorldMap>,
    mut highlighted_hexes: Local<HighlightedHexes>,
    mut player_q: Query<(&ActionState<GameAction>, &mut Transform), With<Player>>,
    time: Res<Time>,
) {
    for (input, mut t) in player_q.iter_mut() {
        t.translation += (input
            .axis_pair(GameAction::MoveDir)
            .map_or(Vec2::ZERO, |x| x.xy())
            .normalize_or_zero()
            * 300.
            * time.delta_seconds())
        .extend(0.);
    }
}
