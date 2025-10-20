use bevy::prelude::*;
use crate::render::camera::OnboardCamera;
use crate::simulation::BlimpComponent;


pub fn camera_follow(blimp_transform: Query<&Transform, With<BlimpComponent>>, mut onboard_camera: Query<&mut Transform, (With<OnboardCamera>, Without<BlimpComponent>)>) {
    let desired_transform = blimp_transform.single();
    let mut camera_transform = onboard_camera.get_single_mut().expect("Camera not found");
    camera_transform.translation = desired_transform.translation + desired_transform.rotation * Vec3::new(0.0, -1.0, -0.7);
    camera_transform.rotation = desired_transform.rotation;
}