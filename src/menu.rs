use crate::animation::{delay_tween, get_scale_tween};
use crate::loading::TextureAssets;
use crate::reset::Resettable;
use crate::GameState;
use bevy::prelude::*;
use bevy_tweening::{Animator, EaseFunction};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, click_play_button);
    }
}

#[derive(Component)]
struct ButtonColors {
    normal: Color,
    hovered: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::rgb_u8(222, 159, 71),
            hovered: Color::rgb_u8(165, 120, 85),
        }
    }
}

#[derive(Component)]
struct Menu;

pub fn spawn_play_btn(
    children: &mut ChildBuilder,
    tween_delay_ms: u64,
    font: Handle<Font>,
) -> Entity {
    let button_colors = ButtonColors::default();
    children
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(140.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: button_colors.normal.into(),
                transform: Transform::from_scale(Vec2::ZERO.extend(1.)),
                ..Default::default()
            },
            button_colors,
            ChangeState(GameState::Game),
            Animator::new(delay_tween(
                get_scale_tween(None, Vec3::ONE, 350, EaseFunction::BackOut),
                tween_delay_ms,
            )),
            Resettable,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Play",
                TextStyle {
                    font_size: 40.0,
                    color: Color::rgb_u8(61, 51, 51),
                    font,
                    ..default()
                },
            ));
        })
        .id()
}

#[derive(Component)]
struct ChangeState(GameState);

#[derive(Component)]
struct OpenLink(&'static str);

fn click_play_button(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &ButtonColors,
            Option<&ChangeState>,
            Option<&OpenLink>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, button_colors, change_state, open_link) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if let Some(state) = change_state {
                    next_state.set(state.0.clone());
                } else if let Some(link) = open_link {
                    if let Err(error) = webbrowser::open(link.0) {
                        warn!("Failed to open link {error:?}");
                    }
                }
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
    }
}
