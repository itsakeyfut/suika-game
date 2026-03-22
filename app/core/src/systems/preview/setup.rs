//! Next fruit preview setup system

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::components::NextFruitPreview;
use crate::config::{
    FruitsConfig, FruitsConfigHandle, GameRulesConfig, GameRulesConfigHandle, PhysicsConfig,
    PhysicsConfigHandle,
};
use crate::resources::{CircleTexture, FruitSprites, NextFruitType};

/// Sets up the next fruit preview display
///
/// Creates a preview entity showing the next fruit that will be spawned.
/// The preview is displayed in a fixed position on the right side of the screen,
/// similar to Tetris or Puyo Puyo next piece preview.
///
/// The preview starts hidden and will become visible when the first held fruit
/// is spawned (after the first fruit lands).
///
/// # System Parameters
///
/// - `commands`: For spawning the preview entity
/// - `next_fruit`: The type of fruit to preview
///
/// # Note
///
/// This system should run during Startup to create the initial preview entity.
#[allow(clippy::too_many_arguments)]
pub fn setup_fruit_preview(
    mut commands: Commands,
    next_fruit: Res<NextFruitType>,
    fruits_config_handle: Res<FruitsConfigHandle>,
    fruits_config_assets: Res<Assets<FruitsConfig>>,
    physics_config_handle: Res<PhysicsConfigHandle>,
    physics_config_assets: Res<Assets<PhysicsConfig>>,
    game_rules_handle: Res<GameRulesConfigHandle>,
    game_rules_assets: Res<Assets<GameRulesConfig>>,
    circle_texture: Res<CircleTexture>,
    fruit_sprites: Option<Res<FruitSprites>>,
) {
    // Resolve sprite image and color (real sprite or tinted placeholder).
    let (radius, sprite_scale, anchor_x, anchor_y) =
        if let Some(config) = fruits_config_assets.get(&fruits_config_handle.0) {
            next_fruit
                .get()
                .try_parameters_from_config(config)
                .map(|p| {
                    (
                        p.radius,
                        p.sprite_scale,
                        p.sprite_anchor_x,
                        p.sprite_anchor_y,
                    )
                })
                .unwrap_or_else(|| {
                    warn!(
                        "⚠️ No config entry for fruit {:?}, using defaults",
                        next_fruit.get()
                    );
                    (20.0, 1.0, 0.0, 0.0)
                })
        } else {
            warn!("Fruits config not loaded yet, using defaults for preview");
            (20.0, 1.0, 0.0, 0.0)
        };

    let (image, color) = fruit_sprites
        .as_deref()
        .map(|s| s.resolve(next_fruit.get(), circle_texture.0.clone()))
        .unwrap_or_else(|| {
            (
                circle_texture.0.clone(),
                next_fruit.get().placeholder_color(),
            )
        });

    // Get preview position and scale from game rules config
    let (preview_x_offset, preview_y_offset, preview_scale) =
        if let Some(rules) = game_rules_assets.get(&game_rules_handle.0) {
            (
                rules.preview_x_offset,
                rules.preview_y_offset,
                rules.preview_scale,
            )
        } else {
            (120.0, -100.0, 1.5) // Fallback defaults
        };

    // Get container dimensions from physics config
    let (container_width, container_height) =
        if let Some(physics) = physics_config_assets.get(&physics_config_handle.0) {
            (physics.container_width, physics.container_height)
        } else {
            (600.0, 800.0) // Fallback defaults
        };

    // Preview position: positioned relative to container
    let preview_x = container_width / 2.0 + preview_x_offset;
    let preview_y = container_height / 2.0 + preview_y_offset;

    commands.spawn((
        NextFruitPreview,
        Sprite {
            image,
            color,
            custom_size: Some(Vec2::splat(radius * 2.0 * sprite_scale * preview_scale)),
            ..default()
        },
        Anchor(Vec2::new(anchor_x, anchor_y)),
        Transform::from_xyz(preview_x, preview_y, 10.0),
        Visibility::Hidden, // Start hidden, will show when held fruit spawns
    ));
}
