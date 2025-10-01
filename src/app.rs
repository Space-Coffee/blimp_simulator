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
use crate::render::{CustomRendererPlugin};
use crate::simulation::BlimpSimulationPlugin;

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
    app.add_plugins(CustomRendererPlugin {});
    app.add_plugins(BlimpSimulationPlugin {});

    app
}