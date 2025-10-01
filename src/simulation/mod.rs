use bevy::app::{App, FixedUpdate, Plugin, Startup};
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::pbr::{MeshMaterial3d, PointLight, StandardMaterial};
use bevy::prelude::{default, Commands, Component, Cuboid, Mesh, Mesh3d, Query, Res, ResMut, Resource, Time, Transform, With};

#[derive(Component)]
struct BlimpComponent;

#[derive(Resource)]
struct VirtualBlimpData {
    pos: Vec3,
}
fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn blimp
    let blimp_mesh = meshes.add(Cuboid::new(1.0, 1.0, 2.5));
    let blimp_material = materials.add(Color::srgb_u8(192, 255, 128));
    cmds.spawn((
        BlimpComponent,
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
}
fn update_blimp(
    mut blimp: Query<&mut Transform, With<BlimpComponent>>,
    mut virtual_blimp_data: ResMut<VirtualBlimpData>,
    time: Res<Time>,
) {
    let mut blimp = blimp.single_mut();
    let virtual_blimp_data = virtual_blimp_data.as_mut();
    virtual_blimp_data.pos += (Vec3::new(0.0, 0.0, 1.0)
        * ((time.elapsed_secs() * 5.0).sin() * 0.8 + 0.05))
        * time.delta_secs();
    *blimp = Transform::from_translation(virtual_blimp_data.pos);
}

pub struct BlimpSimulationPlugin {}

impl Plugin for BlimpSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(FixedUpdate, update_blimp)
            .insert_resource(VirtualBlimpData {
                pos: Vec3::new(5.0, 0.0, 0.0),
            });
    }
}
