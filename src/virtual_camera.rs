use bevy::prelude::*;
use bevy_headless_render;

pub async fn virtual_camera_start() {
    let mut bevy_app = App::new();
    bevy_app
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_headless_render::HeadlessRenderPlugin);
    bevy_app.run();
}
