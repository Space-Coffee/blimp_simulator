use bevy::prelude::{
    App, Component, FixedUpdate, IntoSystemConfigs, Mat3, Plugin, Quat, Query, Res, Time,
    Transform, Vec3, With,
};
use nalgebra;
use std::f64::consts::PI;

use crate::app::{AsyncSyncBridgeRes, SyncAsyncBridgeRes};
use crate::simulation::constants::{
    AIR_MOLAR_MASS, BASE_TEMPERATURE, GAS_CONSTANT, GRAVITATIONAL_ACCELERATION,
};
use crate::simulation::util::pressure_at;
use crate::simulation::BlimpComponent;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (tick_rigid_body, sync_transform).chain());
        app.add_systems(FixedUpdate, apply_gravity);
        app.add_systems(FixedUpdate, apply_buoyancy);
        app.add_systems(FixedUpdate, blimp_drive);
        app.add_systems(FixedUpdate, pass_blimp_sim_data);
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
        let mass = body.body.mass.clone();
        body.body.apply_force_at(
            nalgebra::Vector3::new(0.0, -GRAVITATIONAL_ACCELERATION as f32 * mass, 0.0),
            time.delta_secs(),
            pos,
        );
    }
}

pub fn apply_buoyancy(mut query: Query<&mut RigidBody>, time: Res<Time>) {
    for mut body in query.iter_mut() {
        let pos = body.body.pos.clone();
        let air_density =
            pressure_at(pos.y as f64) * AIR_MOLAR_MASS / GAS_CONSTANT / BASE_TEMPERATURE;
        let size = nalgebra::Vector3::new(0.5, 0.5, 0.7978);
        let volume: f64 = 4.0 / 3.0 * PI * size.x * size.y * size.z;

        let buoyancy = air_density * volume * GRAVITATIONAL_ACCELERATION;

        body.body.apply_force_at(
            nalgebra::Vector3::new(0.0, buoyancy as f32, 0.0),
            time.delta_secs(),
            pos,
        );
    }
}

pub fn blimp_drive(
    mut query: Query<&mut RigidBody, With<BlimpComponent>>,
    time: Res<Time>,
    as_bridge: Res<AsyncSyncBridgeRes>,
) {
    let motors_servos_state = as_bridge.as_ref().0.motors_servos_rx.borrow();
    // We should probably use query.single_mut() instead
    for mut body in query.iter_mut() {
        let pos = body.body.pos.clone();
        for i in 0..4 {
            let motor_pos_rel = nalgebra::Vector3::new(
                if i % 2 == 0 { -2.0 } else { 2.0 },
                0.0,
                if i < 2 { 3.0 } else { -3.0 },
            );
            let pos_with_offset = pos + &(&body.body.rot_mat * &motor_pos_rel);
            let force = body.body.rot_mat
                * nalgebra::Rotation3::from_euler_angles(
                    if i % 2 == 0 { 1.0 } else { -1.0 }
                        * motors_servos_state.1[2 * i]
                        * std::f32::consts::PI
                        / 180.0,
                    0.0,
                    0.0,
                )
                * nalgebra::Rotation3::from_euler_angles(
                    0.0,
                    if i % 2 == 0 { -1.0 } else { 1.0 }
                        * motors_servos_state.1[2 * i + 1]
                        * std::f32::consts::PI
                        / 180.0,
                    0.0,
                )
                * nalgebra::Vector3::new(
                    0.25 * if i % 2 == 0 { -1.0 } else { 1.0 } * motors_servos_state.0[i],
                    0.0,
                    0.0,
                );
            body.body
                .apply_force_at(force * 0.1, time.delta_secs(), pos_with_offset);
        }
    }
}

fn pass_blimp_sim_data(
    query: Query<&RigidBody, With<BlimpComponent>>,
    sa_bridge: Res<SyncAsyncBridgeRes>,
) {
    let body = query.single();
    sa_bridge
        .pos_tx
        .send((body.body.pos.x, body.body.pos.y, body.body.pos.z))
        .unwrap();
    // sa_bridge
    //     .rot_tx
    //     .send((
    //         *body.body.rot_mat.get((0, 0)).unwrap(),
    //         *body.body.rot_mat.get((0, 1)).unwrap(),
    //         *body.body.rot_mat.get((0, 2)).unwrap(),
    //         *body.body.rot_mat.get((1, 0)).unwrap(),
    //         *body.body.rot_mat.get((1, 1)).unwrap(),
    //         *body.body.rot_mat.get((1, 2)).unwrap(),
    //         *body.body.rot_mat.get((2, 0)).unwrap(),
    //         *body.body.rot_mat.get((2, 1)).unwrap(),
    //         *body.body.rot_mat.get((2, 2)).unwrap(),
    //     ))
    //     .unwrap();
    sa_bridge
        .rot_tx
        .send(nalgebra::Rotation3::from_matrix(&body.body.rot_mat))
        .unwrap();
}
