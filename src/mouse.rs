use bevy::{prelude::*, window::PrimaryWindow};

use crate::loading::MainCam;

#[derive(Debug, Resource, Deref, DerefMut, Default)]
pub struct CursorPosition(pub Vec2);

pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPosition>()
            .add_systems(Update, update_cursor_pos);
    }
}

fn update_cursor_pos(
    mut cursor_pos: ResMut<CursorPosition>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCam>>,
) {
    let (camera, cam_transform) = camera_q.single();
    let window = window_q.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(cam_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        cursor_pos.0 = world_position;
    }
}
