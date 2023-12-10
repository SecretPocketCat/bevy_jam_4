use crate::score::{UpdateScoreEv, UpdateTimerEv};
use bevy::{ecs::system::SystemId, prelude::*};
use std::time::Duration;

pub struct EcsPlugin;
impl Plugin for EcsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                run_delayed_systems,
                send_delayed_events::<UpdateScoreEv>,
                send_delayed_events::<UpdateTimerEv>,
            ),
        );
    }
}

#[derive(Component)]
pub struct DelayedSystem {
    pub system_id: SystemId,
    pub delay: Timer,
}

#[derive(Component)]
pub struct DelayedEvent<T: Event> {
    pub delay: Timer,
    pub data: Option<T>,
}

impl<T: Event> DelayedEvent<T> {
    pub fn new_ms(delay_ms: u64, data: T) -> Self {
        Self {
            delay: Timer::new(Duration::from_millis(delay_ms), TimerMode::Once),
            data: Some(data),
        }
    }

    pub fn new_sec(delay_sec: f32, data: T) -> Self {
        Self::new_ms((delay_sec * 1000.0) as u64, data)
    }
}

fn run_delayed_systems(
    mut query: Query<(Entity, &mut DelayedSystem)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (e, mut sys) in query.iter_mut() {
        sys.delay.tick(time.delta());

        if sys.delay.just_finished() {
            commands.run_system(sys.system_id);
            commands.entity(e).despawn();
        }
    }
}

fn send_delayed_events<T: Event>(
    mut query: Query<(Entity, &mut DelayedEvent<T>)>,
    mut event_w: EventWriter<T>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (e, mut ev) in query.iter_mut() {
        ev.delay.tick(time.delta());

        if ev.delay.just_finished() {
            if let Some(ev) = ev.data.take() {
                event_w.send(ev);
            }

            commands.entity(e).despawn();
        }
    }
}
