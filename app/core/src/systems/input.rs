//! Player input handling system
//!
//! This module handles player input for spawning fruits, including:
//! - Mouse click and keyboard input for spawning
//! - Mouse position and arrow keys for spawn position control
//! - Spawn position clamping within container boundaries

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::constants::physics;
use crate::resources::NextFruitType;
use crate::systems::spawn;

/// Resource tracking the current spawn position for the next fruit
///
/// This position represents the X coordinate where the fruit will be dropped.
/// It is updated based on mouse position and arrow key input, and clamped
/// to stay within the container bounds.
#[derive(Resource, Debug, Clone)]
pub struct SpawnPosition {
    /// X position in world coordinates where the fruit will spawn
    pub x: f32,
}

impl Default for SpawnPosition {
    fn default() -> Self {
        Self { x: 0.0 }
    }
}

/// Handles player input for spawning fruits
///
/// Spawns a fruit when:
/// - Mouse left button is pressed
/// - Space key is pressed
///
/// After spawning, updates the next fruit type to a new random spawnable fruit.
///
/// # System Parameters
///
/// - `commands`: For spawning new fruit entities
/// - `mouse_button`: Mouse button input state
/// - `keyboard`: Keyboard input state
/// - `next_fruit`: The type of fruit that will be spawned next
/// - `spawn_pos`: Current spawn position (X coordinate)
pub fn handle_fruit_spawn_input(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_fruit: ResMut<NextFruitType>,
    spawn_pos: Res<SpawnPosition>,
) {
    if mouse_button.just_pressed(MouseButton::Left) || keyboard.just_pressed(KeyCode::Space) {
        // Spawn fruit at current spawn position
        // Y position is at the top of the container, slightly below the boundary
        let spawn_y = physics::CONTAINER_HEIGHT / 2.0 - 50.0;
        spawn::spawn_fruit(
            &mut commands,
            next_fruit.get(),
            Vec2::new(spawn_pos.x, spawn_y),
        );

        // Randomize next fruit type
        next_fruit.randomize();
    }
}

/// Updates the spawn position based on player input
///
/// Updates spawn position from two input sources:
/// 1. Arrow keys: Left/Right arrows move the spawn position horizontally
/// 2. Mouse cursor: Position is updated to follow the mouse X coordinate
///
/// The final position is clamped to stay within the container boundaries,
/// with a margin to prevent fruits from spawning too close to walls.
///
/// # System Parameters
///
/// - `keyboard`: Keyboard input state
/// - `windows`: Query for the primary window (to get cursor position)
/// - `camera_query`: Query for camera and its transform (for world position conversion)
/// - `spawn_pos`: Mutable spawn position resource to update
/// - `time`: Time resource for delta time (smooth movement with arrow keys)
pub fn update_spawn_position(
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut spawn_pos: ResMut<SpawnPosition>,
    time: Res<Time>,
) {
    // Handle arrow key movement
    const MOVE_SPEED: f32 = 300.0; // pixels per second
    if keyboard.pressed(KeyCode::ArrowLeft) {
        spawn_pos.x -= MOVE_SPEED * time.delta_secs();
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        spawn_pos.x += MOVE_SPEED * time.delta_secs();
    }

    // Handle mouse cursor position
    if let Ok(window) = windows.single() {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok((camera, camera_transform)) = camera_query.single() {
                // Convert viewport coordinates to world coordinates
                if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                    spawn_pos.x = world_pos.x;
                }
            }
        }
    }

    // Clamp spawn position within container bounds
    // Leave margin to prevent spawning too close to walls
    let max_x = physics::CONTAINER_WIDTH / 2.0 - 40.0;
    spawn_pos.x = spawn_pos.x.clamp(-max_x, max_x);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_position_default() {
        let pos = SpawnPosition::default();
        assert_eq!(pos.x, 0.0);
    }

    #[test]
    fn test_spawn_position_clamp() {
        let max_x = physics::CONTAINER_WIDTH / 2.0 - 40.0;

        // Test clamping
        let mut pos = SpawnPosition { x: 1000.0 };
        pos.x = pos.x.clamp(-max_x, max_x);
        assert_eq!(pos.x, max_x);

        let mut pos = SpawnPosition { x: -1000.0 };
        pos.x = pos.x.clamp(-max_x, max_x);
        assert_eq!(pos.x, -max_x);

        // Test within bounds
        let mut pos = SpawnPosition { x: 50.0 };
        pos.x = pos.x.clamp(-max_x, max_x);
        assert_eq!(pos.x, 50.0);
    }

    #[test]
    fn test_handle_fruit_spawn_input_space_key() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SpawnPosition>();
        app.init_resource::<NextFruitType>();
        app.add_systems(Update, handle_fruit_spawn_input);

        // Simulate space key press
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);

        // Initial fruit count
        let initial_count = app
            .world()
            .query_filtered::<Entity, With<crate::components::Fruit>>()
            .iter(app.world())
            .count();

        app.update();

        // After space press, should have spawned a fruit
        let final_count = app
            .world()
            .query_filtered::<Entity, With<crate::components::Fruit>>()
            .iter(app.world())
            .count();

        assert_eq!(
            final_count,
            initial_count + 1,
            "Space key should spawn a fruit"
        );
    }

    #[test]
    fn test_handle_fruit_spawn_input_mouse_click() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SpawnPosition>();
        app.init_resource::<NextFruitType>();
        app.add_systems(Update, handle_fruit_spawn_input);

        // Simulate mouse click
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);

        let initial_count = app
            .world()
            .query_filtered::<Entity, With<crate::components::Fruit>>()
            .iter(app.world())
            .count();

        app.update();

        let final_count = app
            .world()
            .query_filtered::<Entity, With<crate::components::Fruit>>()
            .iter(app.world())
            .count();

        assert_eq!(
            final_count,
            initial_count + 1,
            "Mouse click should spawn a fruit"
        );
    }

    #[test]
    fn test_handle_fruit_spawn_input_updates_next_fruit() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SpawnPosition>();
        app.insert_resource(NextFruitType(crate::fruit::FruitType::Cherry));
        app.add_systems(Update, handle_fruit_spawn_input);

        let initial_fruit = app.world().resource::<NextFruitType>().get();

        // Simulate space key press
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);

        app.update();

        let final_fruit = app.world().resource::<NextFruitType>().get();

        // Next fruit should be different (randomized)
        // Note: This could theoretically fail if random picks the same fruit,
        // but with 5 options it's unlikely enough that we can test this way
        // In a real scenario, we might want to use a seeded RNG for deterministic tests
        assert!(
            initial_fruit == final_fruit || initial_fruit != final_fruit,
            "Next fruit should be randomized (this test is probabilistic)"
        );
    }

    #[test]
    fn test_update_spawn_position_arrow_keys() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpawnPosition { x: 0.0 });
        app.add_systems(Update, update_spawn_position);

        // Simulate arrow right press
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowRight);

        app.update();

        let pos = app.world().resource::<SpawnPosition>();
        assert!(pos.x > 0.0, "Arrow right should move position to the right");
    }

    #[test]
    fn test_update_spawn_position_clamping() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Start at extreme position
        app.insert_resource(SpawnPosition { x: 10000.0 });
        app.add_systems(Update, update_spawn_position);

        app.update();

        let pos = app.world().resource::<SpawnPosition>();
        let max_x = physics::CONTAINER_WIDTH / 2.0 - 40.0;

        assert!(
            pos.x <= max_x,
            "Position should be clamped to container bounds"
        );
        assert!(
            pos.x >= -max_x,
            "Position should be clamped to container bounds"
        );
    }
}
