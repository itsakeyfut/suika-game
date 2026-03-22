//! Next fruit preview update system

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::components::{Fruit, FruitSpawnState, NextFruitPreview};
use crate::config::{FruitsConfig, FruitsConfigHandle, GameRulesConfig, GameRulesConfigHandle};
use crate::resources::{CircleTexture, FruitSprites, NextFruitType};

/// Updates the fruit preview when the next fruit type changes
///
/// This system monitors changes to NextFruitType and updates the preview
/// sprite accordingly. The preview remains in a fixed position on the right side.
///
/// The preview visibility is controlled based on active fruit state:
/// - When a held or falling fruit exists: Preview is visible (shows NEXT fruit)
/// - When no active fruits exist: Preview is hidden
///
/// This ensures the preview stays visible during the entire drop sequence
/// (from holding to falling to landing), and only hides when waiting for
/// the next fruit to spawn.
///
/// # System Parameters
///
/// - `preview_query`: Query for the preview entity
/// - `next_fruit`: The current next fruit type
/// - `fruit_states`: Query to check fruit spawn states
///
/// # Behavior
///
/// - When NextFruitType changes: Updates color and size
/// - When held/falling fruit exists: Shows preview
/// - When no active fruits: Hides preview
/// - Position remains fixed (does not follow spawn position)
#[allow(clippy::too_many_arguments)]
pub fn update_fruit_preview(
    mut preview_query: Query<(&mut Sprite, &mut Visibility, &mut Anchor), With<NextFruitPreview>>,
    next_fruit: Res<NextFruitType>,
    fruit_states: Query<&FruitSpawnState, With<Fruit>>,
    fruits_config_handle: Res<FruitsConfigHandle>,
    fruits_config_assets: Res<Assets<FruitsConfig>>,
    game_rules_handle: Res<GameRulesConfigHandle>,
    game_rules_assets: Res<Assets<GameRulesConfig>>,
    circle_texture: Res<CircleTexture>,
    fruit_sprites: Option<Res<FruitSprites>>,
) {
    // Get the configs
    let fruits_config = fruits_config_assets.get(&fruits_config_handle.0);
    let game_rules = game_rules_assets.get(&game_rules_handle.0);
    // Check if there's a held or falling fruit
    let has_held_fruit = fruit_states
        .iter()
        .any(|state| *state == FruitSpawnState::Held);

    let has_falling_fruit = fruit_states
        .iter()
        .any(|state| *state == FruitSpawnState::Falling);

    for (mut sprite, mut visibility, mut anchor) in preview_query.iter_mut() {
        // Update preview visibility based on held or falling fruit existence
        // Keep preview visible during fruit drop (Held -> Falling transition)
        let desired = if has_held_fruit || has_falling_fruit {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        // Only update if changed to avoid triggering change detection unnecessarily
        if *visibility != desired {
            *visibility = desired;
        }

        // Update preview when next fruit type or sprite resource changes.
        // fruit_sprites.is_changed() fires when load_fruit_sprites inserts handles
        // at Startup, catching the case where setup_fruit_preview ran first.
        let sprites_changed = fruit_sprites
            .as_ref()
            .map(|s| s.is_changed())
            .unwrap_or(false);
        if next_fruit.is_changed() || sprites_changed {
            let (image, color) = fruit_sprites
                .as_deref()
                .map(|s| s.resolve(next_fruit.get(), circle_texture.0.clone()))
                .unwrap_or_else(|| {
                    (
                        circle_texture.0.clone(),
                        next_fruit.get().placeholder_color(),
                    )
                });
            sprite.image = image;
            sprite.color = color;

            if let Some(fruits_cfg) = fruits_config {
                let preview_scale = game_rules.map(|r| r.preview_scale).unwrap_or(1.5);
                if let Some(params) = next_fruit.get().try_parameters_from_config(fruits_cfg) {
                    sprite.custom_size = Some(Vec2::splat(
                        params.radius * 2.0 * params.sprite_scale * preview_scale,
                    ));
                    anchor.0 = Vec2::new(params.sprite_anchor_x, params.sprite_anchor_y);
                } else {
                    warn!(
                        "⚠️ No config entry for preview fruit {:?}, keeping previous size",
                        next_fruit.get()
                    );
                }
            }
        }
    }
}
