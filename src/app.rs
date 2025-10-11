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
use tokio::sync::{oneshot, watch};

use crate::render::CustomRendererPlugin;
use crate::simulation::BlimpSimulationPlugin;
use crate::{AsyncSyncBridge, SyncAsyncBridge};

#[derive(Resource)]
pub struct AsyncSyncBridgeRes(pub AsyncSyncBridge);

#[derive(Resource)]
pub struct SyncAsyncBridgeRes {
    pub pos_tx: watch::Sender<(f32, f32, f32)>,
    pub rot_tx: watch::Sender<(f32, f32, f32)>,
}

pub fn get_app(
    as_bridge_rx: oneshot::Receiver<AsyncSyncBridge>,
    sa_bridge_tx: oneshot::Sender<SyncAsyncBridge>,
) -> App {
    let (pos_tx, pos_rx) = watch::channel((0.0, 0.0, 0.0));
    let (rot_tx, rot_rx) = watch::channel((0.0, 0.0, 0.0));
    let sa_bridge = SyncAsyncBridge { pos_rx, rot_rx };
    sa_bridge_tx
        .send(sa_bridge)
        .map_err(|_| "Couldn't send sync-async bridge")
        .unwrap();
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
    .insert_resource(AsyncSyncBridgeRes(as_bridge))
    .insert_resource(SyncAsyncBridgeRes { pos_tx, rot_tx })
    .add_plugins(CustomRendererPlugin {})
    .add_plugins(BlimpSimulationPlugin);

    app
}
