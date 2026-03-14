//! Fruit sprite loading system.
//!
//! Loads available fruit sprite images from `assets/images/fruits/` via the
//! [`AssetServer`].  Only fruit types that have artwork are registered; the
//! rest fall back to the procedurally generated circle placeholder.
//!
//! # Adding new sprites
//!
//! 1. Place the image at `assets/images/fruits/<name>.png`.
//! 2. Add a `fruit_sprites.insert(FruitType::Name, asset_server.load("..."))` line below.

use bevy::prelude::*;
use suika_game_core::fruit::FruitType;
use suika_game_core::resources::FruitSprites;

/// Loads available fruit sprites into the [`FruitSprites`] resource.
///
/// Registered on `Startup` by [`crate::GameAssetsPlugin`].
///
/// Currently only `cherry.png` exists; more sprites will be added as artwork
/// is created.
pub fn load_fruit_sprites(asset_server: Res<AssetServer>, mut fruit_sprites: ResMut<FruitSprites>) {
    // Cherry — experimental first sprite to validate the pipeline.
    fruit_sprites.insert(
        FruitType::Cherry,
        asset_server.load("images/fruits/cherry.png"),
    );

    info!("Fruit sprites queued for loading: cherry");
}
