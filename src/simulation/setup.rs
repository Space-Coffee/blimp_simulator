use bevy::prelude::{default, Camera, Commands, Cuboid, Mesh, Mesh3d, Plane3d, Query, Res, ResMut, Transform, With};
use bevy::asset::{AssetServer, Assets};
use bevy::pbr::{MeshMaterial3d, PointLight, StandardMaterial};
use bevy::color::Color;
use bevy::math::{Vec2, Vec3};
use crate::simulation::physics::RigidBody;

pub fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut camera_transform: Query<&mut Transform, With<Camera>>,
    asset_server: Res<AssetServer>
) {
    // Spawn blimp
    let blimp_mesh = meshes.add(Cuboid::new(1.0, 1.0, 2.5));
    let blimp_material = materials.add(Color::srgb_u8(192, 255, 128));
    cmds.spawn((
        RigidBody{body: physsim::RigidBody {
            pos: nalgebra::Vector3::new(5.0, 0.0, 0.0),
            lin_vel: nalgebra::Vector3::zeros(),
            rot_mat: nalgebra::Matrix3::new(0.0, 0.0, 1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0),
            ang_mom: nalgebra::Vector3::zeros(),
            inv_ine: nalgebra::Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            mass: 1.0
        }},
        Mesh3d(blimp_mesh),
        MeshMaterial3d(blimp_material),
        Transform::from_xyz(5.0, 0.0, 0.0),
    ));

    //Spawn light
    cmds.spawn((
        PointLight {
            color: Color::srgb_u8(255, 255, 255),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    // Floor
    let floor_mesh = meshes.add(Plane3d::new(Vec3::new(0.0,1.0, 0.0), Vec2::new(100.0, 100.0)));
    let floor_material = materials.add(asset_server.load("debug.png"));
    cmds.spawn((
        Mesh3d(floor_mesh),
        MeshMaterial3d(floor_material),
        Transform::from_xyz(0.0,-10.0, 0.0)
    ));

    // Camera
    let mut transform = camera_transform.get_single_mut().expect("Camera not found");
    *transform = Transform::from_xyz(0.0, 2.5, 5.0)
        .looking_at(Vec3::new(3.0, -1.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
}