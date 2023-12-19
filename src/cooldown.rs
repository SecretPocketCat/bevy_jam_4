use bevy::prelude::*;
use std::{marker::PhantomData, time::Duration};

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Cooldown<T: Send + Sync + 'static> {
    timer: Timer,
    _phantom: PhantomData<T>,
}

impl<T: Send + Sync> Cooldown<T> {
    pub fn new(duration_ms: u64) -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(duration_ms), TimerMode::Once),
            _phantom: PhantomData,
        }
    }
}

pub struct Rotating;

pub struct CooldownPlugin;
impl Plugin for CooldownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, process_cooldown::<Rotating>);
    }
}

pub fn process_cooldown<T: Send + Sync>(
    mut cmd: Commands,
    mut cooldown_q: Query<(Entity, &mut Cooldown<T>)>,
    time: Res<Time>,
) {
    for (e, mut cooldown) in &mut cooldown_q {
        cooldown.timer.tick(time.delta());

        if cooldown.timer.just_finished() {
            cmd.entity(e).remove::<Cooldown<T>>();
        }
    }
}
