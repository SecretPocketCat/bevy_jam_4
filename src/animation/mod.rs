use bevy::prelude::*;
use bevy_tweening::{component_animator_system, TweenCompleted, TweeningPlugin};

mod tween;
pub mod tween_lenses;
mod tween_macros;

pub use tween::*;

#[derive(Component)]
pub struct DespawnOnTweenCompleted;

pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TweeningPlugin).add_systems(
            Update,
            (
                component_animator_system::<TextureAtlasSprite>,
                component_animator_system::<BackgroundColor>,
                despawn_after_tween,
            ),
        );
    }
}

fn despawn_after_tween(
    mut cmd: Commands,
    mut ev_r: EventReader<TweenCompleted>,
    despawn_q: Query<(), With<DespawnOnTweenCompleted>>,
) {
    for ev in ev_r.read() {
        if despawn_q.contains(ev.entity) {
            cmd.entity(ev.entity).despawn_recursive();
        }
    }
}
