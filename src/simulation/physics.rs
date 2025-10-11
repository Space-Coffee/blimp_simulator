use bevy::prelude::{
    App, Component, FixedUpdate, IntoSystemConfigs, Mat3, Plugin, Quat, Query, Res, Time,
    Transform, Vec3, With,
};
use nalgebra;

use crate::simulation::BlimpComponent;

pub struct PhysicsPlugin {
    pub motors_servos_rx: tokio::sync::watch::Receiver<([f32; 4], [f32; 12])>,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (tick_rigid_body, sync_transform).chain());
        app.add_systems(FixedUpdate, apply_gravity);
        app.add_systems(FixedUpdate, blimp_drive);
    }
}

#[derive(Component)]
pub struct RigidBody {
    pub body: physsim::RigidBody<f32>,
}

pub fn sync_transform(mut query: Query<(&mut Transform, &RigidBody)>) {
    for (mut transform, rb) in query.iter_mut() {
        transform.translation = Vec3::from_array(rb.body.pos.into());
        transform.rotation = Quat::from_mat3(&Mat3::from_cols_array_2d(&rb.body.rot_mat.into()));
    }
}

pub fn tick_rigid_body(mut query: Query<&mut RigidBody>, time: Res<Time>) {
    for mut body in query.iter_mut() {
        body.body.step_sim(time.delta_secs());
    }
}

pub fn apply_gravity(mut query: Query<&mut RigidBody>, time: Res<Time>) {
    for mut body in query.iter_mut() {
        let pos = body.body.pos.clone();
        body.body.apply_force_at(
            nalgebra::Vector3::new(0.0, -0.1, 0.0),
            time.delta_secs(),
            pos,
        );
    }
}
pub fn blimp_drive(mut query: Query<&mut RigidBody, With<BlimpComponent>>, time: Res<Time>) {
    for mut body in query.iter_mut() {
        let pos = body.body.pos.clone();
        let pos_with_offset = pos + &(&body.body.rot_mat * &nalgebra::Vector3::new(2.0, 0.0, 0.0));
        body.body.apply_force_at(
            nalgebra::Vector3::new(0.0, 0.0, 0.1),
            time.delta_secs(),
            pos_with_offset,
        );
    }
}
