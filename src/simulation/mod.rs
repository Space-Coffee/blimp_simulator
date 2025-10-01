mod physics;
pub mod setup;

use bevy::app::{App, Plugin, Startup};
use bevy::prelude::*;
use crate::simulation::physics::PhysicsPlugin;

pub struct BlimpSimulationPlugin {}

impl Plugin for BlimpSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup::setup);
        app.add_plugins(PhysicsPlugin);
    }
}
