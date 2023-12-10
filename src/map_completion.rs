use std::time::Duration;

use crate::{
    animation::{delay_tween, get_scale_anim, get_scale_tween, DespawnOnTweenCompleted},
    ecs::{DelayedEvent, DelayedSystem},
    map::{EdgeConnection, WorldMap},
    piece::Piece,
    reset::RegisteredSystems,
    score::{UpdateScoreEv, UpdateTimerEv},
    GameState,
};
use bevy::{ecs::system::SystemId, prelude::*};
use bevy_tweening::{Animator, EaseFunction};
use hexx::Hex;

pub struct MapCompletionPlugin;
impl Plugin for MapCompletionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            on_map_completed
                .run_if(in_state(GameState::Playing).and_then(resource_added::<CompletedMap>())),
        );
    }
}

#[derive(Debug, Resource)]
pub struct CompletedMap {
    pub routes: Vec<Vec<Hex>>,
    pub dead_ends: Vec<EdgeConnection>,
}

fn on_map_completed(
    mut cmd: Commands,
    map: Res<WorldMap>,
    completed_map: Res<CompletedMap>,
    systems: Res<RegisteredSystems>,
    piece_q: Query<Entity, With<Piece>>,
) {
    //  despawn pieces
    for e in piece_q.iter() {
        cmd.entity(e).insert((
            get_scale_anim(None, Vec3::ZERO, 300, EaseFunction::BackIn),
            DespawnOnTweenCompleted,
        ));
    }

    // add routes score
    cmd.spawn(DelayedEvent::new_ms(
        300,
        UpdateScoreEv(completed_map.routes.len() as i32 * 10),
    ));

    cmd.spawn(DelayedEvent::new_ms(
        300,
        UpdateTimerEv(completed_map.routes.len() as f32 * 20.),
    ));

    let hex_stagger_ms = 80;
    let route_hex_in_ms = 350;
    let route_hex_out_ms = 300;

    for route in completed_map.routes.iter() {
        // todo: raise score

        for (i, hex) in route.iter().enumerate() {
            cmd.entity(map.hexes[hex].placed_hex_e.unwrap())
                .insert(Animator::new(
                    delay_tween(
                        get_scale_tween(
                            None,
                            (Vec2::ONE * 1.4).extend(1.),
                            route_hex_in_ms,
                            EaseFunction::BackOut,
                        ),
                        i as u64 * hex_stagger_ms,
                    )
                    .then(get_scale_tween(
                        None,
                        Vec3::ONE,
                        route_hex_out_ms,
                        EaseFunction::QuadraticOut,
                    )),
                ));
        }
    }

    let longest_route = completed_map
        .routes
        .iter()
        .max_by_key(|r| r.len())
        .map(|r| r.len())
        .unwrap_or(0);

    let deadends_delay =
        longest_route as u64 * hex_stagger_ms + route_hex_in_ms + route_hex_out_ms + 300;

    let mut reset_delay = deadends_delay;

    if !completed_map.dead_ends.is_empty() {
        reset_delay += completed_map.dead_ends.len() as u64 * hex_stagger_ms
            + route_hex_in_ms
            + route_hex_out_ms
            + 300;

        for (i, dead_end) in completed_map.dead_ends.iter().enumerate() {
            for e in [dead_end.first(), dead_end.second()]
                .iter()
                .map(|h| map.hexes.get(h))
                .flatten()
                .flat_map(|h| h.placed_hex_e)
            {
                cmd.entity(e).insert(Animator::new(
                    delay_tween(
                        get_scale_tween(
                            None,
                            (Vec2::ONE * 1.35).extend(1.),
                            route_hex_in_ms,
                            EaseFunction::BackOut,
                        ),
                        deadends_delay + i as u64 * hex_stagger_ms,
                    )
                    .then(get_scale_tween(
                        None,
                        Vec3::ONE,
                        route_hex_out_ms,
                        EaseFunction::QuadraticOut,
                    )),
                ));
            }
        }

        // sub deadends score
        cmd.spawn(DelayedEvent::new_ms(
            deadends_delay + 300,
            UpdateScoreEv(-(completed_map.dead_ends.len() as i32)),
        ));
    }

    cmd.spawn(DelayedSystem {
        system_id: systems.reset,
        delay: Timer::new(Duration::from_millis(reset_delay), TimerMode::Once),
    });
}
