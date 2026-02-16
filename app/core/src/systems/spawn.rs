//! Fruit spawning system
//!
//! This module handles spawning fruits into the game world with appropriate
//! physics bodies, colliders, and visual representation.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::components::Fruit;
use crate::fruit::FruitType;

/// Spawns a fruit entity at the specified position
///
/// Creates a complete fruit entity with all necessary components:
/// - Physics body (RigidBody::Dynamic)
/// - Collision shape (Collider::ball)
/// - Physical properties (mass, restitution, friction, damping)
/// - Visual representation (placeholder sprite circle)
/// - Fruit marker component
///
/// # Arguments
///
/// * `commands` - Mutable reference to Bevy Commands for entity spawning
/// * `fruit_type` - The type of fruit to spawn (determines size, color, physics)
/// * `position` - 2D position (x, y) where the fruit should be spawned
///
/// # Returns
///
/// Returns the Entity ID of the spawned fruit
///
/// # Example
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy::math::Vec2;
/// # use suika_game_core::systems::spawn::spawn_fruit;
/// # use suika_game_core::fruit::FruitType;
/// fn spawn_system(mut commands: Commands) {
///     let fruit_entity = spawn_fruit(
///         &mut commands,
///         FruitType::Cherry,
///         Vec2::new(0.0, 300.0),
///     );
///     info!("Spawned fruit with ID: {:?}", fruit_entity);
/// }
/// ```
pub fn spawn_fruit(commands: &mut Commands, fruit_type: FruitType, position: Vec2) -> Entity {
    let params = fruit_type.parameters();

    commands
        .spawn((
            // Fruit marker component
            Fruit,
            // Sprite rendering (placeholder - solid color circle)
            Sprite {
                color: fruit_type.placeholder_color(),
                custom_size: Some(Vec2::splat(params.radius * 2.0)),
                ..default()
            },
            // Transform (position, rotation, scale)
            Transform::from_xyz(position.x, position.y, 0.0),
            // Physics: Dynamic rigid body (affected by gravity and forces)
            RigidBody::Dynamic,
            // Collision shape: Ball with fruit's radius
            Collider::ball(params.radius),
            // Restitution: How bouncy the fruit is (0.0 = no bounce, 1.0 = perfect bounce)
            Restitution::coefficient(params.restitution),
            // Friction: Surface friction coefficient
            Friction::coefficient(params.friction),
            // Mass: Physical mass of the fruit
            ColliderMassProperties::Mass(params.mass),
            // Damping: Reduces linear and angular velocity over time
            Damping {
                linear_damping: 0.5,  // Reduces linear velocity
                angular_damping: 1.0, // Reduces rotation
            },
            // Gravity scale: 1.0 = normal gravity, 0.0 = no gravity
            GravityScale(1.0),
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_fruit_creates_entity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut commands = app.world_mut().commands();
        let entity = spawn_fruit(&mut commands, FruitType::Cherry, Vec2::new(0.0, 100.0));

        // Flush commands to apply them
        app.update();

        // Verify entity exists
        assert!(
            app.world().get_entity(entity).is_ok(),
            "Spawned entity should exist"
        );
    }

    #[test]
    fn test_spawn_fruit_has_fruit_component() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut commands = app.world_mut().commands();
        let entity = spawn_fruit(&mut commands, FruitType::Strawberry, Vec2::new(10.0, 20.0));

        app.update();

        // Verify Fruit component exists
        assert!(
            app.world().get::<Fruit>(entity).is_some(),
            "Spawned entity should have Fruit component"
        );
    }

    #[test]
    fn test_spawn_fruit_has_transform() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut commands = app.world_mut().commands();
        let position = Vec2::new(50.0, 150.0);
        let entity = spawn_fruit(&mut commands, FruitType::Grape, position);

        app.update();

        // Verify Transform component exists and has correct position
        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_eq!(transform.translation.x, position.x);
        assert_eq!(transform.translation.y, position.y);
        assert_eq!(transform.translation.z, 0.0);
    }

    #[test]
    fn test_spawn_fruit_has_sprite() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut commands = app.world_mut().commands();
        let fruit_type = FruitType::Apple;
        let entity = spawn_fruit(&mut commands, fruit_type, Vec2::new(0.0, 0.0));

        app.update();

        // Verify Sprite component exists
        let sprite = app.world().get::<Sprite>(entity);
        assert!(
            sprite.is_some(),
            "Spawned entity should have Sprite component"
        );

        // Verify sprite has correct size (diameter = radius * 2)
        let sprite = sprite.unwrap();
        let params = fruit_type.parameters();
        let expected_size = Vec2::splat(params.radius * 2.0);
        assert_eq!(sprite.custom_size, Some(expected_size));
    }

    #[test]
    fn test_spawn_fruit_has_physics_components() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut commands = app.world_mut().commands();
        let entity = spawn_fruit(&mut commands, FruitType::Peach, Vec2::new(0.0, 0.0));

        app.update();

        // Verify physics components exist
        assert!(
            app.world().get::<RigidBody>(entity).is_some(),
            "Should have RigidBody component"
        );
        assert!(
            app.world().get::<Collider>(entity).is_some(),
            "Should have Collider component"
        );
        assert!(
            app.world().get::<Restitution>(entity).is_some(),
            "Should have Restitution component"
        );
        assert!(
            app.world().get::<Friction>(entity).is_some(),
            "Should have Friction component"
        );
        assert!(
            app.world().get::<GravityScale>(entity).is_some(),
            "Should have GravityScale component"
        );
        assert!(
            app.world().get::<Damping>(entity).is_some(),
            "Should have Damping component"
        );
    }

    #[test]
    fn test_spawn_fruit_rigid_body_is_dynamic() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut commands = app.world_mut().commands();
        let entity = spawn_fruit(&mut commands, FruitType::Pineapple, Vec2::new(0.0, 0.0));

        app.update();

        // Verify RigidBody is Dynamic
        let rigid_body = app.world().get::<RigidBody>(entity).unwrap();
        assert_eq!(*rigid_body, RigidBody::Dynamic);
    }

    #[test]
    fn test_spawn_different_fruit_types() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Test spawning all fruit types
        let fruit_types = [
            FruitType::Cherry,
            FruitType::Strawberry,
            FruitType::Grape,
            FruitType::Dekopon,
            FruitType::Persimmon,
            FruitType::Apple,
            FruitType::Pear,
            FruitType::Peach,
            FruitType::Pineapple,
            FruitType::Melon,
            FruitType::Watermelon,
        ];

        for fruit_type in fruit_types {
            let mut commands = app.world_mut().commands();
            let entity = spawn_fruit(&mut commands, fruit_type, Vec2::new(0.0, 0.0));
            app.update();

            assert!(
                app.world().get::<Fruit>(entity).is_some(),
                "Should spawn {} successfully",
                format!("{:?}", fruit_type)
            );
        }
    }
}
