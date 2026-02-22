//! Weather post-process overlay system
//!
//! Manages the full-screen colour overlay that visually differentiates weather
//! conditions during gameplay:
//!
//! | Weather | Overlay | Bloom |
//! |---------|---------|-------|
//! | Sunny   | none (transparent) | on |
//! | Rainy   | cool blue wash     | off |
//! | Cloudy  | grey wash          | off |
//!
//! The overlay is a large quad at Z = 997 rendered through
//! [`WeatherPostprocessMaterial`].  Bloom is applied by inserting / removing
//! the [`Bloom`] component on the camera entity.

use bevy::prelude::*;

use crate::resources::weather::CurrentWeather;
use crate::shaders::{FullScreenQuadMesh, WeatherPostprocessMaterial};

/// Z layer for the weather overlay (below screen droplets at 998).
pub const WEATHER_OVERLAY_Z: f32 = 997.0;

// ---------------------------------------------------------------------------
// Marker component
// ---------------------------------------------------------------------------

/// Marks the full-screen weather-overlay entity so it can be despawned on exit.
#[derive(Component)]
pub struct WeatherOverlay;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the weather overlay entity at the start of a play session.
///
/// Run on `OnEnter(AppState::Playing)`.
/// For Sunny weather the overlay colour has alpha=0 so no overlay is spawned.
pub fn setup_weather_overlay(
    mut commands: Commands,
    weather: Res<CurrentWeather>,
    quad: Option<Res<FullScreenQuadMesh>>,
    mut materials: Option<ResMut<Assets<WeatherPostprocessMaterial>>>,
) {
    let params = weather.state.params();
    let [r, g, b, a] = params.overlay_color;

    // Don't spawn an overlay when alpha is zero (Sunny weather).
    if a <= 0.0 {
        return;
    }

    match (quad.as_ref(), materials.as_mut()) {
        (Some(quad), Some(mats)) => {
            let color = LinearRgba::new(r, g, b, a);
            let mat = mats.add(WeatherPostprocessMaterial { color });
            commands.spawn((
                WeatherOverlay,
                Mesh2d(quad.0.clone()),
                MeshMaterial2d(mat),
                Transform::from_translation(Vec3::new(0.0, 0.0, WEATHER_OVERLAY_Z)),
            ));
            info!(
                "WeatherOverlay spawned (shader): weather={:?} rgba=({:.2},{:.2},{:.2},{:.2})",
                weather.state, r, g, b, a
            );
        }
        _ => {
            // Sprite fallback (headless / test contexts without ShadersPlugin).
            commands.spawn((
                WeatherOverlay,
                Sprite {
                    color: Color::srgba(r, g, b, a),
                    custom_size: Some(Vec2::splat(20_000.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, WEATHER_OVERLAY_Z)),
            ));
            info!(
                "WeatherOverlay spawned (sprite fallback): weather={:?} rgba=({:.2},{:.2},{:.2},{:.2})",
                weather.state, r, g, b, a
            );
        }
    }
}

/// Despawns the weather overlay when leaving the Playing state.
///
/// Run on `OnExit(AppState::Playing)`.
pub fn despawn_weather_overlay(
    mut commands: Commands,
    overlays: Query<Entity, With<WeatherOverlay>>,
) {
    for entity in overlays.iter() {
        commands.entity(entity).despawn();
    }
}

/// Applies or removes camera [`Bloom`] based on the current weather.
///
/// Run on `OnEnter(AppState::Playing)`.
/// Requires Bevy's core-pipeline bloom feature and an HDR-capable camera.
pub fn apply_weather_bloom(
    mut commands: Commands,
    weather: Res<CurrentWeather>,
    cameras: Query<Entity, With<Camera2d>>,
) {
    let params = weather.state.params();
    for camera_entity in cameras.iter() {
        if params.bloom {
            commands
                .entity(camera_entity)
                .insert(bevy::post_process::bloom::Bloom {
                    intensity: params.bloom_intensity,
                    ..default()
                });
            info!(
                "Bloom enabled: weather={:?} intensity={:.2}",
                weather.state, params.bloom_intensity
            );
        } else {
            commands
                .entity(camera_entity)
                .remove::<bevy::post_process::bloom::Bloom>();
            info!("Bloom disabled: weather={:?}", weather.state);
        }
    }
}

/// Removes camera bloom when leaving the Playing state.
///
/// Run on `OnExit(AppState::Playing)`.
pub fn remove_weather_bloom(mut commands: Commands, cameras: Query<Entity, With<Camera2d>>) {
    for camera_entity in cameras.iter() {
        commands
            .entity(camera_entity)
            .remove::<bevy::post_process::bloom::Bloom>();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::weather::WeatherState;

    #[test]
    fn test_setup_weather_overlay_sunny_spawns_nothing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(CurrentWeather {
            state: WeatherState::Sunny,
        });
        app.add_systems(bevy::app::Startup, setup_weather_overlay);
        app.update();

        let count = app
            .world_mut()
            .query::<&WeatherOverlay>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "Sunny weather should not spawn any overlay");
    }

    #[test]
    fn test_setup_weather_overlay_rainy_spawns_overlay() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(CurrentWeather {
            state: WeatherState::Rainy,
        });
        app.add_systems(bevy::app::Startup, setup_weather_overlay);
        app.update();

        let count = app
            .world_mut()
            .query::<&WeatherOverlay>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "Rainy weather should spawn one overlay entity");
    }

    #[test]
    fn test_despawn_weather_overlay_removes_entity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(bevy::app::Startup, |mut commands: Commands| {
            commands.spawn((WeatherOverlay, Sprite::default(), Transform::default()));
        });
        app.add_systems(Update, despawn_weather_overlay);
        app.update(); // Startup
        app.update(); // Update (runs despawn)

        let count = app
            .world_mut()
            .query::<&WeatherOverlay>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "All overlay entities should be despawned");
    }
}
