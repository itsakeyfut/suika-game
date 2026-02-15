//! 2.5D Camera Setup
//!
//! This module handles the camera configuration for the game's
//! 2.5D oblique overhead view. The camera uses orthographic projection
//! and is angled to provide depth perception while maintaining
//! 2D physics simulation.

use bevy::prelude::*;

/// Sets up the 2.5D camera for the game
///
/// This system creates an orthographic camera with the following properties:
/// - Orthographic projection (no perspective distortion)
/// - Positioned along the Z-axis, looking straight down at the game plane
/// - Z-axis is used for sprite layering (higher Z = closer to camera)
///
/// # Camera Configuration
///
/// The camera is positioned for a top-down orthographic view:
/// - X: 0.0 (centered horizontally)
/// - Y: 0.0 (centered vertically)
/// - Z: 999.9 (far from the game plane, allowing sprites with Z < 999.9 to render)
///
/// # 2.5D Effect
///
/// The "2.5D" visual effect is achieved through Z-based sprite layering,
/// not through camera rotation. Sprites can use different Z values to
/// control rendering order and create depth perception while the physics
/// simulation remains strictly 2D on the X-Y plane.
///
/// The projection is orthographic with a scale that fits the game container.
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 999.9),
        Projection::Orthographic(OrthographicProjection {
            // Set the scale to show the entire game area
            // The viewport will be scaled to fit the window
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        }),
    ));

    info!("2.5D camera initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_setup() {
        let mut app = App::new();
        app.add_systems(Startup, setup_camera);
        app.update();

        // Verify camera entity exists
        let mut query = app.world_mut().query::<&Camera2d>();
        let camera_count = query.iter(app.world()).count();
        assert_eq!(camera_count, 1, "Should have exactly one camera");
    }

    #[test]
    fn test_camera_transform() {
        let mut app = App::new();
        app.add_systems(Startup, setup_camera);
        app.update();

        // Verify camera transform
        let mut query = app.world_mut().query::<(&Camera2d, &Transform)>();
        let Ok((_, transform)) = query.single(app.world()) else {
            panic!("Camera not found");
        };

        assert_eq!(transform.translation.x, 0.0);
        assert_eq!(transform.translation.y, 0.0);
        assert_eq!(transform.translation.z, 999.9);
    }

    #[test]
    fn test_orthographic_projection() {
        let mut app = App::new();
        app.add_systems(Startup, setup_camera);
        app.update();

        // Verify orthographic projection exists
        let mut query = app.world_mut().query::<(&Camera2d, &Projection)>();
        let Ok((_, projection)) = query.single(app.world()) else {
            panic!("Camera projection not found");
        };

        if let Projection::Orthographic(ortho) = projection {
            assert_eq!(ortho.scale, 1.0);
        } else {
            panic!("Expected orthographic projection");
        }
    }
}
