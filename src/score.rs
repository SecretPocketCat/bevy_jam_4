use crate::{
    animation::{delay_tween, get_scale_tween},
    loading::FontAssets,
    map_completion::CompletedMap,
    menu::{ButtonColors, RunSystem},
    piece::Piece,
    reset::{RegisteredSystems, Resettable},
    GameState,
};
use bevy::{prelude::*};
use bevy_trauma_shake::{TraumaCommands};
use bevy_tweening::{Animator, EaseFunction};

use std::{
    ops::{Sub},
    time::Duration,
};

pub struct ScorePlugin;
impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .init_resource::<Level>()
            .add_event::<UpdateScoreEv>()
            .add_event::<UpdateTimerEv>()
            .add_systems(
                OnEnter(GameState::Game),
                (setup_ui, restart_timer, restart_level, restart_score),
            )
            .add_systems(OnEnter(GameState::GameOver), (restart_level,))
            .add_systems(
                Update,
                (
                    update_score,
                    update_score_text,
                    update_pieces_text,
                    (update_timer, tick_timer).run_if(resource_exists::<GameTimer>()),
                    update_level.run_if(resource_added::<CompletedMap>()),
                )
                    .distributive_run_if(in_state(GameState::Game)),
            );
    }
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct TimerText;

#[derive(Component)]
struct PiecesText;

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct Score(pub u32);

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct Level(pub u32);

#[derive(Debug, Resource, Default, Event)]
pub struct UpdateScoreEv(pub i32);

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct GameTimer(pub Timer);

#[derive(Debug, Resource, Default, Event)]
pub struct UpdateTimerEv(pub f32);

fn setup_ui(mut cmd: Commands, fonts: Res<FontAssets>, systems: Res<RegisteredSystems>) {
    cmd.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        ..default()
    })
    .with_children(|b| {
        b.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|b| {
            b.spawn((
                TextBundle::from_section(
                    "SCORE",
                    TextStyle {
                        font_size: 40.0,
                        color: Color::rgb_u8(61, 51, 51),
                        font: fonts.main.clone(),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(40.)),
                    ..default()
                }),
                Resettable,
            ));

            b.spawn((
                TextBundle::from_section(
                    "0",
                    TextStyle {
                        font_size: 60.0,
                        color: Color::rgb_u8(61, 51, 51),
                        font: fonts.main.clone(),
                        ..default()
                    },
                )
                .with_style(Style {
                    // width: Val::Px(60.),
                    margin: UiRect::horizontal(Val::Px(40.)),
                    ..default()
                }),
                ScoreText,
                Resettable,
            ));

            let button_colors = ButtonColors::default();
            b.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(140.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::new(Val::Px(30.), Val::DEFAULT, Val::DEFAULT, Val::Px(30.)),
                        ..Default::default()
                    },
                    background_color: button_colors.normal.into(),
                    transform: Transform::from_scale(Vec2::ZERO.extend(1.)),
                    ..Default::default()
                },
                button_colors,
                RunSystem(systems.skip_board),
                Animator::new(delay_tween(
                    get_scale_tween(None, Vec3::ONE, 350, EaseFunction::BackOut),
                    1000,
                )),
                Resettable,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "SKIP",
                    TextStyle {
                        font_size: 40.0,
                        color: Color::rgb_u8(61, 51, 51),
                        font: fonts.main.clone(),
                        ..default()
                    },
                ));
            });
        });

        b.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|b| {
            b.spawn((
                TextBundle::from_section(
                    "REMAINING PIECES",
                    TextStyle {
                        font_size: 25.0,
                        color: Color::rgb_u8(61, 51, 51),
                        font: fonts.main.clone(),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(40.)),
                    ..default()
                }),
                Resettable,
            ));

            b.spawn((
                TextBundle::from_section(
                    "2",
                    TextStyle {
                        font_size: 60.0,
                        color: Color::rgb_u8(61, 51, 51),
                        font: fonts.main.clone(),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::horizontal(Val::Px(40.)),
                    ..default()
                }),
                Resettable,
                PiecesText,
            ));
        });

        b.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|b| {
            b.spawn((
                TextBundle::from_section(
                    "TIME",
                    TextStyle {
                        font_size: 40.0,
                        color: Color::rgb_u8(61, 51, 51),
                        font: fonts.main.clone(),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(40.)),
                    ..default()
                }),
                Resettable,
            ));

            b.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 60.0,
                        color: Color::rgb_u8(61, 51, 51),
                        font: fonts.main.clone(),
                        ..default()
                    },
                )
                .with_style(Style {
                    width: Val::Px(70.),
                    margin: UiRect::horizontal(Val::Px(40.)),
                    ..default()
                }),
                TimerText,
                Resettable,
            ));
        });
    });
}

fn update_score_text(score: Res<Score>, mut text_q: Query<&mut Text, With<ScoreText>>) {
    if score.is_changed() {
        if let Ok(mut text) = text_q.get_single_mut() {
            text.sections[0].value = format!("{}", score.0);
        }
    }
}

fn update_score(
    mut cmd: Commands,
    mut ev_r: EventReader<UpdateScoreEv>,
    mut score: ResMut<Score>,
    text_q: Query<Entity, With<ScoreText>>,
) {
    for ev in ev_r.read() {
        score.0 = score.0.saturating_add_signed(ev.0);

        if let Ok(e) = text_q.get_single() {
            cmd.entity(e).try_insert(Animator::new(
                get_scale_tween(
                    None,
                    (Vec2::ONE * 1.5).extend(1.),
                    250,
                    EaseFunction::BackOut,
                )
                .then(get_scale_tween(
                    None,
                    Vec3::ONE,
                    200,
                    EaseFunction::QuadraticOut,
                )),
            ));

            if ev.0 < 0 {
                cmd.add_trauma(0.3);
            }
        }
    }
}

fn update_pieces_text(
    mut cmd: Commands,
    pieces_q: Query<(), With<Piece>>,
    mut text_q: Query<(Entity, &mut Text), With<PiecesText>>,
) {
    if let Ok((e, mut text)) = text_q.get_single_mut() {
        let count = if pieces_q.iter().len() == 2 { 1 } else { 2 };
        let txt = format!("{}", count);
        if text.sections[0].value != txt {
            text.sections[0].value = txt;
            cmd.entity(e).try_insert(Animator::new(
                get_scale_tween(
                    None,
                    (Vec2::ONE * 1.5).extend(1.),
                    250,
                    EaseFunction::BackOut,
                )
                .then(get_scale_tween(
                    None,
                    Vec3::ONE,
                    200,
                    EaseFunction::QuadraticOut,
                )),
            ));
        }
    }
}

fn restart_timer(mut cmd: Commands) {
    cmd.insert_resource(GameTimer(Timer::from_seconds(150., TimerMode::Once)));
}

fn restart_level(mut cmd: Commands) {
    cmd.insert_resource(Level::default());
}

fn restart_score(mut cmd: Commands) {
    cmd.insert_resource(Score::default());
}

fn tick_timer(
    mut cmd: Commands,
    mut timer: ResMut<GameTimer>,
    time: Res<Time>,
    mut text_q: Query<&mut Text, With<TimerText>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if timer.finished() {
        return;
    }

    timer.tick(time.delta());

    if let Ok(mut text) = text_q.get_single_mut() {
        text.sections[0].value = format!("{:.0}", timer.remaining().as_secs_f32());
    }

    if timer.just_finished() {
        cmd.add_trauma(0.3);
        next.set(GameState::GameOver);
    }
}

fn update_timer(
    mut cmd: Commands,
    mut ev_r: EventReader<UpdateTimerEv>,
    mut score: ResMut<GameTimer>,
    text_q: Query<Entity, With<TimerText>>,
) {
    for ev in ev_r.read() {
        let elapsed = Duration::from_secs_f32(score.elapsed_secs().sub(ev.0).max(0.1));
        score.0.set_elapsed(elapsed);

        if let Ok(e) = text_q.get_single() {
            cmd.entity(e).try_insert(Animator::new(
                get_scale_tween(
                    None,
                    (Vec2::ONE * 1.5).extend(1.),
                    250,
                    EaseFunction::BackOut,
                )
                .then(get_scale_tween(
                    None,
                    Vec3::ONE,
                    200,
                    EaseFunction::QuadraticOut,
                )),
            ));
        }
    }
}

fn update_level(mut lvl: ResMut<Level>) {
    lvl.0 += 1;
}
