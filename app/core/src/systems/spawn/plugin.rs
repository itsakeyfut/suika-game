//! SpawnPlugin — registers spawn-related startup systems

use bevy::prelude::*;

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, super::circle::setup_circle_texture);
    }
}
