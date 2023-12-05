use bevy::prelude::*;
use bevy_tweening::{component_animator_system, TweeningPlugin};

mod tween;
pub mod tween_lenses;
mod tween_macros;

pub use tween::*;

pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TweeningPlugin).add_systems(
            Update,
            (
                component_animator_system::<TextureAtlasSprite>,
                component_animator_system::<BackgroundColor>,
            ),
        );
    }
}
