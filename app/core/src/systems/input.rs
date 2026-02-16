//! Player input handling system
//!
//! This module handles player input for fruit control, including:
//! - Spawning a held fruit at the start
//! - Mouse position and arrow keys (←→ or A/D) for position control
//! - Space key or mouse click to drop the fruit
//! - Automatic spawning of next fruit after drop

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

use crate::components::{Fruit, FruitSpawnState};
use crate::constants::physics;
use crate::fruit::FruitType;
use crate::resources::NextFruitType;

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

/// Spawns a new held fruit if none exists
///
/// This system runs once at startup and after each fruit is dropped.
/// It creates a fruit in the Held state that hovers at the top of the container.
///
/// # System Parameters
///
/// - `commands`: For spawning new fruit entities
/// - `next_fruit`: The type of fruit to spawn
/// - `spawn_pos`: Current spawn position (X coordinate)
/// - `held_fruits`: Query to check if a held fruit already exists
pub fn spawn_held_fruit(
    mut commands: Commands,
    next_fruit: Res<NextFruitType>,
    spawn_pos: Res<SpawnPosition>,
    held_fruits: Query<Entity, (With<Fruit>, With<FruitSpawnState>)>,
) {
    // Only spawn if there's no held fruit
    if held_fruits.is_empty() {
        let spawn_y = physics::CONTAINER_HEIGHT / 2.0 - 50.0;
        let params = next_fruit.get().parameters();

        commands.spawn((
            // Fruit marker and type
            Fruit,
            next_fruit.get(),
            FruitSpawnState::Held,
            // Sprite rendering
            Sprite {
                color: next_fruit.get().placeholder_color(),
                custom_size: Some(Vec2::splat(params.radius * 2.0)),
                ..default()
            },
            Transform::from_xyz(spawn_pos.x, spawn_y, 0.0),
            // Kinematic body (no gravity, manually controlled)
            RigidBody::KinematicPositionBased,
            // Collision shape (for preview, not for physics yet)
            Collider::ball(params.radius),
        ));

        info!("Spawned held fruit: {:?}", next_fruit.get());
    }
}

/// Handles player input for dropping held fruits
///
/// Drops the currently held fruit when:
/// - Mouse left button is pressed
/// - Space key is pressed
///
/// After dropping, the fruit transitions from Held to Falling state,
/// becomes a dynamic rigid body, and gets physics properties.
/// The next fruit type is randomized for the next spawn.
///
/// # System Parameters
///
/// - `commands`: For adding/removing components
/// - `mouse_button`: Mouse button input state
/// - `keyboard`: Keyboard input state
/// - `next_fruit`: The type of fruit that will be spawned next
/// - `held_fruits`: Query for held fruits to drop
pub fn handle_fruit_drop_input(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_fruit: ResMut<NextFruitType>,
    mut held_fruits: Query<(Entity, &FruitType, &mut FruitSpawnState), With<Fruit>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) || keyboard.just_pressed(KeyCode::Space) {
        for (entity, fruit_type, mut spawn_state) in held_fruits.iter_mut() {
            if *spawn_state == FruitSpawnState::Held {
                // Transition to Falling state
                *spawn_state = FruitSpawnState::Falling;

                let params = fruit_type.parameters();

                // Convert to dynamic rigid body with physics properties
                commands.entity(entity).insert((
                    RigidBody::Dynamic,
                    Restitution::coefficient(params.restitution),
                    Friction::coefficient(params.friction),
                    ColliderMassProperties::Mass(params.mass),
                    Damping {
                        linear_damping: 0.5,
                        angular_damping: 1.0,
                    },
                    GravityScale(1.0),
                ));

                info!("Dropped fruit: {:?}", fruit_type);

                // Randomize next fruit type for the next spawn
                next_fruit.randomize();
            }
        }
    }
}

/// Updates the spawn position and held fruit position based on player input
///
/// Updates spawn position from multiple input sources:
/// 1. Arrow keys: Left/Right (←→) or A/D keys move horizontally
/// 2. Mouse cursor: Position follows the mouse X coordinate
///
/// The held fruit (if any) is moved to match the spawn position in real-time.
/// The final position is clamped to stay within container boundaries.
///
/// # System Parameters
///
/// - `keyboard`: Keyboard input state
/// - `windows`: Query for the primary window (to get cursor position)
/// - `camera_query`: Query for camera and its transform (for world position conversion)
/// - `spawn_pos`: Mutable spawn position resource to update
/// - `held_fruits`: Query for held fruits to move
/// - `time`: Time resource for delta time (smooth movement with keys)
pub fn update_spawn_position(
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut spawn_pos: ResMut<SpawnPosition>,
    mut held_fruits: Query<&mut Transform, (With<Fruit>, With<FruitSpawnState>)>,
    time: Res<Time>,
) {
    // Handle keyboard movement (Arrow keys or A/D keys)
    const MOVE_SPEED: f32 = 300.0; // pixels per second
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        spawn_pos.x -= MOVE_SPEED * time.delta_secs();
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
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

    // Update held fruit position to match spawn position
    for mut transform in held_fruits.iter_mut() {
        transform.translation.x = spawn_pos.x;
    }
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
    fn test_spawn_held_fruit_creates_fruit() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SpawnPosition>();
        app.init_resource::<NextFruitType>();
        app.add_systems(Update, spawn_held_fruit);

        // Initial fruit count
        let initial_count = app
            .world()
            .query_filtered::<Entity, With<Fruit>>()
            .iter(app.world())
            .count();

        app.update();

        // Should have spawned one held fruit
        let final_count = app
            .world()
            .query_filtered::<Entity, With<Fruit>>()
            .iter(app.world())
            .count();

        assert_eq!(final_count, initial_count + 1, "Should spawn one held fruit");
    }

    #[test]
    fn test_spawn_held_fruit_only_one_at_a_time() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SpawnPosition>();
        app.init_resource::<NextFruitType>();
        app.add_systems(Update, spawn_held_fruit);

        app.update();
        let count_after_first = app
            .world()
            .query_filtered::<Entity, With<Fruit>>()
            .iter(app.world())
            .count();

        app.update();
        let count_after_second = app
            .world()
            .query_filtered::<Entity, With<Fruit>>()
            .iter(app.world())
            .count();

        assert_eq!(
            count_after_first, count_after_second,
            "Should not spawn multiple held fruits"
        );
    }

    #[test]
    fn test_handle_fruit_drop_input_space_key() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SpawnPosition>();
        app.init_resource::<NextFruitType>();
        app.add_systems(Update, (spawn_held_fruit, handle_fruit_drop_input));

        // Spawn a held fruit first
        app.update();

        // Simulate space key press
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);

        app.update();

        // Verify fruit transitioned to Falling state
        let falling_count = app
            .world()
            .query_filtered::<&FruitSpawnState, With<Fruit>>()
            .iter(app.world())
            .filter(|state| **state == FruitSpawnState::Falling)
            .count();

        assert_eq!(falling_count, 1, "Space key should drop the held fruit");
    }

    #[test]
    fn test_handle_fruit_drop_input_mouse_click() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SpawnPosition>();
        app.init_resource::<NextFruitType>();
        app.add_systems(Update, (spawn_held_fruit, handle_fruit_drop_input));

        app.update();

        // Simulate mouse click
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);

        app.update();

        let falling_count = app
            .world()
            .query_filtered::<&FruitSpawnState, With<Fruit>>()
            .iter(app.world())
            .filter(|state| **state == FruitSpawnState::Falling)
            .count();

        assert_eq!(falling_count, 1, "Mouse click should drop the held fruit");
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
    fn test_update_spawn_position_ad_keys() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpawnPosition { x: 0.0 });
        app.add_systems(Update, update_spawn_position);

        // Simulate D key press
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);

        app.update();

        let pos = app.world().resource::<SpawnPosition>();
        assert!(pos.x > 0.0, "D key should move position to the right");
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

    #[test]
    fn test_update_spawn_position_moves_held_fruit() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SpawnPosition>();
        app.init_resource::<NextFruitType>();
        app.add_systems(Update, (spawn_held_fruit, update_spawn_position));

        // Spawn held fruit
        app.update();

        // Move spawn position
        app.world_mut().resource_mut::<SpawnPosition>().x = 100.0;

        app.update();

        // Verify held fruit moved
        let fruit_x = app
            .world()
            .query_filtered::<&Transform, (With<Fruit>, With<FruitSpawnState>)>()
            .iter(app.world())
            .next()
            .map(|t| t.translation.x);

        assert_eq!(fruit_x, Some(100.0), "Held fruit should move with spawn position");
    }
}
