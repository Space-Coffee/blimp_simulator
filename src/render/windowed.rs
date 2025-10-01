use bevy::prelude::*;

fn setup_windowed_render(
    mut cmds: Commands
) {
    cmds.spawn((
        Camera3d::default(),
        Camera::default(),
        Transform::from_xyz(0.0, 2.5, -2.0)
            .looking_at(Vec3::new(5.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
    ));
    
}

pub fn apply_windowed_config(mut app: &mut App) {
    app.add_plugins(WindowPlugin::default());
    app.add_systems(Startup, setup_windowed_render);
}

