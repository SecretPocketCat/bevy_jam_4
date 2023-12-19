use crate::{
    animation::{delay_tween, get_scale_tween},
    loading::FontAssets,
    menu::spawn_play_btn,
    reset::{tween_reset, Resettable},
    GameState,
};
use bevy::prelude::*;
use bevy_tweening::{Animator, EaseFunction};

pub struct TutorialPlugin;
impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Tutorial), setup_ui)
            .add_systems(OnExit(GameState::Tutorial), tween_reset);
    }
}

fn setup_ui(mut cmd: Commands, fonts: Res<FontAssets>) {
    cmd.spawn((NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(80.0),
            height: Val::Percent(80.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::all(Val::Auto),
            ..default()
        },
        ..default()
    },))
        .with_children(|b| {
            b.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Bee Trails",
                        TextStyle {
                            font_size: 60.,
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
                    500,
                )),
                Resettable,
            ));
           
            b.spawn((
                TextBundle {
                    text: Text::from_section(
                        "The bees were given a lot _wink_. Help them out by connecting their houses in the hive.

                        Use your mouse to place 2 out of 3 pieces (which is your lot to pick from _nudge_).
                        After connectiong all bees you will get 10 points and extra time for each bee, but lose points for unconnected routes. Complete as many lots/hives as possible to score the most points. Do share your score in the comments or on the bevy discord.
                        

                        After you are finished rating the game, feel free to roast me, I'm very much open to constructive feedback no matter how harsh.
                        Because jams happen imagine hearing pleasant upbeat music and punchy SFX and while you are at it picture a nice interactive tutorail too.",
                        TextStyle {
                            font_size: 30.,
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
                    800,
                )),
                Resettable,
            ));

            b.spawn((
                TextBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font_size: 90.,
                            font: fonts.main.clone(),
                            color: Color::rgb_u8(61, 51, 51),
                            ..default()
                        },
                    )
                    .with_alignment(TextAlignment::Center),
                    style: Style {
                        margin: UiRect::top(Val::Px(20.)),
                        ..default()
                    },
                    transform: Transform::from_scale(Vec2::ZERO.extend(1.)),
                    ..default()
                },
                Animator::new(delay_tween(
                    get_scale_tween(None, Vec3::ONE, 350, EaseFunction::BackOut),
                    2000,
                )),
                Resettable,
            ));

            spawn_play_btn(b, 1200, fonts.main.clone());
        });
}
