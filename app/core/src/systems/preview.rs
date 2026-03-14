//! Next fruit preview system
//!
//! This module handles the display of the next fruit that will be spawned.
//! The preview shows a smaller version of the fruit above the spawn position.

use bevy::prelude::*;

use bevy::sprite::Anchor;

use crate::components::{Fruit, FruitSpawnState, NextFruitPreview};
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
                .map(|p| (p.radius, p.sprite_scale, p.sprite_anchor_x, p.sprite_anchor_y))
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
        .unwrap_or_else(|| (circle_texture.0.clone(), next_fruit.get().placeholder_color()));

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
        let sprites_changed = fruit_sprites.as_ref().map(|s| s.is_changed()).unwrap_or(false);
        if next_fruit.is_changed() || sprites_changed {
            let (image, color) = fruit_sprites
                .as_deref()
                .map(|s| s.resolve(next_fruit.get(), circle_texture.0.clone()))
                .unwrap_or_else(|| {
                    (circle_texture.0.clone(), next_fruit.get().placeholder_color())
                });
            sprite.image = image;
            sprite.color = color;

            if let Some(fruits_cfg) = fruits_config {
                let preview_scale = game_rules.map(|r| r.preview_scale).unwrap_or(1.5);
                if let Some(params) = next_fruit.get().try_parameters_from_config(fruits_cfg) {
                    sprite.custom_size =
                        Some(Vec2::splat(params.radius * 2.0 * params.sprite_scale * preview_scale));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;
    use crate::resources::CircleTexture;
    use bevy::asset::Assets;

    /// Helper to setup test app with required resources
    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Create and add config assets
        let mut fruits_assets = Assets::<FruitsConfig>::default();
        let fruits_config = FruitsConfig {
            fruits: vec![
                FruitConfigEntry {
                    name: "Cherry".to_string(),
                    radius: 20.0,
                    points: 10,
                    restitution: 0.3,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                    ..Default::default()
                },
                FruitConfigEntry {
                    name: "Strawberry".to_string(),
                    radius: 30.0,
                    points: 20,
                    restitution: 0.3,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                    ..Default::default()
                },
                FruitConfigEntry {
                    name: "Grape".to_string(),
                    radius: 40.0,
                    points: 40,
                    restitution: 0.3,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                    ..Default::default()
                },
                FruitConfigEntry {
                    name: "Dekopon".to_string(),
                    radius: 50.0,
                    points: 80,
                    restitution: 0.25,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                    ..Default::default()
                },
                FruitConfigEntry {
                    name: "Persimmon".to_string(),
                    radius: 60.0,
                    points: 160,
                    restitution: 0.25,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                    ..Default::default()
                },
                FruitConfigEntry {
                    name: "Apple".to_string(),
                    radius: 70.0,
                    points: 320,
                    restitution: 0.25,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                    ..Default::default()
                },
                FruitConfigEntry {
                    name: "Pear".to_string(),
                    radius: 80.0,
                    points: 640,
                    restitution: 0.25,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                    ..Default::default()
                },
                FruitConfigEntry {
                    name: "Peach".to_string(),
                    radius: 90.0,
                    points: 1280,
                    restitution: 0.2,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                    ..Default::default()
                },
                FruitConfigEntry {
                    name: "Pineapple".to_string(),
                    radius: 100.0,
                    points: 2560,
                    restitution: 0.2,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                    ..Default::default()
                },
                FruitConfigEntry {
                    name: "Melon".to_string(),
                    radius: 110.0,
                    points: 5120,
                    restitution: 0.2,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                    ..Default::default()
                },
                FruitConfigEntry {
                    name: "Watermelon".to_string(),
                    radius: 120.0,
                    points: 10240,
                    restitution: 0.2,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                    ..Default::default()
                },
            ],
        };
        let fruits_handle = fruits_assets.add(fruits_config);

        let mut physics_assets = Assets::<PhysicsConfig>::default();
        let physics_config = PhysicsConfig {
            gravity: -980.0,
            container_width: 600.0,
            container_height: 800.0,
            wall_thickness: 20.0,
            boundary_line_y: 300.0,
            wall_restitution: 0.2,
            wall_friction: 0.5,
            fruit_spawn_y_offset: 50.0,
            fruit_spawn_x_offset: 0.0,
            fruit_linear_damping: 0.5,
            fruit_angular_damping: 1.0,
            keyboard_move_speed: 300.0,
        };
        let physics_handle = physics_assets.add(physics_config);

        let mut game_rules_assets = Assets::<GameRulesConfig>::default();
        let game_rules_config = GameRulesConfig {
            spawnable_fruit_count: 5,
            combo_window: 2.0,
            combo_max: 10,
            game_over_timer: 3.0,
            combo_bonuses: std::collections::HashMap::new(),
            preview_x_offset: 120.0,
            preview_y_offset: -100.0,
            preview_scale: 1.5,
        };
        let game_rules_handle = game_rules_assets.add(game_rules_config);

        app.insert_resource(fruits_assets);
        app.insert_resource(FruitsConfigHandle(fruits_handle));
        app.insert_resource(physics_assets);
        app.insert_resource(PhysicsConfigHandle(physics_handle));
        app.insert_resource(game_rules_assets);
        app.insert_resource(GameRulesConfigHandle(game_rules_handle));
        app.init_resource::<NextFruitType>();
        app.insert_resource(CircleTexture(Handle::default()));

        app
    }

    #[test]
    fn test_setup_fruit_preview_creates_entity() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_fruit_preview);

        app.update();

        // Verify that a preview entity was created
        let count = app
            .world_mut()
            .query_filtered::<Entity, With<NextFruitPreview>>()
            .iter(app.world())
            .count();

        assert_eq!(count, 1, "Should create exactly one preview entity");
    }

    #[test]
    fn test_setup_fruit_preview_has_correct_components() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_fruit_preview);

        app.update();

        // Verify the preview has the required components
        let preview_data = app
            .world_mut()
            .query_filtered::<(&Sprite, &Transform, &Visibility), With<NextFruitPreview>>()
            .iter(app.world())
            .next();

        assert!(
            preview_data.is_some(),
            "Preview should have Sprite, Transform, and Visibility components"
        );

        let (sprite, transform, visibility) = preview_data.unwrap();
        assert!(
            sprite.custom_size.is_some(),
            "Sprite should have custom size"
        );
        assert!(
            transform.translation.z > 0.0,
            "Preview should be rendered above other sprites"
        );
        assert_eq!(
            *visibility,
            Visibility::Hidden,
            "Preview should start hidden"
        );
    }

    #[test]
    fn test_preview_has_fixed_position() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_fruit_preview);
        app.add_systems(Update, update_fruit_preview);

        // Run startup
        app.update();

        let initial_transform = app
            .world_mut()
            .query_filtered::<&Transform, With<NextFruitPreview>>()
            .iter(app.world())
            .next()
            .unwrap()
            .clone();

        // Run multiple updates
        app.update();
        app.update();
        app.update();

        // Verify preview position remains fixed
        let final_transform = app
            .world_mut()
            .query_filtered::<&Transform, With<NextFruitPreview>>()
            .iter(app.world())
            .next()
            .unwrap();

        assert_eq!(
            initial_transform.translation.x, final_transform.translation.x,
            "Preview X position should remain fixed"
        );
        assert_eq!(
            initial_transform.translation.y, final_transform.translation.y,
            "Preview Y position should remain fixed"
        );
    }

    #[test]
    fn test_update_fruit_preview_updates_on_type_change() {
        use crate::fruit::FruitType;

        let mut app = setup_test_app();
        app.add_systems(Startup, setup_fruit_preview);
        app.add_systems(Update, update_fruit_preview);

        app.update();

        // Get initial color
        let initial_color = app
            .world_mut()
            .query_filtered::<&Sprite, With<NextFruitPreview>>()
            .iter(app.world())
            .next()
            .unwrap()
            .color;

        // Change next fruit type
        app.world_mut()
            .resource_mut::<NextFruitType>()
            .set(FruitType::Strawberry);

        app.update();

        // Get updated color
        let updated_color = app
            .world_mut()
            .query_filtered::<&Sprite, With<NextFruitPreview>>()
            .iter(app.world())
            .next()
            .unwrap()
            .color;

        assert_ne!(
            initial_color, updated_color,
            "Preview color should change when fruit type changes"
        );
    }

    #[test]
    fn test_preview_position_is_on_right_side() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_fruit_preview);

        app.update();

        let transform = app
            .world_mut()
            .query_filtered::<&Transform, With<NextFruitPreview>>()
            .iter(app.world())
            .next()
            .unwrap();

        // Verify preview is positioned to the right of the container
        let expected_x = 600.0 / 2.0 + 120.0; // Default container width
        assert_eq!(
            transform.translation.x, expected_x,
            "Preview should be positioned on the right side"
        );

        // Verify preview is in the upper area
        assert!(
            transform.translation.y > 0.0,
            "Preview should be in the upper area"
        );
    }

    #[test]
    fn test_preview_visibility_based_on_held_fruit() {
        use crate::fruit::FruitType;

        let mut app = setup_test_app();
        app.add_systems(Startup, setup_fruit_preview);
        app.add_systems(Update, update_fruit_preview);

        app.update();

        // Initially no held fruit, preview should be hidden
        let visibility = app
            .world_mut()
            .query_filtered::<&Visibility, With<NextFruitPreview>>()
            .iter(app.world())
            .next()
            .unwrap();

        assert_eq!(
            *visibility,
            Visibility::Hidden,
            "Preview should be hidden when no held fruit exists"
        );

        // Spawn a held fruit
        app.world_mut().spawn((
            Fruit,
            FruitType::Cherry,
            FruitSpawnState::Held,
            Transform::default(),
        ));

        app.update();

        // With held fruit, preview should be visible
        let visibility = app
            .world_mut()
            .query_filtered::<&Visibility, With<NextFruitPreview>>()
            .iter(app.world())
            .next()
            .unwrap();

        assert_eq!(
            *visibility,
            Visibility::Visible,
            "Preview should be visible when held fruit exists"
        );
    }
}
