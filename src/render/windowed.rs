use crate::simulation::setup;
use bevy::prelude::*;
use crate::render::camera::{GroundCamera, OnboardCamera};

fn setup_windowed_render(mut cmds: Commands) {
    cmds.spawn((Camera3d::default(), Camera {is_active: false, ..Default::default()}, Transform::default(), GroundCamera));
    cmds.spawn((Camera3d::default(), Camera {is_active: true, ..Default::default()}, Transform::default(), OnboardCamera));
}

pub fn apply_windowed_config(app: &mut App) {
    app.add_plugins(WindowPlugin::default());
    app.add_systems(Startup, setup_windowed_render.before(setup::setup));
}
