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

#[cfg(test)]
use bevy_rapier2d::rapier::geometry::CollisionEventFlags;

use crate::components::{BottomWall, Fruit, FruitSpawnState};
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

/// Input mode for controlling fruit position
///
/// Tracks whether the player is currently using keyboard or mouse input.
/// The mode automatically switches based on which input device was used most recently.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Player is using keyboard (arrow keys or A/D)
    Keyboard,
    /// Player is using mouse cursor
    Mouse,
}

impl Default for InputMode {
    fn default() -> Self {
        Self::Mouse // Default to mouse mode
    }
}

/// Tracks the last known cursor position to detect mouse movement
///
/// Used to distinguish between actual mouse movement and position changes
/// caused by keyboard input. Only switches to mouse mode when the cursor
/// itself moves.
#[derive(Resource, Debug, Clone)]
pub struct LastCursorPosition {
    /// Last known cursor position in world coordinates
    pub position: Option<Vec2>,
}

impl Default for LastCursorPosition {
    fn default() -> Self {
        Self { position: None }
    }
}

/// Spawns a new held fruit if none exists
///
/// This system runs once at startup and after each fruit lands.
/// It creates a fruit in the Held state that hovers at the top of the container.
///
/// **Important**: Will NOT spawn if there's a falling fruit (waiting for it to land first).
///
/// # System Parameters
///
/// - `commands`: For spawning new fruit entities
/// - `next_fruit`: The type of fruit to spawn
/// - `spawn_pos`: Current spawn position (X coordinate)
/// - `fruit_states`: Query to check fruit spawn states
pub fn spawn_held_fruit(
    mut commands: Commands,
    next_fruit: Res<NextFruitType>,
    spawn_pos: Res<SpawnPosition>,
    fruit_states: Query<&FruitSpawnState, With<Fruit>>,
) {
    // Check if there's a held or falling fruit
    let has_held_fruit = fruit_states
        .iter()
        .any(|state| *state == FruitSpawnState::Held);

    let has_falling_fruit = fruit_states
        .iter()
        .any(|state| *state == FruitSpawnState::Falling);

    // Debug: Count fruits by state
    let held_count = fruit_states
        .iter()
        .filter(|s| **s == FruitSpawnState::Held)
        .count();
    let falling_count = fruit_states
        .iter()
        .filter(|s| **s == FruitSpawnState::Falling)
        .count();
    let landed_count = fruit_states
        .iter()
        .filter(|s| **s == FruitSpawnState::Landed)
        .count();

    if held_count > 0 || falling_count > 0 || landed_count > 0 {
        info!(
            "Fruit states - Held: {}, Falling: {}, Landed: {}",
            held_count, falling_count, landed_count
        );
    }

    // Only spawn if:
    // 1. No fruit in Held state
    // 2. No fruit in Falling state (wait for it to land first)
    if !has_held_fruit && !has_falling_fruit {
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
            // Enable collision events (required for Rapier)
            ActiveEvents::COLLISION_EVENTS,
            // Disable sleeping to allow continuous physics interactions
            Sleeping::disabled(),
        ));

        info!("Spawned held fruit: {:?}", next_fruit.get());
    }
}

/// Detects when falling fruits land (collide with ground or other fruits)
///
/// Monitors collision events and transitions falling fruits to Landed state
/// when they collide with the bottom wall (ground) or other fruits.
/// Side walls are ignored - only ground collisions count as landing.
/// This triggers the spawning of the next fruit.
///
/// # System Parameters
///
/// - `collision_events`: Rapier collision message reader
/// - `fruit_query`: Query for fruits and their spawn state
/// - `bottom_wall_query`: Query for bottom wall entity (ground)
pub fn detect_fruit_landing(
    mut collision_events: MessageReader<CollisionEvent>,
    mut fruit_query: Query<&mut FruitSpawnState, With<Fruit>>,
    bottom_wall_query: Query<Entity, With<BottomWall>>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = event {
            // Collect entities to update (to avoid borrow checker issues)
            let mut entities_to_land = Vec::new();

            // Check if entity1 is a falling fruit
            if let Ok(spawn_state) = fruit_query.get(*entity1) {
                if *spawn_state == FruitSpawnState::Falling {
                    let hit_bottom_wall = bottom_wall_query.contains(*entity2);
                    let hit_fruit = fruit_query.contains(*entity2);

                    if hit_bottom_wall || hit_fruit {
                        entities_to_land.push((*entity1, hit_bottom_wall));
                    }
                }
            }

            // Check if entity2 is a falling fruit
            if let Ok(spawn_state) = fruit_query.get(*entity2) {
                if *spawn_state == FruitSpawnState::Falling {
                    let hit_bottom_wall = bottom_wall_query.contains(*entity1);
                    let hit_fruit = fruit_query.contains(*entity1);

                    if hit_bottom_wall || hit_fruit {
                        entities_to_land.push((*entity2, hit_bottom_wall));
                    }
                }
            }

            // Now update the states
            for (entity, hit_bottom_wall) in entities_to_land {
                if let Ok(mut spawn_state) = fruit_query.get_mut(entity) {
                    *spawn_state = FruitSpawnState::Landed;
                    info!(
                        "Fruit landed (collided with {})",
                        if hit_bottom_wall { "ground" } else { "fruit" }
                    );
                }
            }
        }
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
                // Reset velocity to prevent diagonal falling due to kinematic movement
                commands.entity(entity).insert((
                    RigidBody::Dynamic,
                    Velocity::zero(), // Reset velocity to drop straight down
                    Restitution::coefficient(0.0), // No bounce
                    Friction::coefficient(params.friction),
                    ColliderMassProperties::Mass(params.mass),
                    Damping {
                        linear_damping: 0.5,
                        angular_damping: 0.1,
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
/// The input mode automatically switches based on which input device was used most recently:
/// - Pressing arrow/AD keys switches to keyboard mode
/// - Moving the mouse cursor switches to mouse mode
///
/// Only fruits in the Held state are moved. Falling and Landed fruits are not affected.
/// The final position is clamped to stay within container boundaries.
///
/// # System Parameters
///
/// - `keyboard`: Keyboard input state
/// - `windows`: Query for the primary window (to get cursor position)
/// - `camera_query`: Query for camera and its transform (for world position conversion)
/// - `spawn_pos`: Mutable spawn position resource to update
/// - `input_mode`: Current input mode (keyboard or mouse)
/// - `held_fruits`: Query for held fruits to move (only Held state)
/// - `time`: Time resource for delta time (smooth movement with keys)
pub fn update_spawn_position(
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut spawn_pos: ResMut<SpawnPosition>,
    mut input_mode: ResMut<InputMode>,
    mut last_cursor_pos: ResMut<LastCursorPosition>,
    mut held_fruits: Query<(&mut Transform, &FruitSpawnState, &FruitType), With<Fruit>>,
    time: Res<Time>,
) {
    // Check for keyboard input and switch mode if detected
    let keyboard_input = keyboard.pressed(KeyCode::ArrowLeft)
        || keyboard.pressed(KeyCode::KeyA)
        || keyboard.pressed(KeyCode::ArrowRight)
        || keyboard.pressed(KeyCode::KeyD);

    if keyboard_input {
        *input_mode = InputMode::Keyboard;
    }

    // Handle keyboard movement (only in keyboard mode)
    if *input_mode == InputMode::Keyboard {
        const MOVE_SPEED: f32 = 300.0; // pixels per second
        if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
            spawn_pos.x -= MOVE_SPEED * time.delta_secs();
        }
        if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
            spawn_pos.x += MOVE_SPEED * time.delta_secs();
        }
    }

    // Check for mouse movement and switch mode if detected
    if let Ok(window) = windows.single() {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok((camera, camera_transform)) = camera_query.single() {
                // Convert viewport coordinates to world coordinates
                if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                    // Detect actual mouse movement by comparing with last cursor position
                    const MOUSE_MOVEMENT_THRESHOLD: f32 = 1.0; // pixels
                    let mouse_moved = if let Some(last_pos) = last_cursor_pos.position {
                        (world_pos - last_pos).length() > MOUSE_MOVEMENT_THRESHOLD
                    } else {
                        false // First frame, don't switch to mouse mode yet
                    };

                    if mouse_moved {
                        *input_mode = InputMode::Mouse;
                    }

                    // Update last cursor position
                    last_cursor_pos.position = Some(world_pos);

                    // Handle mouse cursor position (only in mouse mode)
                    if *input_mode == InputMode::Mouse {
                        spawn_pos.x = world_pos.x;
                    }
                }
            }
        }
    }

    // Get the held fruit's radius for proper clamping
    let held_fruit_radius = held_fruits
        .iter()
        .find(|(_, state, _)| **state == FruitSpawnState::Held)
        .map(|(_, _, fruit_type)| fruit_type.parameters().radius)
        .unwrap_or(20.0); // Default to smallest fruit if none found

    // Clamp spawn position within container bounds
    // Use the actual fruit radius to allow the fruit to touch the wall
    let max_x = physics::CONTAINER_WIDTH / 2.0 - held_fruit_radius;
    spawn_pos.x = spawn_pos.x.clamp(-max_x, max_x);

    // Update ONLY held fruit position to match spawn position
    // Falling and Landed fruits are not affected
    for (mut transform, spawn_state, _) in held_fruits.iter_mut() {
        if *spawn_state == FruitSpawnState::Held {
            transform.translation.x = spawn_pos.x;
        }
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
        // Use default fruit radius (20.0) to match system behavior
        let max_x = physics::CONTAINER_WIDTH / 2.0 - 20.0;

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
            .world_mut()
            .query_filtered::<Entity, With<Fruit>>()
            .iter(app.world())
            .count();

        app.update();

        // Should have spawned one held fruit
        let final_count = app
            .world_mut()
            .query_filtered::<Entity, With<Fruit>>()
            .iter(app.world())
            .count();

        assert_eq!(
            final_count,
            initial_count + 1,
            "Should spawn one held fruit"
        );
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
            .world_mut()
            .query_filtered::<Entity, With<Fruit>>()
            .iter(app.world())
            .count();

        app.update();
        let count_after_second = app
            .world_mut()
            .query_filtered::<Entity, With<Fruit>>()
            .iter(app.world())
            .count();

        assert_eq!(
            count_after_first, count_after_second,
            "Should not spawn multiple held fruits"
        );
    }

    #[test]
    fn test_spawn_held_fruit_waits_for_falling_fruit() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SpawnPosition>();
        app.init_resource::<NextFruitType>();
        app.add_systems(Update, spawn_held_fruit);

        // Manually spawn a falling fruit
        app.world_mut().spawn((
            Fruit,
            FruitType::Cherry,
            FruitSpawnState::Falling,
            Transform::default(),
        ));

        app.update();

        // Should NOT spawn a new held fruit while one is falling
        let held_count = app
            .world_mut()
            .query_filtered::<&FruitSpawnState, With<Fruit>>()
            .iter(app.world())
            .filter(|s| **s == FruitSpawnState::Held)
            .count();

        assert_eq!(
            held_count, 0,
            "Should not spawn held fruit while another is falling"
        );
    }

    #[test]
    fn test_handle_fruit_drop_input_space_key() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SpawnPosition>();
        app.init_resource::<NextFruitType>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<ButtonInput<MouseButton>>();
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
            .world_mut()
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
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<ButtonInput<MouseButton>>();
        app.add_systems(Update, (spawn_held_fruit, handle_fruit_drop_input));

        app.update();

        // Simulate mouse click
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);

        app.update();

        let falling_count = app
            .world_mut()
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
        app.init_resource::<InputMode>();
        app.init_resource::<LastCursorPosition>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, update_spawn_position);

        // Simulate arrow right press
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowRight);

        // Run update twice to ensure non-zero delta time
        app.update();
        app.update();

        let pos = app.world().resource::<SpawnPosition>();
        assert!(pos.x > 0.0, "Arrow right should move position to the right");
    }

    #[test]
    fn test_update_spawn_position_ad_keys() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SpawnPosition { x: 0.0 });
        app.init_resource::<InputMode>();
        app.init_resource::<LastCursorPosition>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, update_spawn_position);

        // Simulate D key press
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);

        // Run update twice to ensure non-zero delta time
        app.update();
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
        app.init_resource::<InputMode>();
        app.init_resource::<LastCursorPosition>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, update_spawn_position);

        app.update();

        let pos = app.world().resource::<SpawnPosition>();
        // Use default fruit radius (20.0) to match system behavior
        let max_x = physics::CONTAINER_WIDTH / 2.0 - 20.0;

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
        app.init_resource::<InputMode>();
        app.init_resource::<LastCursorPosition>();
        app.init_resource::<NextFruitType>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, (spawn_held_fruit, update_spawn_position));

        // Spawn held fruit
        app.update();

        // Move spawn position manually (simulating external input)
        app.world_mut().resource_mut::<SpawnPosition>().x = 100.0;

        app.update();

        // Verify held fruit moved
        let fruit_x = app
            .world_mut()
            .query_filtered::<(&Transform, &FruitSpawnState), With<Fruit>>()
            .iter(app.world())
            .filter(|(_, state)| **state == FruitSpawnState::Held)
            .next()
            .map(|(t, _)| t.translation.x);

        assert_eq!(
            fruit_x,
            Some(100.0),
            "Held fruit should move with spawn position"
        );
    }

    #[test]
    fn test_detect_fruit_landing_transitions_to_landed() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<CollisionEvent>();
        app.init_resource::<SpawnPosition>();
        app.init_resource::<NextFruitType>();
        app.add_systems(Update, detect_fruit_landing);

        // Manually spawn a falling fruit
        let fruit = app
            .world_mut()
            .spawn((
                Fruit,
                FruitType::Cherry,
                FruitSpawnState::Falling,
                Transform::default(),
            ))
            .id();

        // Spawn a bottom wall (ground)
        let bottom_wall = app.world_mut().spawn(BottomWall).id();

        // Simulate collision event (bevy_rapier2d 0.32 API)
        app.world_mut().write_message(CollisionEvent::Started(
            fruit,
            bottom_wall,
            CollisionEventFlags::empty(),
        ));

        app.update();

        // Verify fruit transitioned to Landed
        let state = app.world().get::<FruitSpawnState>(fruit).unwrap();
        assert_eq!(
            *state,
            FruitSpawnState::Landed,
            "Fruit should transition to Landed after collision with ground"
        );
    }

    #[test]
    fn test_spawn_held_fruit_after_landing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<CollisionEvent>();
        app.init_resource::<SpawnPosition>();
        app.init_resource::<NextFruitType>();
        app.add_systems(Update, (detect_fruit_landing, spawn_held_fruit));

        // Spawn initial held fruit
        app.update();

        // Manually transition to falling then landed
        let fruit_entity = app
            .world_mut()
            .query_filtered::<Entity, With<Fruit>>()
            .iter(app.world())
            .next()
            .unwrap();

        app.world_mut()
            .entity_mut(fruit_entity)
            .insert(FruitSpawnState::Landed);

        app.update();

        // Should spawn a new held fruit
        let final_count = app
            .world_mut()
            .query_filtered::<&FruitSpawnState, With<Fruit>>()
            .iter(app.world())
            .filter(|s| **s == FruitSpawnState::Held)
            .count();

        // After landing, a new held fruit should be spawned (replacing behavior)
        // initial_count was 1 (the first held fruit)
        // After setting it to Landed and updating, a new Held fruit spawns
        // So final_count should still be 1 (one Held fruit exists)
        assert_eq!(
            final_count, 1,
            "Should have one held fruit after previous one lands"
        );

        // Verify total fruit count increased (landed + new held = 2 total)
        let total_fruit_count = app
            .world_mut()
            .query_filtered::<Entity, With<Fruit>>()
            .iter(app.world())
            .count();
        assert_eq!(
            total_fruit_count, 2,
            "Should have 2 total fruits (1 landed + 1 new held)"
        );
    }

    #[test]
    fn test_update_spawn_position_does_not_move_falling_fruit() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SpawnPosition>();
        app.init_resource::<InputMode>();
        app.init_resource::<LastCursorPosition>();
        app.init_resource::<NextFruitType>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<ButtonInput<MouseButton>>();
        app.add_systems(
            Update,
            (
                spawn_held_fruit,
                handle_fruit_drop_input,
                update_spawn_position,
            ),
        );

        // Spawn and drop a fruit
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        app.update();

        // Get falling fruit position
        let initial_x = app
            .world_mut()
            .query_filtered::<(&Transform, &FruitSpawnState), With<Fruit>>()
            .iter(app.world())
            .filter(|(_, state)| **state == FruitSpawnState::Falling)
            .next()
            .map(|(t, _)| t.translation.x);

        // Change spawn position
        app.world_mut().resource_mut::<SpawnPosition>().x = 200.0;
        app.update();

        // Verify falling fruit did NOT move
        let final_x = app
            .world_mut()
            .query_filtered::<(&Transform, &FruitSpawnState), With<Fruit>>()
            .iter(app.world())
            .filter(|(_, state)| **state == FruitSpawnState::Falling)
            .next()
            .map(|(t, _)| t.translation.x);

        assert_eq!(
            initial_x, final_x,
            "Falling fruit should not move with spawn position"
        );
    }
}
