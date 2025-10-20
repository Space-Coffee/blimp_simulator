use bevy::asset::{AssetServer, Assets};
use bevy::color::Color;
use bevy::gltf::GltfAssetLabel;
use bevy::math::{Vec2, Vec3};
use bevy::pbr::{DirectionalLight, MeshMaterial3d, StandardMaterial};
use bevy::prelude::{
    default, Camera, Commands, Mesh, Mesh3d, Plane3d, Query, Res, ResMut, SceneRoot, Transform,
    With,
};
use crate::render::camera::GroundCamera;
use crate::simulation::physics::RigidBody;
use crate::simulation::BlimpComponent;

pub fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut camera_transform: Query<&mut Transform, With<GroundCamera>>,
    asset_server: Res<AssetServer>,
) {
    // Spawn blimp
    // let blimp_mesh = meshes.add(Cuboid::new(1.0, 1.0, 2.5));
    // let blimp_material = materials.add(Color::srgb_u8(192, 255, 128));
    let blimp_pos = nalgebra::Vector3::new(0.0, 5.0, 0.0);
    cmds.spawn((
        RigidBody {
            body: physsim::RigidBody {
                pos: blimp_pos,
                lin_vel: nalgebra::Vector3::zeros(),
                rot_mat: nalgebra::Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
                ang_mom: nalgebra::Vector3::zeros(),
                inv_ine: nalgebra::Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
                mass: 1.0,
            },
        },
        // Mesh3d(blimp_mesh),
        // MeshMaterial3d(blimp_material),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("blimp.glb"))),
        Transform::from_xyz(blimp_pos.x, blimp_pos.y, blimp_pos.z),
        BlimpComponent,
    ));

    //Spawn light
    cmds.spawn((
        DirectionalLight {
            color: Color::srgb_u8(255, 255, 255),
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 100.0, 5.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
    ));

    // Floor
    let floor_mesh = meshes.add(Plane3d::new(
        Vec3::new(0.0, 1.0, 0.0),
        Vec2::new(100.0, 100.0),
    ));
    let floor_material = materials.add(asset_server.load("debug.png"));
    cmds.spawn((
        Mesh3d(floor_mesh),
        MeshMaterial3d(floor_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Camera
    let mut transform = camera_transform.get_single_mut().expect("Camera not found");
    *transform = Transform::from_xyz(16.0, 32.0, 64.0)
        .looking_at(Vec3::new(0.0, 8.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
}
