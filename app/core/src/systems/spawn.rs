//! Fruit spawning system
//!
//! This module handles spawning fruits into the game world with appropriate
//! physics bodies, colliders, and visual representation.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_rapier2d::prelude::*;

use crate::components::Fruit;
use crate::config::FruitsConfig;
use crate::fruit::FruitType;
use crate::resources::CircleTexture;

/// Generates a white circle image and stores it as [`CircleTexture`].
///
/// Creates a 128×128 RGBA image where every pixel inside the disc is opaque
/// white and every pixel outside is fully transparent.  Fruit sprites tint
/// this texture with `Sprite::color` to achieve their individual colours.
///
/// Run this system once at `Startup` (before any fruit is spawned).
pub(crate) fn setup_circle_texture(mut circle_texture: ResMut<CircleTexture>, mut images: ResMut<Assets<Image>>) {
    const DIAMETER: u32 = 128;
    const RADIUS: f32 = 64.0;

    let mut data = vec![0u8; (DIAMETER * DIAMETER * 4) as usize];
    for y in 0..DIAMETER {
        for x in 0..DIAMETER {
            let dx = x as f32 + 0.5 - RADIUS;
            let dy = y as f32 + 0.5 - RADIUS;
            let alpha = if dx * dx + dy * dy <= RADIUS * RADIUS {
                255u8
            } else {
                0u8
            };
            let idx = ((y * DIAMETER + x) * 4) as usize;
            data[idx] = 255; // R — white
            data[idx + 1] = 255; // G — white
            data[idx + 2] = 255; // B — white
            data[idx + 3] = alpha; // A — disc shape
        }
    }

    let image = Image::new(
        Extent3d {
            width: DIAMETER,
            height: DIAMETER,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    circle_texture.0 = images.add(image);
}

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
/// * `config` - Reference to the fruits configuration (for parameters)
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
/// # use suika_game_core::config::{FruitsConfig, FruitsConfigHandle};
/// # use suika_game_core::prelude::CircleTexture;
/// fn spawn_system(
///     mut commands: Commands,
///     fruits_handle: Res<FruitsConfigHandle>,
///     fruits_assets: Res<Assets<FruitsConfig>>,
///     circle_texture: Res<CircleTexture>,
/// ) {
///     if let Some(config) = fruits_assets.get(&fruits_handle.0) {
///         let fruit_entity = spawn_fruit(
///             &mut commands,
///             FruitType::Cherry,
///             Vec2::new(0.0, 300.0),
///             config,
///             circle_texture.0.clone(),
///         );
///         info!("Spawned fruit with ID: {:?}", fruit_entity);
///     }
/// }
/// ```
pub fn spawn_fruit(
    commands: &mut Commands,
    fruit_type: FruitType,
    position: Vec2,
    config: &FruitsConfig,
    circle_image: Handle<Image>,
) -> Entity {
    let params = fruit_type.parameters_from_config(config);

    commands
        .spawn((
            // Fruit marker component
            Fruit,
            // Sprite rendering — circular placeholder tinted with the fruit colour.
            // When pixel-art sprites are ready, replace `image` with the real handle
            // and set `color` to `Color::WHITE`.
            Sprite {
                image: circle_image,
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
    use crate::config::FruitConfigEntry;

    // Helper function to create a test config
    fn create_test_config() -> FruitsConfig {
        FruitsConfig {
            fruits: vec![
                FruitConfigEntry {
                    name: "Cherry".to_string(),
                    radius: 20.0,
                    points: 10,
                    restitution: 0.3,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                },
                FruitConfigEntry {
                    name: "Strawberry".to_string(),
                    radius: 30.0,
                    points: 20,
                    restitution: 0.3,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                },
                FruitConfigEntry {
                    name: "Grape".to_string(),
                    radius: 40.0,
                    points: 40,
                    restitution: 0.3,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                },
                FruitConfigEntry {
                    name: "Dekopon".to_string(),
                    radius: 50.0,
                    points: 80,
                    restitution: 0.25,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                },
                FruitConfigEntry {
                    name: "Persimmon".to_string(),
                    radius: 60.0,
                    points: 160,
                    restitution: 0.25,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                },
                FruitConfigEntry {
                    name: "Apple".to_string(),
                    radius: 70.0,
                    points: 320,
                    restitution: 0.25,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                },
                FruitConfigEntry {
                    name: "Pear".to_string(),
                    radius: 80.0,
                    points: 640,
                    restitution: 0.25,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                },
                FruitConfigEntry {
                    name: "Peach".to_string(),
                    radius: 90.0,
                    points: 1280,
                    restitution: 0.2,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                },
                FruitConfigEntry {
                    name: "Pineapple".to_string(),
                    radius: 100.0,
                    points: 2560,
                    restitution: 0.2,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                },
                FruitConfigEntry {
                    name: "Melon".to_string(),
                    radius: 110.0,
                    points: 5120,
                    restitution: 0.2,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                },
                FruitConfigEntry {
                    name: "Watermelon".to_string(),
                    radius: 120.0,
                    points: 10240,
                    restitution: 0.2,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                },
            ],
        }
    }

    #[test]
    fn test_spawn_fruit_creates_entity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let config = create_test_config();

        let mut commands = app.world_mut().commands();
        let entity = spawn_fruit(
            &mut commands,
            FruitType::Cherry,
            Vec2::new(0.0, 100.0),
            &config,
            Handle::default(),
        );

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
        let config = create_test_config();

        let mut commands = app.world_mut().commands();
        let entity = spawn_fruit(
            &mut commands,
            FruitType::Strawberry,
            Vec2::new(10.0, 20.0),
            &config,
            Handle::default(),
        );

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
        let config = create_test_config();

        let mut commands = app.world_mut().commands();
        let position = Vec2::new(50.0, 150.0);
        let entity = spawn_fruit(
            &mut commands,
            FruitType::Grape,
            position,
            &config,
            Handle::default(),
        );

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
        let config = create_test_config();

        let mut commands = app.world_mut().commands();
        let fruit_type = FruitType::Apple;
        let entity = spawn_fruit(
            &mut commands,
            fruit_type,
            Vec2::new(0.0, 0.0),
            &config,
            Handle::default(),
        );

        app.update();

        // Verify Sprite component exists
        let sprite = app.world().get::<Sprite>(entity);
        assert!(
            sprite.is_some(),
            "Spawned entity should have Sprite component"
        );

        // Verify sprite has correct size (diameter = radius * 2)
        let sprite = sprite.unwrap();
        let params = fruit_type.parameters_from_config(&config);
        let expected_size = Vec2::splat(params.radius * 2.0);
        assert_eq!(sprite.custom_size, Some(expected_size));
    }

    #[test]
    fn test_spawn_fruit_has_physics_components() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let config = create_test_config();

        let mut commands = app.world_mut().commands();
        let entity = spawn_fruit(
            &mut commands,
            FruitType::Peach,
            Vec2::new(0.0, 0.0),
            &config,
            Handle::default(),
        );

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
        let config = create_test_config();

        let mut commands = app.world_mut().commands();
        let entity = spawn_fruit(
            &mut commands,
            FruitType::Pineapple,
            Vec2::new(0.0, 0.0),
            &config,
            Handle::default(),
        );

        app.update();

        // Verify RigidBody is Dynamic
        let rigid_body = app.world().get::<RigidBody>(entity).unwrap();
        assert_eq!(*rigid_body, RigidBody::Dynamic);
    }

    #[test]
    fn test_spawn_different_fruit_types() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let config = create_test_config();

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
            let entity = spawn_fruit(
                &mut commands,
                fruit_type,
                Vec2::new(0.0, 0.0),
                &config,
                Handle::default(),
            );
            app.update();

            assert!(
                app.world().get::<Fruit>(entity).is_some(),
                "Should spawn {} successfully",
                format!("{:?}", fruit_type)
            );
        }
    }
}
