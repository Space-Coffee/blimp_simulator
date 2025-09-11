use bevy::prelude::*;
use bevy_headless_render;

pub async fn virtual_camera_start() {
    tokio::task::spawn_blocking(|| {
        fn setup(mut cmds: Commands, asset_server: ResMut<AssetServer>) {
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
            ));
        }

        let mut bevy_app = App::new();
        bevy_app
            .add_plugins(DefaultPlugins)
            .add_plugins(bevy_headless_render::HeadlessRenderPlugin)
            .add_systems(Startup, setup);

        bevy_app.run();
    });
}
