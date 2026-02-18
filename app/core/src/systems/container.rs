//! Game container (physics box) setup.
//!
//! Spawns the three fixed walls (left, right, bottom) and the visual boundary
//! line that marks the game-over threshold.  The system reads all dimensions
//! from [`PhysicsConfig`] so the container automatically matches whatever
//! values are in `physics.ron`.
//!
//! Registered by [`crate::GameCorePlugin`] on [`OnExit(AppState::Loading)`] so
//! the config is guaranteed to be fully loaded before the walls are spawned.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::components::{BottomWall, BoundaryLine, Container, LeftWall, RightWall};
use crate::config::{PhysicsConfig, PhysicsConfigHandle};

/// Spawns the three physics walls and the visual boundary line.
///
/// All dimensions are sourced from [`PhysicsConfig`].  The system is
/// intentionally registered on `OnExit(AppState::Loading)` so the config is
/// guaranteed to be loaded — calling `.expect()` here is safe by design.
pub fn setup_container(
    mut commands: Commands,
    physics_handle: Res<PhysicsConfigHandle>,
    physics_assets: Res<Assets<PhysicsConfig>>,
) {
    // Physics config is guaranteed to be loaded because this system runs on
    // OnExit(AppState::Loading), after wait_for_configs confirms all configs ready.
    let config = physics_assets
        .get(&physics_handle.0)
        .expect("PhysicsConfig must be loaded before setup_container runs");

    let (container_width, container_height, wall_thickness, wall_restitution, wall_friction) = (
        config.container_width,
        config.container_height,
        config.wall_thickness,
        config.wall_restitution,
        config.wall_friction,
    );

    let half_width = container_width / 2.0;
    let half_height = container_height / 2.0;

    // Left wall
    commands.spawn((
        Container,
        LeftWall,
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
        RightWall,
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

    // Bottom wall — no bounce, matches original Suika Game behavior
    commands.spawn((
        Container,
        BottomWall,
        RigidBody::Fixed,
        Collider::cuboid(half_width + wall_thickness, wall_thickness / 2.0),
        Friction::coefficient(wall_friction),
        Restitution {
            coefficient: 0.0,
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

    // Boundary line — visual only, no physics
    let line_thickness = 3.0;
    commands.spawn((
        BoundaryLine,
        Transform::from_xyz(0.0, config.boundary_line_y, 0.0),
        Sprite {
            color: Color::srgba(1.0, 0.0, 0.0, 0.5),
            custom_size: Some(Vec2::new(container_width, line_thickness)),
            ..default()
        },
    ));

    info!("Game container initialized with 3 walls and boundary line");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_app() -> App {
        let mut app = App::new();

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

        let mut query = app.world_mut().query::<&Container>();
        assert_eq!(
            query.iter(app.world()).count(),
            3,
            "Should have exactly 3 container walls"
        );
    }

    #[test]
    fn test_container_rigid_bodies() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

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

        let mut query = app.world_mut().query::<(&Container, &Collider)>();
        assert_eq!(
            query.iter(app.world()).count(),
            3,
            "All walls should have colliders"
        );
    }

    #[test]
    fn test_container_friction() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

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

        let mut query = app
            .world_mut()
            .query::<(&Container, &Restitution, Option<&BottomWall>)>();
        for (_, restitution, bottom_wall) in query.iter(app.world()) {
            if bottom_wall.is_some() {
                assert_eq!(restitution.coefficient, 0.0);
            } else {
                assert_eq!(restitution.coefficient, 0.2);
            }
        }
    }

    #[test]
    fn test_container_sprites() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

        let mut query = app.world_mut().query::<(&Container, &Sprite)>();
        assert_eq!(
            query.iter(app.world()).count(),
            3,
            "All walls should have sprites"
        );
    }

    #[test]
    fn test_boundary_line_exists() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

        let mut query = app.world_mut().query::<&BoundaryLine>();
        assert_eq!(
            query.iter(app.world()).count(),
            1,
            "Should have exactly one boundary line"
        );
    }

    #[test]
    fn test_boundary_line_position() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

        let mut query = app.world_mut().query::<(&BoundaryLine, &Transform)>();
        for (_, transform) in query.iter(app.world()) {
            assert_eq!(
                transform.translation.y, 300.0,
                "Boundary line Y should match physics config boundary_line_y"
            );
        }
    }

    #[test]
    fn test_boundary_line_no_physics() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

        let mut query = app.world_mut().query::<(&BoundaryLine, &RigidBody)>();
        assert_eq!(
            query.iter(app.world()).count(),
            0,
            "Boundary line should not have a RigidBody (visual only)"
        );

        let mut query = app.world_mut().query::<(&BoundaryLine, &Collider)>();
        assert_eq!(
            query.iter(app.world()).count(),
            0,
            "Boundary line should not have a Collider (visual only)"
        );
    }

    #[test]
    fn test_boundary_line_sprite() {
        let mut app = setup_test_app();
        app.add_systems(Startup, setup_container);
        app.update();

        let mut query = app.world_mut().query::<(&BoundaryLine, &Sprite)>();
        assert_eq!(
            query.iter(app.world()).count(),
            1,
            "Boundary line should have a sprite"
        );

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
