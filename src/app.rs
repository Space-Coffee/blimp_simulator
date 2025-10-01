use bevy::prelude::*;
use bevy::animation::AnimationPlugin;
use bevy::app::App;
use bevy::asset::{AssetPlugin, Assets};
use bevy::color::Color;
use bevy::core::{FrameCountPlugin, TaskPoolPlugin};
use bevy::gilrs::GilrsPlugin;
use bevy::math::Vec3;
use bevy::pbr::{MeshMaterial3d, PointLight, StandardMaterial};
use bevy::picking::DefaultPickingPlugins;
use crate::render::register_rendering_systems;

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
fn register_simulation_systems(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(FixedUpdate, update_blimp)
        .insert_resource(VirtualBlimpData {
            pos: Vec3::new(5.0, 0.0, 0.0),
        });
}
pub fn get_app() -> App {
    let mut app = App::new();
    app
        .add_plugins((
            bevy::app::PanicHandlerPlugin,
            bevy::log::LogPlugin::default(),
            TaskPoolPlugin::default(),
            FrameCountPlugin,
            bevy::time::TimePlugin,
            TransformPlugin,
            bevy::diagnostic::DiagnosticsPlugin,
            bevy::input::InputPlugin,
            bevy::app::ScheduleRunnerPlugin::default(),
        ))
        .add_plugins((
            bevy::a11y::AccessibilityPlugin,
            bevy::app::TerminalCtrlCHandlerPlugin,
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy::winit::WinitPlugin::<bevy::winit::WakeUp>::default(),
            bevy::render::RenderPlugin::default(),
            ImagePlugin::default(),
            bevy::render::pipelined_rendering::PipelinedRenderingPlugin,
            bevy::core_pipeline::CorePipelinePlugin,
            bevy::sprite::SpritePlugin::default(),
        ))
        .add_plugins((
            bevy::text::TextPlugin,
            bevy::ui::UiPlugin::default(),
            bevy::pbr::PbrPlugin::default(),
            bevy::gltf::GltfPlugin::default(),
            bevy::audio::AudioPlugin::default(),
            GilrsPlugin,
            AnimationPlugin,
            bevy::gizmos::GizmoPlugin,
            bevy::state::app::StatesPlugin,
            DefaultPickingPlugins,
        ))
        .insert_resource(Events::<bevy::window::WindowResized>::default());
    
    register_rendering_systems(&mut app);
    register_simulation_systems(&mut app);
    
    app
}