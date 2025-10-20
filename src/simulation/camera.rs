use bevy::prelude::*;
use bevy::render::camera::CameraPlugin;
use crate::app::AsyncSyncBridgeRes;
use crate::render::camera::{GroundCamera, OnboardCamera};
use crate::simulation::BlimpComponent;


pub fn camera_follow(blimp_transform: Query<&Transform, With<BlimpComponent>>, mut onboard_camera: Query<&mut Transform, (With<OnboardCamera>, Without<BlimpComponent>)>) {
    let desired_transform = blimp_transform.single();
    let mut camera_transform = onboard_camera.get_single_mut().expect("Camera not found");
    camera_transform.translation = desired_transform.translation + desired_transform.rotation * Vec3::new(0.0, -1.0, -0.7);
    camera_transform.rotation = desired_transform.rotation;
}

pub fn camera_switch(mut ground_camera: Query<&mut Camera, With<GroundCamera>>, mut onboard_camera: Query<&mut Camera, (With<OnboardCamera>, Without<GroundCamera>)>, mut as_bridge: ResMut<AsyncSyncBridgeRes>) {
    if !as_bridge.0.camera_index_rx.has_changed().unwrap() { return }
    let camera_index: u8 = as_bridge.0.camera_index_rx.borrow_and_update().clone();
    match camera_index {
        1 => {
            onboard_camera.single_mut().is_active = true;
            ground_camera.single_mut().is_active = false;
        }
        _ => {
            onboard_camera.single_mut().is_active = false;
            ground_camera.single_mut().is_active = true;
        }
    }
}