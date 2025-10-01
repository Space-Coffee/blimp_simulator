use bevy::prelude::*;
use crate::simulation::setup;

fn setup_windowed_render(
    mut cmds: Commands
) {
    cmds.spawn((
        Camera3d::default(),
        Camera::default(),
        Transform::default()
    ));
    
}

pub fn apply_windowed_config(mut app: &mut App) {
    app.add_plugins(WindowPlugin::default());
    app.add_systems(Startup, setup_windowed_render.before(setup::setup));
}

