//! Camera setup for the game.
//!
//! Spawns the single orthographic [`Camera2d`] used to render the game world.
//! Registered by [`crate::GameUIPlugin`] at [`Startup`] so the camera is
//! available from the very first frame, before any state transitions occur.

use bevy::prelude::*;
use suika_game_core::prelude::CameraShake;

/// Spawns the orthographic camera used to render the game world.
///
/// The camera is positioned on the Z axis so that all sprites with
/// `z < 999.9` are visible.  The orthographic scale is `1.0`, meaning
/// one world unit equals one logical pixel.
///
/// A [`CameraShake`] component is attached so that the core shake system can
/// apply trauma-based offsets to this camera when fruits merge.
///
/// Bloom support: the [`bevy::post_process::bloom::Bloom`] component is added
/// and removed dynamically by the postprocess system based on [`CurrentWeather`].
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera::default(),
        Transform::from_xyz(0.0, 0.0, 999.9),
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        }),
        CameraShake::default(),
    ));

    info!("Camera initialized");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_setup() {
        let mut app = App::new();
        app.add_systems(Startup, setup_camera);
        app.update();

        let mut query = app.world_mut().query::<&Camera2d>();
        assert_eq!(
            query.iter(app.world()).count(),
            1,
            "Should have exactly one camera"
        );
    }

    #[test]
    fn test_camera_transform() {
        let mut app = App::new();
        app.add_systems(Startup, setup_camera);
        app.update();

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
