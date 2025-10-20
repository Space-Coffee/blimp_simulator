use bevy::asset::{AssetServer, Assets};
use bevy::color::Color;
use bevy::gltf::GltfAssetLabel;
use bevy::math::{Vec2, Vec3};
use bevy::pbr::{DirectionalLight, MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;
use rand::Rng;
use crate::render::camera::GroundCamera;
use crate::simulation::physics::RigidBody;
use crate::simulation::BlimpComponent;

fn spawn_tree(cmds: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>, root: Vec3, size: f32) {
    let trunk_length = size / 2.0;
    let trunk_thickness = size / 10.0;
    let leaves_radius = size / 3.0;
    // Trunk
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::new(trunk_thickness, trunk_length + leaves_radius / 2.0, trunk_thickness))),
        MeshMaterial3d(materials.add(Color::srgb(0.55, 0.27, 0.07))),
        Transform::from_translation(root + Vec3::new(0.0, trunk_length / 2.0, 0.0))
    ));

    // Leaves
    cmds.spawn((
        Mesh3d(meshes.add(Sphere::new(leaves_radius))),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.6, 0.0))),
        Transform::from_translation(root + Vec3::new(0.0, trunk_length + leaves_radius, 0.0)),
    ));

}

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
        Vec2::new(500.0, 500.0),
    ));
    let floor_material = materials.add(asset_server.load("grass.png"));
    cmds.spawn((
        Mesh3d(floor_mesh),
        MeshMaterial3d(floor_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Camera
    let mut transform = camera_transform.get_single_mut().expect("Camera not found");
    *transform = Transform::from_xyz(16.0, 32.0, 64.0)
        .looking_at(Vec3::new(0.0, 8.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
    let mut rng = rand::rng();
    
    for _ in 0..400 {
        let x = rng.random_range(-500.0f32..500.0f32);
        let z = rng.random_range(-500.0f32..500.0f32);
        let size = rng.random_range(5.0f32..25.0f32);

        spawn_tree(&mut cmds, &mut meshes, &mut materials, Vec3::new(x, 0.0, z), size);
    }

}
