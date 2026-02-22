//! Drifting cloud puff effect for Cloudy weather.
//!
//! Spawns multiple soft cloud blobs — rendered via [`CloudMaterial`] and the
//! `cloud_puff.wgsl` shader — that drift slowly from right to left across the
//! upper part of the play field.  Alpha is faded in and out at the horizontal
//! edges so the transition is invisible even when clouds loop back to the right.

use bevy::prelude::*;
use rand::RngExt;

use crate::resources::weather::{CurrentWeather, WeatherState};
use crate::shaders::{CloudMaterial, UnitQuadMesh};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Total number of cloud puff entities kept alive during Cloudy weather.
pub const MAX_CLOUD_PUFFS: usize = 8;
/// Minimum leftward drift speed (world units / second).
pub const CLOUD_SPEED_MIN: f32 = 18.0;
/// Maximum leftward drift speed (world units / second).
pub const CLOUD_SPEED_MAX: f32 = 45.0;
/// X coordinate at which a cloud is (re-)spawned — off the right edge.
pub const CLOUD_SPAWN_X: f32 = 680.0;
/// X coordinate below which a cloud wraps back to the right edge.
pub const CLOUD_DESPAWN_X: f32 = -680.0;
/// Minimum Y coordinate for cloud placement (world units).
pub const CLOUD_Y_MIN: f32 = 65.0;
/// Maximum Y coordinate for cloud placement (world units).
pub const CLOUD_Y_MAX: f32 = 285.0;
/// Width of the horizontal fade zone at each screen edge (world units).
pub const CLOUD_FADE_ZONE: f32 = 130.0;
/// Minimum cloud quad width (world units).
pub const CLOUD_WIDTH_MIN: f32 = 185.0;
/// Maximum cloud quad width (world units).
pub const CLOUD_WIDTH_MAX: f32 = 275.0;
/// Height-to-width ratio for cloud quads (clouds are wider than tall).
pub const CLOUD_HEIGHT_RATIO: f32 = 0.55;
/// Maximum cloud alpha when fully visible (centre of screen).
pub const CLOUD_BASE_ALPHA: f32 = 0.68;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// A single drifting cloud puff entity.
#[derive(Component)]
pub struct CloudPuff {
    /// Leftward drift speed (negative = moves left).
    pub vel_x: f32,
    /// Material handle used to update alpha for edge fading.
    pub material_handle: Handle<CloudMaterial>,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the initial set of cloud puffs at the start of a Cloudy session.
///
/// Clouds are distributed across the full horizontal range so the screen
/// does not look empty at the moment of entry.
///
/// Run on `OnEnter(AppState::Playing)`.
pub fn setup_cloud_puffs(
    mut commands: Commands,
    weather: Res<CurrentWeather>,
    unit_quad: Option<Res<UnitQuadMesh>>,
    mut cloud_materials: Option<ResMut<Assets<CloudMaterial>>>,
) {
    if weather.state != WeatherState::Cloudy {
        return;
    }

    let (Some(quad), Some(mats)) = (unit_quad.as_ref(), cloud_materials.as_mut()) else {
        // Cloud puffs are shader-only; skip in headless / test contexts.
        return;
    };

    let mut rng = rand::rng();

    for i in 0..MAX_CLOUD_PUFFS {
        // Spread clouds evenly from left to right on first spawn.
        let x_frac = i as f32 / MAX_CLOUD_PUFFS as f32;
        let x = CLOUD_DESPAWN_X + (CLOUD_SPAWN_X - CLOUD_DESPAWN_X) * x_frac;
        let y: f32 = rng.random_range(CLOUD_Y_MIN..CLOUD_Y_MAX);

        let width: f32 = rng.random_range(CLOUD_WIDTH_MIN..CLOUD_WIDTH_MAX);
        let height = width * CLOUD_HEIGHT_RATIO;
        let vel_x = -rng.random_range(CLOUD_SPEED_MIN..CLOUD_SPEED_MAX);

        let alpha = edge_alpha(x);
        let mat = mats.add(CloudMaterial {
            color: LinearRgba::new(0.96, 0.96, 0.98, alpha),
        });
        let mat_handle = mat.clone();

        commands.spawn((
            CloudPuff {
                vel_x,
                material_handle: mat_handle,
            },
            Mesh2d(quad.0.clone()),
            MeshMaterial2d(mat),
            Transform {
                translation: Vec3::new(x, y, 1.5),
                scale: Vec3::new(width, height, 1.0),
                ..default()
            },
        ));
    }

    info!("Cloud puffs spawned: {}", MAX_CLOUD_PUFFS);
}

/// Moves clouds leftward and updates alpha for smooth edge fading.
///
/// When a cloud exits the left edge it wraps back to the right, creating a
/// seamless looping effect.  Alpha is continuously re-computed so each cloud
/// fades in from the right and fades out towards the left.
///
/// Run every `Update` while `AppState::Playing`.
pub fn update_cloud_puffs(
    mut cloud_query: Query<(&CloudPuff, &mut Transform)>,
    mut cloud_materials: Option<ResMut<Assets<CloudMaterial>>>,
    time: Res<Time>,
) {
    let Some(ref mut mats) = cloud_materials else {
        return;
    };
    let dt = time.delta_secs();

    for (cloud, mut transform) in cloud_query.iter_mut() {
        transform.translation.x += cloud.vel_x * dt;

        // Wrap around: reappear from the right when past the left boundary.
        if transform.translation.x < CLOUD_DESPAWN_X {
            transform.translation.x = CLOUD_SPAWN_X;
        }

        // Update material alpha based on screen-edge proximity.
        let alpha = edge_alpha(transform.translation.x);
        if let Some(mat) = mats.get_mut(&cloud.material_handle) {
            mat.color.alpha = alpha;
        }
    }
}

/// Despawns all cloud puff entities when leaving the Playing state.
///
/// Run on `OnExit(AppState::Playing)`.
pub fn despawn_cloud_puffs(mut commands: Commands, cloud_query: Query<Entity, With<CloudPuff>>) {
    for entity in cloud_query.iter() {
        commands.entity(entity).despawn();
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns the alpha for a cloud at the given x position, fading to zero
/// near both horizontal edges of the play area.
fn edge_alpha(x: f32) -> f32 {
    let right_fade = smoothstep(CLOUD_SPAWN_X, CLOUD_SPAWN_X - CLOUD_FADE_ZONE, x);
    let left_fade = smoothstep(CLOUD_DESPAWN_X, CLOUD_DESPAWN_X + CLOUD_FADE_ZONE, x);
    (right_fade * left_fade * CLOUD_BASE_ALPHA).clamp(0.0, CLOUD_BASE_ALPHA)
}

/// Scalar smoothstep (CPU equivalent of WGSL's `smoothstep`).
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::weather::WeatherState;

    #[test]
    fn test_setup_cloud_puffs_skips_non_cloudy() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(CurrentWeather {
            state: WeatherState::Sunny,
        });
        app.add_systems(bevy::app::Startup, setup_cloud_puffs);
        app.update();

        let count = app
            .world_mut()
            .query::<&CloudPuff>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "No clouds should spawn during non-Cloudy weather");
    }

    #[test]
    fn test_despawn_cloud_puffs_removes_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Manually insert two fake CloudPuff entities.
        for _ in 0..2 {
            app.world_mut().spawn((
                CloudPuff {
                    vel_x: -30.0,
                    material_handle: Handle::default(),
                },
                Transform::default(),
            ));
        }
        app.add_systems(Update, despawn_cloud_puffs);
        app.update();

        let count = app
            .world_mut()
            .query::<&CloudPuff>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 0,
            "despawn_cloud_puffs should remove all CloudPuff entities"
        );
    }

    #[test]
    fn test_edge_alpha_zero_at_edges() {
        // Exactly at the spawn/despawn boundaries alpha should be 0.
        assert!(
            edge_alpha(CLOUD_SPAWN_X) < 0.01,
            "Alpha at right spawn edge should be near zero"
        );
        assert!(
            edge_alpha(CLOUD_DESPAWN_X) < 0.01,
            "Alpha at left despawn edge should be near zero"
        );
    }

    #[test]
    fn test_edge_alpha_max_at_center() {
        // Alpha near the centre should be close to CLOUD_BASE_ALPHA.
        let centre_alpha = edge_alpha(0.0);
        assert!(
            (centre_alpha - CLOUD_BASE_ALPHA).abs() < 0.01,
            "Alpha at screen centre should equal CLOUD_BASE_ALPHA"
        );
    }
}
