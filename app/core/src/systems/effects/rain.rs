//! Rain-drop particle system
//!
//! Spawns thin diagonal sprite streaks that fall from the top of the screen to
//! the bottom, simulating rainfall.  Active only when the current weather is
//! [`WeatherState::Rainy`].

use bevy::prelude::*;
use rand::RngExt;

use crate::resources::weather::{CurrentWeather, WeatherState};
use crate::shaders::{RainDropMaterial, UnitQuadMesh};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum simultaneous rain-drop sprites.
pub const MAX_RAIN_PARTICLES: u32 = 200;
/// Minimum downward fall speed (world units / second).
pub const RAIN_SPEED_MIN: f32 = 400.0;
/// Maximum downward fall speed (world units / second).
pub const RAIN_SPEED_MAX: f32 = 600.0;
/// Minimum horizontal wind speed (consistently leftward for realism).
pub const RAIN_WIND_MIN: f32 = -110.0;
/// Maximum horizontal wind speed (mostly leftward, slight variation).
pub const RAIN_WIND_MAX: f32 = -20.0;
/// Y coordinate at which drops are spawned (above the visible area).
pub const RAIN_SPAWN_Y: f32 = 340.0;
/// Y coordinate at which drops are despawned (below the visible area).
pub const RAIN_DESPAWN_Y: f32 = -340.0;
/// Half-width of the horizontal spawn zone.
pub const RAIN_SPAWN_X_HALF: f32 = 450.0;
/// Sprite width (thin streak).
pub const RAIN_WIDTH: f32 = 2.0;
/// Sprite height (long enough to read as a streak).
pub const RAIN_HEIGHT: f32 = 28.0;
/// Rain-drop base colour — translucent light blue (alpha is randomised per drop).
pub const RAIN_COLOR_BASE: Color = Color::srgb(0.72, 0.84, 0.97);
/// Minimum per-drop alpha (far/background drops).
pub const RAIN_ALPHA_MIN: f32 = 0.25;
/// Maximum per-drop alpha (near/foreground drops).
pub const RAIN_ALPHA_MAX: f32 = 0.65;
/// Number of drops spawned per frame when filling up to [`MAX_RAIN_PARTICLES`].
pub const RAIN_BATCH_SPAWN: u32 = 5;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// A single rain-drop sprite.
///
/// The sprite is a thin rotated quad that moves at a constant velocity.
/// When it leaves [`RAIN_DESPAWN_Y`] it is despawned and a new one will be
/// spawned by [`spawn_rain`] on the next frame.
#[derive(Component, Debug)]
pub struct RainDrop {
    /// Horizontal velocity (wind offset), in world units / second.
    pub vel_x: f32,
    /// Vertical velocity (always negative — downward), in world units / second.
    pub vel_y: f32,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Fills rain particles up to [`MAX_RAIN_PARTICLES`] when weather is Rainy.
///
/// Each drop is rendered as a soft-edged streak via [`RainDropMaterial`] when
/// the shader system is available, falling back to a plain [`Sprite`] otherwise.
pub fn spawn_rain(
    mut commands: Commands,
    weather: Res<CurrentWeather>,
    existing: Query<(), With<RainDrop>>,
    unit_quad: Option<Res<UnitQuadMesh>>,
    mut rain_materials: Option<ResMut<Assets<RainDropMaterial>>>,
) {
    if weather.state != WeatherState::Rainy {
        return;
    }

    let current_count = existing.iter().count() as u32;
    let max = weather.state.params().max_rain_particles;
    if current_count >= max {
        return;
    }

    let to_spawn = (max - current_count).min(RAIN_BATCH_SPAWN);
    let mut rng = rand::rng();

    for _ in 0..to_spawn {
        let x: f32 = rng.random_range(-RAIN_SPAWN_X_HALF..RAIN_SPAWN_X_HALF);
        let vel_y = -rng.random_range(RAIN_SPEED_MIN..RAIN_SPEED_MAX);
        let vel_x = rng.random_range(RAIN_WIND_MIN..RAIN_WIND_MAX);

        // Tilt sprite/mesh to match the actual fall direction
        let angle = (vel_x / vel_y.abs()).atan();

        // Spread initial Y so the screen fills immediately on session start
        let y: f32 = rng.random_range(RAIN_DESPAWN_Y..RAIN_SPAWN_Y);

        // Per-drop alpha variation gives depth (far drops are more transparent)
        let alpha = rng.random_range(RAIN_ALPHA_MIN..RAIN_ALPHA_MAX);

        let transform = Transform {
            translation: Vec3::new(x, y, 3.0),
            rotation: Quat::from_rotation_z(angle),
            scale: Vec3::new(RAIN_WIDTH, RAIN_HEIGHT, 1.0),
        };

        match (unit_quad.as_ref(), rain_materials.as_mut()) {
            (Some(quad), Some(mats)) => {
                // Shader path: soft-edged streak with centre glow
                let color = LinearRgba::new(0.72, 0.84, 0.97, alpha);
                let mat = mats.add(RainDropMaterial { color });
                commands.spawn((
                    RainDrop { vel_x, vel_y },
                    Mesh2d(quad.0.clone()),
                    MeshMaterial2d(mat),
                    transform,
                ));
            }
            _ => {
                // Sprite fallback (headless / test contexts without ShadersPlugin)
                commands.spawn((
                    RainDrop { vel_x, vel_y },
                    Sprite {
                        color: RAIN_COLOR_BASE.with_alpha(alpha),
                        custom_size: Some(Vec2::new(RAIN_WIDTH, RAIN_HEIGHT)),
                        ..default()
                    },
                    Transform {
                        translation: transform.translation,
                        rotation: transform.rotation,
                        ..default()
                    },
                ));
            }
        }
    }
}

/// Advances rain particles and despawns those that leave the visible area.
pub fn update_rain(
    mut commands: Commands,
    mut drops: Query<(Entity, &RainDrop, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, drop, mut transform) in drops.iter_mut() {
        transform.translation.x += drop.vel_x * dt;
        transform.translation.y += drop.vel_y * dt;

        if transform.translation.y < RAIN_DESPAWN_Y {
            commands.entity(entity).despawn();
        }
    }
}

/// Despawns all rain particles when weather is no longer [`WeatherState::Rainy`].
pub fn cleanup_rain_on_weather_change(
    mut commands: Commands,
    weather: Res<CurrentWeather>,
    drops: Query<Entity, With<RainDrop>>,
) {
    if weather.state == WeatherState::Rainy {
        return;
    }
    for entity in drops.iter() {
        commands.entity(entity).despawn();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rain_speed_range_valid() {
        assert!(
            RAIN_SPEED_MIN < RAIN_SPEED_MAX,
            "Min speed must be less than max speed"
        );
    }

    #[test]
    fn test_rain_wind_range_valid() {
        assert!(
            RAIN_WIND_MIN < RAIN_WIND_MAX,
            "Min wind must be less than max wind"
        );
    }

    #[test]
    fn test_rain_spawn_y_above_despawn_y() {
        assert!(
            RAIN_SPAWN_Y > RAIN_DESPAWN_Y,
            "Spawn Y must be above despawn Y"
        );
    }

    #[test]
    fn test_spawn_rain_only_when_rainy() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CurrentWeather>(); // Defaults to Sunny
        app.add_systems(Update, spawn_rain);
        app.update();

        let count = app
            .world_mut()
            .query::<&RainDrop>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "No rain should spawn during Sunny weather");
    }

    #[test]
    fn test_spawn_rain_spawns_when_rainy() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(CurrentWeather {
            state: WeatherState::Rainy,
        });
        app.add_systems(Update, spawn_rain);
        app.update();

        let count = app
            .world_mut()
            .query::<&RainDrop>()
            .iter(app.world())
            .count();
        assert!(count > 0, "Rain should spawn during Rainy weather");
    }
}
