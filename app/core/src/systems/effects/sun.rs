//! Animated sun disc for Sunny weather.
//!
//! Spawns a warm glowing sun in the upper-right area of the play field when
//! the weather is [`WeatherState::Sunny`].  The disc is rendered through
//! [`SunMaterial`] and its light rays rotate slowly via an elapsed-time
//! uniform updated each frame.

use bevy::prelude::*;

use crate::resources::weather::{CurrentWeather, WeatherState};
use crate::shaders::{SunMaterial, UnitQuadMesh};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// World-space centre of the sun disc (upper-right, behind the container).
pub const SUN_POSITION: Vec3 = Vec3::new(250.0, 255.0, 2.0);
/// Side length of the sun quad in world units.
pub const SUN_SIZE: f32 = 210.0;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Marks the sun entity and stores its material handle for per-frame updates.
#[derive(Component)]
pub struct SunEffect {
    /// Handle used to update `params.x` (elapsed time) each frame.
    pub material_handle: Handle<SunMaterial>,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the animated sun disc at the start of a Sunny play session.
///
/// Run on `OnEnter(AppState::Playing)`.
/// Does nothing when the weather is not Sunny or when shaders are unavailable.
pub fn setup_sun(
    mut commands: Commands,
    weather: Res<CurrentWeather>,
    unit_quad: Option<Res<UnitQuadMesh>>,
    mut sun_materials: Option<ResMut<Assets<SunMaterial>>>,
) {
    if weather.state != WeatherState::Sunny {
        return;
    }

    match (unit_quad.as_ref(), sun_materials.as_mut()) {
        (Some(quad), Some(mats)) => {
            let mat = mats.add(SunMaterial {
                color: LinearRgba::new(1.0, 0.95, 0.70, 0.90),
                params: Vec4::ZERO,
            });
            let mat_handle = mat.clone();
            commands.spawn((
                SunEffect {
                    material_handle: mat_handle,
                },
                Mesh2d(quad.0.clone()),
                MeshMaterial2d(mat),
                Transform {
                    translation: SUN_POSITION,
                    scale: Vec3::new(SUN_SIZE, SUN_SIZE, 1.0),
                    ..default()
                },
            ));
            info!("Sun spawned at position {:?}", SUN_POSITION);
        }
        _ => {
            // Sun is a shader-only effect; skip in headless / test contexts.
        }
    }
}

/// Updates `SunMaterial.params.x` each frame so the light rays rotate.
///
/// Run every `Update` while `AppState::Playing`.
pub fn animate_sun(
    sun_query: Query<&SunEffect>,
    mut sun_materials: Option<ResMut<Assets<SunMaterial>>>,
    time: Res<Time>,
) {
    let Some(ref mut mats) = sun_materials else {
        return;
    };
    let elapsed = time.elapsed_secs();
    for sun_effect in sun_query.iter() {
        if let Some(mat) = mats.get_mut(&sun_effect.material_handle) {
            mat.params.x = elapsed;
        }
    }
}

/// Despawns all sun entities when leaving the Playing state.
///
/// Run on `OnExit(AppState::Playing)`.
pub fn despawn_sun(mut commands: Commands, sun_query: Query<Entity, With<SunEffect>>) {
    for entity in sun_query.iter() {
        commands.entity(entity).despawn();
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
    fn test_setup_sun_skips_non_sunny() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(CurrentWeather {
            state: WeatherState::Rainy,
        });
        app.add_systems(bevy::app::Startup, setup_sun);
        app.update();

        let count = app
            .world_mut()
            .query::<&SunEffect>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "No sun should spawn during non-Sunny weather");
    }

    #[test]
    fn test_despawn_sun_removes_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Manually insert a fake SunEffect entity (no shader needed)
        app.world_mut().spawn((
            SunEffect {
                material_handle: Handle::default(),
            },
            Transform::default(),
        ));
        app.add_systems(Update, despawn_sun);
        app.update();

        let count = app
            .world_mut()
            .query::<&SunEffect>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "despawn_sun should remove all SunEffect entities");
    }
}
