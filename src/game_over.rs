use crate::{
    animation::{delay_tween, get_scale_tween},
    loading::FontAssets,
    menu::spawn_play_btn,
    reset::{tween_reset, Resettable},
    score::Score,
    GameState,
};
use bevy::prelude::*;
use bevy_tweening::{Animator, EaseFunction};

pub struct GameOverPlugin;
impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), setup_ui)
            .add_systems(OnExit(GameState::Game), tween_reset)
            .add_systems(OnExit(GameState::GameOver), tween_reset);
    }
}

fn setup_ui(mut cmd: Commands, score: Res<Score>, fonts: Res<FontAssets>) {
    cmd.spawn((NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    },))
        .with_children(|b| {
            b.spawn((
                TextBundle {
                    text: Text::from_section(
                        "GAME OVER\n\n",
                        TextStyle {
                            font_size: 50.,
                            font: fonts.main.clone(),
                            color: Color::rgb_u8(61, 51, 51),
                            ..default()
                        },
                    )
                    .with_alignment(TextAlignment::Center),
                    style: Style {
                        margin: UiRect::all(Val::Px(20.)),
                        ..default()
                    },
                    transform: Transform::from_scale(Vec2::ZERO.extend(1.)),
                    ..default()
                },
                Animator::new(delay_tween(
                    get_scale_tween(None, Vec3::ONE, 350, EaseFunction::BackOut),
                    350,
                )),
                Resettable,
            ));

            b.spawn((
                TextBundle {
                    text: Text::from_section(
                        format!("SCORE: {}", score.0),
                        TextStyle {
                            font_size: 90.,
                            font: fonts.main.clone(),
                            color: Color::rgb_u8(61, 51, 51),
                            ..default()
                        },
                    )
                    .with_alignment(TextAlignment::Center),
                    style: Style {
                        margin: UiRect::vertical(Val::Vh(5.)),
                        ..default()
                    },
                    transform: Transform::from_scale(Vec2::ZERO.extend(1.)),
                    ..default()
                },
                Animator::new(delay_tween(
                    get_scale_tween(None, Vec3::ONE, 350, EaseFunction::BackOut),
                    800,
                )),
                Resettable,
            ));

            spawn_play_btn(b, 1200, fonts.main.clone());
        });
}
