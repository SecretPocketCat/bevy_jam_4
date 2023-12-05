use crate::actions::Actions;
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
    mut player_q: Query<(&ActionState<GameAction>, &mut AgentCoords), With<Player>>,
) {
    for (input, mut coords) in player_q.iter_mut() {
        if let Some(dir) = input.axis_pair(GameAction::MoveDir).and_then(|dir| {
            // just up or down does not make sense with pointy hexes
            if dir.x() != 0. {
                Some(dir)
            } else {
                None
            }
        }) {
            let angle = Vec2::new(-dir.x(), dir.y()).angle_between(Vec2::Y) + 0.5;
            let target_hex = coords.neighbor(hexx::Direction::from_pointy_angle(angle));

            if let Some(entity) = map.entities.get(&target_hex) {
                // Clear highlighted hexes materials
                for entity in highlighted_hexes
                    .movement
                    .iter()
                    .filter_map(|h| map.entities.get(h))
                {
                    commands
                        .entity(*entity)
                        .insert(map.default_material.clone());
                }

                if let Some(selected) = highlighted_hexes.selected {
                    commands
                        .entity(map.entities[&selected])
                        .insert(map.default_material.clone());
                }

                if input.just_pressed(GameAction::Move) {
                    coords.0 = target_hex;

                    // movement hexes
                    highlighted_hexes.movement = coords.ring(1).collect();
                }

                for (vec, mat) in [(&highlighted_hexes.movement, &map.ring_material)] {
                    for h in vec {
                        if let Some(e) = map.entities.get(h) {
                            commands.entity(*e).insert(mat.clone());
                        }
                    }
                }

                // selected hex
                commands
                    .entity(*entity)
                    .insert(map.selected_material.clone());
                highlighted_hexes.selected = Some(target_hex);
            }
        } else if let Some(selected) = highlighted_hexes.selected.take() {
            commands
                .entity(map.entities[&selected])
                .insert(map.default_material.clone());
        }
    }
}
