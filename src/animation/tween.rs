#![allow(dead_code)]

use bevy::prelude::*;
use bevy_tweening::*;
use std::time::Duration;

use super::tween_lenses::*;
use super::tween_macros::*;

pub fn delay_tween<T: 'static>(tween: Tween<T>, delay_ms: u64) -> Sequence<T> {
    if delay_ms > 0 {
        Delay::new(Duration::from_millis(delay_ms)).then(tween)
    } else {
        Sequence::new([tween])
    }
}

relative_tween_fns!(
    translation,
    Transform,
    TransformRelativePositionLens,
    Vec3,
    Vec3
);

relative_tween_fns!(scale, Transform, TransformRelativeScaleLens, Vec3, Vec3);

relative_tween_fns!(
    rotation,
    Transform,
    TransformRelativeRotationLens,
    Quat,
    Quat
);

relative_tween_fns!(text_color, Text, TextRelativeColorLens, Vec<Color>, Color);

relative_tween_fns!(
    spritesheet_color,
    TextureAtlasSprite,
    SpriteSheetRelativeColorLens,
    Color,
    Color
);

relative_tween_fns!(sprite_color, Sprite, SpriteRelativeColorLens, Color, Color);

relative_tween_fns!(
    ui_bg_color,
    BackgroundColor,
    UiBackgroundColorLens,
    Color,
    Color
);
