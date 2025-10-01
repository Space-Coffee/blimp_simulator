use std::cmp::PartialEq;
use std::io::Write;

use bevy::prelude::*;
use bevy_headless_render;

#[derive(PartialEq)]
enum DisplayMode {
    Ffplay,
    Stream,
    Window
}

const DEBUG_RENDER: bool = false;
const DISPLAY_MODE: DisplayMode = DisplayMode::Window;

#[derive(Resource)]
struct FfmpegProcess(std::process::Child);

#[derive(Resource)]
struct VirtualBlimpData {
    pos: Vec3,
}

#[derive(Component)]
struct BlimpComponent;

fn render_ffmpeg(
    dest: Query<&bevy_headless_render::components::HeadlessRenderDestination>,
    ffmpeg: ResMut<FfmpegProcess>,
) {
    let mut ffmpeg_stdin = ffmpeg.0.stdin.as_ref().expect("Failed to get ffmpeg stdin");
    let dest = dest.single().0.clone();
    let dest = dest.lock().unwrap();

    if DISPLAY_MODE == DisplayMode::Window {
        panic!("Tried rendering to ffmpeg in windowed mode")
    }
    if DEBUG_RENDER {
        let mut fake_data = Vec::new();
        fake_data.resize(dest.data.len(), 0);
        let mut counter: u8 = 0;
        for b in &mut fake_data {
            *b = counter;
            counter = counter.wrapping_add(1);
        }
        ffmpeg_stdin
            .write_all(&fake_data)
            .expect("Failed to send video data to ffmpeg");
    } else {
        ffmpeg_stdin
            .write_all(&dest.data)
            .expect("Failed to send video data to ffmpeg");
    }
}
fn setup_headless_render(
    mut cmds: Commands,
    asset_server: ResMut<AssetServer>,
) {
    // Headless rendering stuff
    let size = bevy::render::render_resource::Extent3d {
        width: 640,
        height: 480,
        depth_or_array_layers: 1,
    };
    let mut dest_image = Image {
        texture_descriptor: bevy::render::render_resource::TextureDescriptor {
            label: None,
            size,
            dimension: bevy::render::render_resource::TextureDimension::D2,
            format: bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
                | bevy::render::render_resource::TextureUsages::COPY_SRC
                | bevy::render::render_resource::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    dest_image.resize(size);

    let image_handle = asset_server.add(dest_image);

    // Spawn camera
    cmds.spawn((
        bevy_headless_render::components::HeadlessRenderSource::new(
            &asset_server,
            image_handle.clone(),
        ),
        Camera3d::default(),
        Camera {
            target: image_handle.into(),
            ..default()
        },
        Transform::from_xyz(0.0, 2.5, -2.0)
            .looking_at(Vec3::new(5.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
    ));
}

fn setup_window_render(
    mut cmds: Commands
) {
    cmds.spawn((
        Camera3d::default(),
        Camera::default(),
        Transform::from_xyz(0.0, 2.5, -2.0)
            .looking_at(Vec3::new(5.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
    ));
    
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
pub fn virtual_camera_start() {


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

    let mut args = Vec::<String>::new();
    args.extend_from_slice(
        &[
            "-f",
            "rawvideo",
            "-vcodec",
            "rawvideo",
            "-pixel_format",
            "rgba",
            "-colorspace",
            "bt709",
            "-video_size",
            "640x480",
            "-use_wallclock_as_timestamps",
            "1",
        ]
        .map(|s| s.to_owned()),
    );
    if DISPLAY_MODE == DisplayMode::Stream {
        args.push("-i".to_owned());
    }
    args.push("pipe:0".to_owned());
    if DISPLAY_MODE == DisplayMode::Stream {
        args.extend_from_slice(
            &[
                "-c:v",
                "libx264",
                "-tune",
                "zerolatency",
                "-crf",
                "18",
                "-f",
                "rtsp",
                "-rtsp_transport",
                "tcp",
                "rtsp://127.0.0.1:8554/virtual",
            ]
            .map(|s| s.to_owned()),
        );
    }

    let ffmpeg = match DISPLAY_MODE {
        DisplayMode::Ffplay | DisplayMode::Stream => {
            Some(std::process::Command::new(if DISPLAY_MODE == DisplayMode::Ffplay { "ffplay" } else { "ffmpeg" })
                .args(args)
                .stdin(std::process::Stdio::piped())
                .spawn()
                .expect("Couldn't start ffmpeg")
            )
        }
        DisplayMode::Window => {
            None
        }
    };

    let mut bevy_app = App::new();
    bevy_app
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
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, update_blimp)
        .insert_resource(VirtualBlimpData {
            pos: Vec3::new(5.0, 0.0, 0.0),
        })
        .insert_resource(Events::<bevy::window::WindowResized>::default());
    
    match DISPLAY_MODE {
        DisplayMode::Ffplay | DisplayMode::Stream => {
            bevy_app.insert_resource(FfmpegProcess(ffmpeg.unwrap()));
            bevy_app.add_systems(PostUpdate, render_ffmpeg);
            bevy_app.add_plugins((
                bevy_headless_render::HeadlessRenderPlugin,
                WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    ..default()
                }
            ));
            bevy_app.add_systems(Startup, setup_headless_render);
        },
        DisplayMode::Window => {
            bevy_app.add_plugins(WindowPlugin::default());
            bevy_app.add_systems(Startup, setup_window_render);
        }
    }
    
    bevy_app.run();
}
