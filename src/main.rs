// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy_game::GamePlugin; // ToDo: Replace bevy_game with your new crate name.

fn main() {
    let mut app = App::new();

    if cfg!(target_arch = "wasm32") {
        app.insert_resource(AssetMetaCheck::Never);
    }

    app.insert_resource(Msaa::Off)
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy game".to_string(), // ToDo
                    // Bind to canvas included in `index.html`
                    canvas: Some("#bevy".to_owned()),
                    // The canvas size is constrained in index.html and build/web/styles.css
                    fit_canvas_to_parent: false,
                    resizable: false,
                    resolution: (1280., 720.).into(),
                    // Tells wasm not to override default event handling, like F5 and Ctrl+R
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }), // .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(GamePlugin)
        .run();
}
