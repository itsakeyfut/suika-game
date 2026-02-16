//! Game Container (Box) Implementation
//!
//! This module handles the creation and setup of the game container,
//! which consists of three fixed walls: left, right, and bottom.
//! The walls use Rapier2D physics bodies to contain the fruits.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use suika_game_core::prelude::*;

/// Sets up the game container with physics walls
///
/// Creates three fixed walls:
/// - Left wall: positioned at the left edge of the container
/// - Right wall: positioned at the right edge of the container
/// - Bottom wall: positioned at the bottom of the container
///
/// Each wall is configured with:
/// - RigidBody::Fixed (immovable)
/// - Collider (box shape)
/// - Friction coefficient (0.5)
/// - Restitution coefficient (0.3)
/// - Visual sprite representation
///
/// # Container Dimensions
///
/// Loaded from `PhysicsConfig`:
/// - Width, height, wall thickness from physics.ron
/// - Wall restitution and friction from config
pub fn setup_container(
    mut commands: Commands,
    physics_handle: Res<PhysicsConfigHandle>,
    physics_assets: Res<Assets<PhysicsConfig>>,
) {
    // Get physics config, fallback to defaults if not loaded yet
    let (container_width, container_height, wall_thickness, wall_restitution, wall_friction) =
        if let Some(config) = physics_assets.get(&physics_handle.0) {
            (
                config.container_width,
                config.container_height,
                config.wall_thickness,
                config.wall_restitution,
                config.wall_friction,
            )
        } else {
            warn!("Physics config not loaded yet, using fallback values");
            (600.0, 800.0, 20.0, 0.2, 0.5)
        };

    // Calculate wall positions
    let half_width = container_width / 2.0;
    let half_height = container_height / 2.0;

    // Left wall
    commands.spawn((
        Container,
        RigidBody::Fixed,
        Collider::cuboid(wall_thickness / 2.0, half_height),
        Friction::coefficient(wall_friction),
        Restitution {
            coefficient: wall_restitution,
            combine_rule: CoefficientCombineRule::Min,
        },
        ActiveEvents::COLLISION_EVENTS,
        Transform::from_xyz(-half_width - wall_thickness / 2.0, 0.0, 0.0),
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(wall_thickness, container_height)),
            ..default()
        },
    ));

    // Right wall
    commands.spawn((
        Container,
        RigidBody::Fixed,
        Collider::cuboid(wall_thickness / 2.0, half_height),
        Friction::coefficient(wall_friction),
        Restitution {
            coefficient: wall_restitution,
            combine_rule: CoefficientCombineRule::Min,
        },
        ActiveEvents::COLLISION_EVENTS,
        Transform::from_xyz(half_width + wall_thickness / 2.0, 0.0, 0.0),
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(wall_thickness, container_height)),
            ..default()
        },
    ));

    // Bottom wall (no bounce - matches original Suika Game behavior)
    commands.spawn((
        Container,
        BottomWall, // Marker for landing detection
        RigidBody::Fixed,
        Collider::cuboid(half_width + wall_thickness, wall_thickness / 2.0),
        Friction::coefficient(wall_friction),
        Restitution {
            coefficient: 0.0, // No bounce on ground
            combine_rule: CoefficientCombineRule::Min,
        },
        ActiveEvents::COLLISION_EVENTS,
        Transform::from_xyz(0.0, -half_height - wall_thickness / 2.0, 0.0),
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(
                container_width + wall_thickness * 2.0,
                wall_thickness,
            )),
            ..default()
        },
    ));

    // Boundary line (game over line) - visual only, no physics
    let boundary_y = if let Some(config) = physics_assets.get(&physics_handle.0) {
        config.boundary_line_y
    } else {
        300.0 // Fallback
    };
    let line_thickness = 3.0;

    commands.spawn((
        BoundaryLine,
        Transform::from_xyz(0.0, boundary_y, 0.0),
        Sprite {
            color: Color::srgba(1.0, 0.0, 0.0, 0.5), // Red semi-transparent
            custom_size: Some(Vec2::new(container_width, line_thickness)),
            ..default()
        },
    ));

    info!("Game container initialized with 3 walls and boundary line");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a test app with necessary resources
    fn setup_test_app() -> App {
        let mut app = App::new();

        // Add required resources for setup_container
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
            fruit_linear_damping: 0.5,
            fruit_angular_damping: 1.0,
            keyboard_move_speed: 300.0,
        };
        let handle = physics_assets.add(physics_config);

        app.insert_resource(physics_assets);
        app.insert_resource(PhysicsConfigHandle(handle));

        app
    }

    #[test]
    fn test_container_setup() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

        // Verify 3 container walls exist
        let mut query = app.world_mut().query::<&Container>();
        let wall_count = query.iter(app.world()).count();
        assert_eq!(wall_count, 3, "Should have exactly 3 container walls");
    }

    #[test]
    fn test_container_rigid_bodies() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

        // Verify all walls have RigidBody::Fixed
        let mut query = app.world_mut().query::<(&Container, &RigidBody)>();
        let walls: Vec<_> = query.iter(app.world()).collect();

        assert_eq!(walls.len(), 3);
        for (_, body) in walls {
            assert_eq!(*body, RigidBody::Fixed);
        }
    }

    #[test]
    fn test_container_colliders() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

        // Verify all walls have colliders
        let mut query = app.world_mut().query::<(&Container, &Collider)>();
        let collider_count = query.iter(app.world()).count();
        assert_eq!(collider_count, 3, "All walls should have colliders");
    }

    #[test]
    fn test_container_friction() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

        // Verify friction coefficient
        let mut query = app.world_mut().query::<(&Container, &Friction)>();
        for (_, friction) in query.iter(app.world()) {
            assert_eq!(friction.coefficient, 0.5);
        }
    }

    #[test]
    fn test_container_restitution() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

        // Verify restitution coefficient for side walls (0.3)
        let mut query = app
            .world_mut()
            .query::<(&Container, &Restitution, Option<&BottomWall>)>();
        for (_, restitution, bottom_wall) in query.iter(app.world()) {
            if bottom_wall.is_some() {
                // Bottom wall should have no bounce
                assert_eq!(restitution.coefficient, 0.0);
            } else {
                // Side walls should have some bounce (from physics.ron)
                assert_eq!(restitution.coefficient, 0.2);
            }
        }
    }

    #[test]
    fn test_container_sprites() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

        // Verify all walls have sprites
        let mut query = app.world_mut().query::<(&Container, &Sprite)>();
        let sprite_count = query.iter(app.world()).count();
        assert_eq!(sprite_count, 3, "All walls should have sprites");
    }

    #[test]
    fn test_boundary_line_exists() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

        // Verify exactly one boundary line exists
        let mut query = app.world_mut().query::<&BoundaryLine>();
        let boundary_count = query.iter(app.world()).count();
        assert_eq!(boundary_count, 1, "Should have exactly one boundary line");
    }

    #[test]
    fn test_boundary_line_position() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

        // Verify boundary line Y position (default from fallback)
        let mut query = app.world_mut().query::<(&BoundaryLine, &Transform)>();
        for (_, transform) in query.iter(app.world()) {
            // Using fallback value 300.0 since PhysicsConfig not loaded in test
            assert_eq!(
                transform.translation.y, 300.0,
                "Boundary line should be at default position"
            );
        }
    }

    #[test]
    fn test_boundary_line_no_physics() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

        // Verify boundary line has no RigidBody
        let mut query = app.world_mut().query::<(&BoundaryLine, &RigidBody)>();
        let rigid_body_count = query.iter(app.world()).count();
        assert_eq!(
            rigid_body_count, 0,
            "Boundary line should not have a RigidBody (visual only)"
        );

        // Verify boundary line has no Collider
        let mut query = app.world_mut().query::<(&BoundaryLine, &Collider)>();
        let collider_count = query.iter(app.world()).count();
        assert_eq!(
            collider_count, 0,
            "Boundary line should not have a Collider (visual only)"
        );
    }

    #[test]
    fn test_boundary_line_sprite() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

        // Verify boundary line has a sprite
        let mut query = app.world_mut().query::<(&BoundaryLine, &Sprite)>();
        let sprite_count = query.iter(app.world()).count();
        assert_eq!(sprite_count, 1, "Boundary line should have a sprite");

        // Verify the sprite color is red with transparency
        let mut query = app.world_mut().query::<(&BoundaryLine, &Sprite)>();
        for (_, sprite) in query.iter(app.world()) {
            let color = sprite.color.to_srgba();
            assert_eq!(color.red, 1.0, "Boundary line should be red");
            assert!(
                color.alpha < 1.0,
                "Boundary line should be semi-transparent"
            );
        }
    }
}
