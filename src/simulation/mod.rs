mod physics;
pub mod setup;

use std::sync::Mutex;

use bevy::app::{App, Plugin, Startup};
use bevy::prelude::*;

use crate::simulation::physics::PhysicsPlugin;

pub struct BlimpSimulationPlugin {
    pub motors_servos_rx: Mutex<Option<tokio::sync::watch::Receiver<([f32; 4], [f32; 12])>>>,
}

impl Plugin for BlimpSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup::setup);
        app.add_plugins(PhysicsPlugin {
            motors_servos_rx: self.motors_servos_rx.lock().unwrap().take().unwrap(),
        });
    }
}

#[derive(Component)]
pub struct BlimpComponent;
