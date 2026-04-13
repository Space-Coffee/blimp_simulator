pub mod camera;
mod headless;
mod windowed;

use crate::render::headless::apply_headless_config;
use crate::render::windowed::apply_windowed_config;
use bevy::prelude::*;

enum DisplayMode {
    Ffplay,
    Stream,
    Window,
}

const DEBUG_RENDER: bool = false;
const DISPLAY_MODE: DisplayMode = DisplayMode::Window;

pub struct CustomRendererPlugin;

impl Plugin for CustomRendererPlugin {
    fn build(&self, app: &mut App) {
        match DISPLAY_MODE {
            DisplayMode::Window => apply_windowed_config(app),
            DisplayMode::Stream => apply_headless_config(app, false, DEBUG_RENDER),
            DisplayMode::Ffplay => apply_headless_config(app, true, DEBUG_RENDER),
        }
    }
}
