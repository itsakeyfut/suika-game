//! Next fruit preview system
//!
//! This module handles the display of the next fruit that will be spawned.
//! The preview shows a smaller version of the fruit above the spawn position.

pub mod setup;
pub mod update;

pub use setup::setup_fruit_preview;
pub use update::update_fruit_preview;

use bevy::prelude::*;

pub struct PreviewPlugin;

impl Plugin for PreviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnExit(crate::states::AppState::Loading),
            setup_fruit_preview,
        );
        app.add_systems(Update, update_fruit_preview);
    }
}

#[cfg(test)]
mod tests {
    use crate::components::{Fruit, FruitSpawnState, NextFruitPreview};
    use crate::config::*;
    use crate::resources::{CircleTexture, NextFruitType};
    use crate::systems::preview::setup::setup_fruit_preview;
    use crate::systems::preview::update::update_fruit_preview;
    use bevy::asset::Assets;
    use bevy::prelude::*;

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
