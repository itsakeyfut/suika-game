//! # suika-game-assets
//!
//! Asset loading for the Suika Game: sprites, sounds, and fonts.

use bevy::prelude::*;

pub mod sprites;

/// Asset management plugin.
///
/// Registers all asset-loading systems with the Bevy app.
/// Must be added after `DefaultPlugins` (which includes `AssetPlugin`).
pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, sprites::load_fruit_sprites);
        info!("GameAssetsPlugin initialized");
    }
}
