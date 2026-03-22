//! Player input handling system
//!
//! This module handles player input for fruit control, including:
//! - Spawning a held fruit at the start
//! - Mouse position and arrow keys (←→ or A/D) for position control
//! - Space key or mouse click to drop the fruit
//! - Automatic spawning of next fruit after drop

pub mod drop;
pub mod movement;
pub mod plugin;
pub mod resources;
pub mod spawn;

pub use drop::{detect_fruit_landing, handle_fruit_drop_input};
pub use movement::update_spawn_position;
pub use plugin::InputPlugin;
pub use resources::{InputMode, LastCursorPosition, SpawnPosition};
pub use spawn::spawn_held_fruit;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{BottomWall, Fruit, FruitSpawnState};
    use crate::config::*;
    use crate::fruit::FruitType;
    use crate::resources::{CircleTexture, NextFruitType};
    use bevy::asset::Assets;
    use bevy::prelude::*;
    use bevy_rapier2d::prelude::CollisionEvent;

    #[cfg(test)]
    use bevy_rapier2d::rapier::geometry::CollisionEventFlags;

    // These match the constants in movement.rs
    const DEFAULT_CONTAINER_WIDTH: f32 = 600.0;
    const DEFAULT_FRUIT_RADIUS: f32 = 20.0;

    /// Helper to setup test app with required resources
    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Create and add config assets
        let mut fruits_assets = Assets::<FruitsConfig>::default();
        let fruits_config = create_test_fruits_config();
        let fruits_handle = fruits_assets.add(fruits_config);

        let mut physics_assets = Assets::<PhysicsConfig>::default();
        let physics_config = create_test_physics_config();
        let physics_handle = physics_assets.add(physics_config);

        app.insert_resource(fruits_assets);
        app.insert_resource(FruitsConfigHandle(fruits_handle));
        app.insert_resource(physics_assets);
        app.insert_resource(PhysicsConfigHandle(physics_handle));
        app.init_resource::<SpawnPosition>();
        app.init_resource::<NextFruitType>();
        app.insert_resource(CircleTexture(Handle::default()));

        app
    }

    fn create_test_fruits_config() -> FruitsConfig {
        FruitsConfig {
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
        }
    }

    fn create_test_physics_config() -> PhysicsConfig {
        PhysicsConfig {
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
        }
    }

    #[test]
    fn test_spawn_position_default() {
        let pos = SpawnPosition::default();
        assert_eq!(pos.x, 0.0);
    }

    #[test]
    fn test_spawn_position_clamp() {
        // Use default fruit radius (20.0) to match system behavior
        let max_x = DEFAULT_CONTAINER_WIDTH / 2.0 - DEFAULT_FRUIT_RADIUS;

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
        let mut app = setup_test_app();
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
        let mut app = setup_test_app();
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
        let mut app = setup_test_app();
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
        let mut app = setup_test_app();
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
        let mut app = setup_test_app();
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
        let mut app = setup_test_app();
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
        let mut app = setup_test_app();
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
        let mut app = setup_test_app();

        // Start at extreme position
        app.insert_resource(SpawnPosition { x: 10000.0 });
        app.init_resource::<InputMode>();
        app.init_resource::<LastCursorPosition>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, update_spawn_position);

        app.update();

        let pos = app.world().resource::<SpawnPosition>();
        // Use default fruit radius (20.0) to match system behavior
        let max_x = DEFAULT_CONTAINER_WIDTH / 2.0 - DEFAULT_FRUIT_RADIUS;

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
        let mut app = setup_test_app();
        app.init_resource::<InputMode>();
        app.init_resource::<LastCursorPosition>();
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
        let mut app = setup_test_app();
        app.add_message::<CollisionEvent>();
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
        let mut app = setup_test_app();
        app.init_resource::<InputMode>();
        app.init_resource::<LastCursorPosition>();
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
