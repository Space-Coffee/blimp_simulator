mod physics;
pub mod setup;
pub mod util;
mod constants;

use std::sync::Mutex;

use bevy::app::{App, Plugin, Startup};
use bevy::prelude::*;

use crate::simulation::physics::PhysicsPlugin;

pub struct BlimpSimulationPlugin;

impl Plugin for BlimpSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup::setup);
        app.add_plugins(PhysicsPlugin);
    }
}

#[derive(Component)]
pub struct BlimpComponent;
