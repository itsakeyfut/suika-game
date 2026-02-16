//! Next fruit preview system
//!
//! This module handles the display of the next fruit that will be spawned.
//! The preview shows a smaller version of the fruit above the spawn position.

use bevy::prelude::*;

use crate::components::{Fruit, FruitSpawnState, NextFruitPreview};
use crate::constants::physics;
use crate::resources::NextFruitType;

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
pub fn setup_fruit_preview(mut commands: Commands, next_fruit: Res<NextFruitType>) {
    let params = next_fruit.get().parameters();

    // Preview position: fixed position on the right side of the screen
    // Positioned to the right of the container with some margin
    let preview_x = physics::CONTAINER_WIDTH / 2.0 + 120.0;
    let preview_y = physics::CONTAINER_HEIGHT / 2.0 - 100.0; // Upper area

    commands.spawn((
        NextFruitPreview,
        Sprite {
            color: next_fruit.get().placeholder_color(),
            custom_size: Some(Vec2::splat(params.radius * 1.5)),
            ..default()
        },
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
    mut preview_query: Query<(&mut Sprite, &mut Visibility), With<NextFruitPreview>>,
    next_fruit: Res<NextFruitType>,
    fruit_states: Query<&FruitSpawnState, With<Fruit>>,
) {
    // Check if there's a held or falling fruit
    let has_held_fruit = fruit_states
        .iter()
        .any(|state| *state == FruitSpawnState::Held);

    let has_falling_fruit = fruit_states
        .iter()
        .any(|state| *state == FruitSpawnState::Falling);

    for (mut sprite, mut visibility) in preview_query.iter_mut() {
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

        // Update preview when next fruit type changes
        if next_fruit.is_changed() {
            let params = next_fruit.get().parameters();
            sprite.color = next_fruit.get().placeholder_color();
            sprite.custom_size = Some(Vec2::splat(params.radius * 1.5));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_fruit_preview_creates_entity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<NextFruitType>();
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
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<NextFruitType>();
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
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<NextFruitType>();
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

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<NextFruitType>();
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
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<NextFruitType>();
        app.add_systems(Startup, setup_fruit_preview);

        app.update();

        let transform = app
            .world_mut()
            .query_filtered::<&Transform, With<NextFruitPreview>>()
            .iter(app.world())
            .next()
            .unwrap();

        // Verify preview is positioned to the right of the container
        let expected_x = physics::CONTAINER_WIDTH / 2.0 + 120.0;
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

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<NextFruitType>();
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
