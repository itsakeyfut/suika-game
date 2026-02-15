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
/// Based on `constants::physics`:
/// - Width: 600 pixels
/// - Height: 800 pixels
/// - Wall thickness: 20 pixels
pub fn setup_container(mut commands: Commands) {
    let container_width = constants::physics::CONTAINER_WIDTH;
    let container_height = constants::physics::CONTAINER_HEIGHT;
    let wall_thickness = constants::physics::WALL_THICKNESS;

    // Calculate wall positions
    let half_width = container_width / 2.0;
    let half_height = container_height / 2.0;

    // Left wall
    commands.spawn((
        Container,
        RigidBody::Fixed,
        Collider::cuboid(wall_thickness / 2.0, half_height),
        Friction::coefficient(0.5),
        Restitution::coefficient(0.3),
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
        Friction::coefficient(0.5),
        Restitution::coefficient(0.3),
        Transform::from_xyz(half_width + wall_thickness / 2.0, 0.0, 0.0),
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(wall_thickness, container_height)),
            ..default()
        },
    ));

    // Bottom wall
    commands.spawn((
        Container,
        RigidBody::Fixed,
        Collider::cuboid(half_width + wall_thickness, wall_thickness / 2.0),
        Friction::coefficient(0.5),
        Restitution::coefficient(0.3),
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

    info!("Game container initialized with 3 walls");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_setup() {
        let mut app = App::new();
        app.add_systems(Startup, setup_container);
        app.update();

        // Verify 3 container walls exist
        let mut query = app.world_mut().query::<&Container>();
        let wall_count = query.iter(app.world()).count();
        assert_eq!(wall_count, 3, "Should have exactly 3 container walls");
    }

    #[test]
    fn test_container_rigid_bodies() {
        let mut app = App::new();
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
        let mut app = App::new();
        app.add_systems(Startup, setup_container);
        app.update();

        // Verify all walls have colliders
        let mut query = app.world_mut().query::<(&Container, &Collider)>();
        let collider_count = query.iter(app.world()).count();
        assert_eq!(collider_count, 3, "All walls should have colliders");
    }

    #[test]
    fn test_container_friction() {
        let mut app = App::new();
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
        let mut app = App::new();
        app.add_systems(Startup, setup_container);
        app.update();

        // Verify restitution coefficient
        let mut query = app.world_mut().query::<(&Container, &Restitution)>();
        for (_, restitution) in query.iter(app.world()) {
            assert_eq!(restitution.coefficient, 0.3);
        }
    }

    #[test]
    fn test_container_sprites() {
        let mut app = App::new();
        app.add_systems(Startup, setup_container);
        app.update();

        // Verify all walls have sprites
        let mut query = app.world_mut().query::<(&Container, &Sprite)>();
        let sprite_count = query.iter(app.world()).count();
        assert_eq!(sprite_count, 3, "All walls should have sprites");
    }
}
