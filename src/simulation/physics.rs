use bevy::prelude::*;
use nalgebra;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (tick_rigid_body, sync_transform).chain());
        app.add_systems(FixedUpdate, apply_gravity);
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

pub fn tick_rigid_body(mut query: Query<&mut RigidBody>) {
    for mut body in query.iter_mut() {
        body.body.step_sim(0.1);
    }
}

pub fn apply_gravity(mut query: Query<&mut RigidBody>) {
    for mut body in query.iter_mut() {
        let position = nalgebra::Vector3::new(body.body.pos.x, body.body.pos.y, body.body.pos.z);
        body.body
            .apply_force_at(nalgebra::Vector3::new(0.0, -0.01, 0.0), 0.1, position);
    }
}

