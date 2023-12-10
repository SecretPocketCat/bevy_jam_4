use crate::{
    animation::{delay_tween, get_scale_anim, get_scale_tween, DespawnOnTweenCompleted},
    map::{EdgeConnection, WorldMap},
    map_completion::CompletedMap,
    piece::Piece,
    reset::RegisteredSystems,
    GameState,
};
use bevy::{ecs::system::SystemId, prelude::*};
use bevy_tweening::{Animator, EaseFunction};
use hexx::Hex;
use std::time::Duration;

pub struct ScorePlugin;
impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .init_resource::<Level>()
            .add_event::<UpdateScoreEv>()
            .add_systems(OnEnter(GameState::Playing), (setup_ui, restart_timer))
            .add_systems(
                Update,
                (
                    update_score,
                    update_score_text,
                    update_timer.run_if(resource_exists::<GameTimer>()),
                    update_level.run_if(resource_added::<CompletedMap>()),
                )
                    .distributive_run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct TimerText;

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct Score(u32);

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct Level(u32);

#[derive(Debug, Resource, Default, Event)]
pub struct UpdateScoreEv(pub i32);

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct GameTimer(Timer);

fn setup_ui(mut cmd: Commands) {
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
        b.spawn((
            TextBundle::from_section(
                "0",
                TextStyle {
                    font_size: 60.0,
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect::all(Val::Px(20.)),
                ..default()
            }),
            ScoreText,
        ));

        b.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 60.0,
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect::all(Val::Px(20.)),
                ..default()
            }),
            TimerText,
        ));
    });
}

fn update_score_text(score: Res<Score>, mut text_q: Query<&mut Text, With<ScoreText>>) {
    if score.is_changed() {
        if let Ok(mut text) = text_q.get_single_mut() {
            text.sections[0].value = format!("{}", score.0);
        }
    }
}

fn update_score(mut ev_r: EventReader<UpdateScoreEv>, mut score: ResMut<Score>) {
    for ev in ev_r.read() {
        score.0 = score.0.saturating_add_signed(ev.0);

        // todo: tween text & shake
    }
}

fn restart_timer(mut cmd: Commands) {
    cmd.insert_resource(GameTimer(Timer::from_seconds(90., TimerMode::Once)));
}

fn update_timer(
    mut timer: ResMut<GameTimer>,
    time: Res<Time>,
    mut text_q: Query<&mut Text, With<TimerText>>,
) {
    timer.tick(time.delta());

    if let Ok(mut text) = text_q.get_single_mut() {
        text.sections[0].value = format!("{:.0}", timer.remaining().as_secs_f32());
    }

    if timer.just_finished() {
        // stop & transition to score state
    }
}

fn update_level(mut lvl: ResMut<Level>) {
    lvl.0 += 1;
}
