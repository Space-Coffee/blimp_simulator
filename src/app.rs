use std::sync::Mutex;

use bevy::animation::AnimationPlugin;
use bevy::app::App;
use bevy::asset::{AssetPlugin, Assets};
use bevy::color::Color;
use bevy::core::{FrameCountPlugin, TaskPoolPlugin};
use bevy::gilrs::GilrsPlugin;
use bevy::math::Vec3;
use bevy::pbr::{MeshMaterial3d, PointLight, StandardMaterial};
use bevy::picking::DefaultPickingPlugins;
use bevy::prelude::*;

use crate::render::CustomRendererPlugin;
use crate::simulation::BlimpSimulationPlugin;
use crate::AsyncSyncBridge;

pub fn get_app(as_bridge_rx: tokio::sync::oneshot::Receiver<AsyncSyncBridge>) -> App {
    let as_bridge = as_bridge_rx.blocking_recv().unwrap();

    let mut app = App::new();
    app.add_plugins((
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
    .register_type::<bevy::hierarchy::Children>()
    .register_type::<bevy::hierarchy::Parent>()
    .register_type::<bevy::core::Name>()
    .insert_resource(Events::<bevy::window::WindowResized>::default())
    .add_plugins(CustomRendererPlugin {})
    .add_plugins(BlimpSimulationPlugin {
        motors_servos_rx: Mutex::new(Some(as_bridge.motors_servos_rx)),
    });

    app
}
