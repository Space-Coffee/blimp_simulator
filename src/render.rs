use std::cmp::PartialEq;
use std::io::Write;

use bevy::prelude::*;
use bevy_headless_render;

enum DisplayMode {
    Ffplay,
    Stream,
    Window
}

const DEBUG_RENDER: bool = false;
const DISPLAY_MODE: DisplayMode = DisplayMode::Window;

#[derive(Resource)]
struct FfmpegProcess(std::process::Child);


fn render_ffmpeg(
    dest: Query<&bevy_headless_render::components::HeadlessRenderDestination>,
    ffmpeg: ResMut<FfmpegProcess>,
) {
    let mut ffmpeg_stdin = ffmpeg.0.stdin.as_ref().expect("Failed to get ffmpeg stdin");
    let dest = dest.single().0.clone();
    let dest = dest.lock().unwrap();

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

fn setup_windowed_render(
    mut cmds: Commands
) {
    cmds.spawn((
        Camera3d::default(),
        Camera::default(),
        Transform::from_xyz(0.0, 2.5, -2.0)
            .looking_at(Vec3::new(5.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
    ));
    
}

pub fn apply_windowed_config(mut app: &mut App) {
    app.add_plugins(WindowPlugin::default());
    app.add_systems(Startup, setup_windowed_render);
}

pub fn apply_headless_config(mut app: &mut App, ffplay: bool) {
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
    if !ffplay {
        args.push("-i".to_owned());
    }
    args.push("pipe:0".to_owned());
    if !ffplay {
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
    let ffmpeg = std::process::Command::new(if ffplay { "ffplay" } else { "ffmpeg" })
        .args(args)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("Couldn't start ffmpeg");
    
    app.insert_resource(FfmpegProcess(ffmpeg));
    app.add_systems(PostUpdate, render_ffmpeg);
    app.add_plugins((
        bevy_headless_render::HeadlessRenderPlugin,
        WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        }
    ));
    app.add_systems(Startup, setup_headless_render);
}

pub fn register_rendering_systems(app: &mut App) {
    match DISPLAY_MODE {
        DisplayMode::Window => apply_windowed_config(app),
        DisplayMode::Stream => apply_headless_config(app, false),
        DisplayMode::Ffplay => apply_headless_config(app, true)
    }
}