//! Screen-droplet overlay effect
//!
//! Spawns large translucent water-droplet shapes on the overlay layer (Z ≈ 998)
//! that simulate droplets on the camera lens.  Each drop slides slowly
//! downward and fades out at the end of its lifetime.
//!
//! **Triggers:**
//! - A large-fruit merge (Peach or above) → a small burst of drops.
//! - [`WeatherState::Rainy`] → drops spawn periodically throughout the session.

use bevy::prelude::*;
use rand::RngExt;

use crate::events::FruitMergeEvent;
use crate::resources::weather::{CurrentWeather, WeatherState};
use crate::shaders::{ScreenDropletMaterial, UnitQuadMesh};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Hard cap on simultaneous screen droplets.
pub const MAX_SCREEN_DROPLETS: u32 = 30;
/// Minimum fruit stage index (0-based) that triggers screen droplets on merge.
/// `7` = Peach (Cherry=0, …, Peach=7, Pineapple=8, …).
pub const SCREEN_DROPLET_MIN_STAGE: usize = 7;
/// Droplets spawned per qualifying merge event.
pub const SCREEN_DROPLETS_PER_MERGE: u32 = 3;
/// Visual radius of each screen droplet (world units).
pub const SCREEN_DROPLET_RADIUS: f32 = 28.0;
/// Minimum slide-down speed (world units / second).
pub const SCREEN_DROPLET_SPEED_MIN: f32 = 30.0;
/// Maximum slide-down speed (world units / second).
pub const SCREEN_DROPLET_SPEED_MAX: f32 = 70.0;
/// Minimum droplet lifetime before despawn (seconds).
pub const SCREEN_DROPLET_LIFETIME_MIN: f32 = 3.0;
/// Maximum droplet lifetime before despawn (seconds).
pub const SCREEN_DROPLET_LIFETIME_MAX: f32 = 6.0;
/// Half-width of the screen area where droplets are placed.
pub const SCREEN_DROPLET_X_HALF: f32 = 360.0;
/// Y coordinate at which droplets are initially placed.
pub const SCREEN_DROPLET_SPAWN_Y: f32 = 250.0;
/// Z layer for screen droplets (just below the screen-flash overlay at 999).
pub const SCREEN_DROPLET_Z: f32 = 998.0;
/// Base RGBA colour for screen droplets (semi-transparent white-blue).
pub const SCREEN_DROPLET_COLOR: [f32; 4] = [0.82, 0.92, 1.0, 0.85];

// ---------------------------------------------------------------------------
// Component / Resource
// ---------------------------------------------------------------------------

/// A water droplet on the "camera lens" overlay layer.
#[derive(Component, Debug)]
pub struct ScreenDroplet {
    /// Downward slide speed (world units / second).
    pub speed_y: f32,
    /// Elapsed time since spawn (seconds).
    pub lifetime: f32,
    /// Total lifetime before despawn (seconds).
    pub max_lifetime: f32,
    /// Current alpha value (updated each frame by the update system).
    pub alpha: f32,
}

/// Accumulates time for the rain-weather screen-droplet rate limiter.
#[derive(Resource, Default)]
pub struct ScreenDropletSpawnTimer {
    /// Accumulated seconds since the last screen-droplet spawn burst.
    pub elapsed: f32,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns screen droplets when a large fruit (Peach or above) is merged.
pub fn spawn_screen_droplets_on_merge(
    mut commands: Commands,
    mut merge_events: MessageReader<FruitMergeEvent>,
    existing: Query<(), With<ScreenDroplet>>,
    unit_quad: Option<Res<UnitQuadMesh>>,
    mut materials: Option<ResMut<Assets<ScreenDropletMaterial>>>,
) {
    let current_count = existing.iter().count() as u32;
    if current_count >= MAX_SCREEN_DROPLETS {
        return;
    }

    for event in merge_events.read() {
        if event.fruit_type.stage_index() < SCREEN_DROPLET_MIN_STAGE {
            continue;
        }
        let remaining = MAX_SCREEN_DROPLETS - current_count;
        let count = SCREEN_DROPLETS_PER_MERGE.min(remaining);
        spawn_n_screen_droplets(&mut commands, count, &unit_quad, &mut materials);
    }
}

/// Spawns screen droplets periodically during rainy weather.
pub fn spawn_screen_droplets_rain(
    mut commands: Commands,
    weather: Res<CurrentWeather>,
    mut timer: ResMut<ScreenDropletSpawnTimer>,
    time: Res<Time>,
    existing: Query<(), With<ScreenDroplet>>,
    unit_quad: Option<Res<UnitQuadMesh>>,
    mut materials: Option<ResMut<Assets<ScreenDropletMaterial>>>,
) {
    if weather.state != WeatherState::Rainy {
        timer.elapsed = 0.0;
        return;
    }

    let current_count = existing.iter().count() as u32;
    if current_count >= MAX_SCREEN_DROPLETS {
        return;
    }

    let rate = weather.state.params().screen_droplet_rate;
    if rate <= 0.0 {
        return;
    }

    timer.elapsed += time.delta_secs();
    let interval = 1.0 / rate;
    if timer.elapsed < interval {
        return;
    }
    timer.elapsed -= interval;

    spawn_n_screen_droplets(&mut commands, 1, &unit_quad, &mut materials);
}

/// Advances droplets: slides them down, fades the tail end, and despawns them.
pub fn update_screen_droplets(
    mut commands: Commands,
    mut droplets: Query<(Entity, &mut ScreenDroplet, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut droplet, mut transform) in droplets.iter_mut() {
        transform.translation.y -= droplet.speed_y * dt;
        droplet.lifetime += dt;

        // Fade out during the last 20 % of the lifetime.
        let progress = if droplet.max_lifetime > 0.0 {
            (droplet.lifetime / droplet.max_lifetime).clamp(0.0, 1.0)
        } else {
            1.0
        };
        droplet.alpha = if progress > 0.8 {
            SCREEN_DROPLET_COLOR[3] * ((1.0 - progress) / 0.2)
        } else {
            SCREEN_DROPLET_COLOR[3]
        };

        if droplet.lifetime >= droplet.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// Syncs screen-droplet alpha to `Sprite` (fallback / headless mode).
pub fn sync_screen_droplet_sprite_alpha(
    mut droplets: Query<
        (&ScreenDroplet, &mut Sprite),
        Without<MeshMaterial2d<ScreenDropletMaterial>>,
    >,
) {
    for (droplet, mut sprite) in droplets.iter_mut() {
        sprite.color = sprite.color.with_alpha(droplet.alpha);
    }
}

/// Syncs screen-droplet alpha to [`ScreenDropletMaterial`] (shader mode).
pub fn sync_screen_droplet_material_alpha(
    droplets: Query<(&ScreenDroplet, &MeshMaterial2d<ScreenDropletMaterial>)>,
    mut materials: Option<ResMut<Assets<ScreenDropletMaterial>>>,
) {
    let Some(ref mut mats) = materials else {
        return;
    };
    for (droplet, handle) in droplets.iter() {
        if let Some(mat) = mats.get_mut(&handle.0) {
            mat.color.alpha = droplet.alpha;
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helper
// ---------------------------------------------------------------------------

fn spawn_n_screen_droplets(
    commands: &mut Commands,
    count: u32,
    unit_quad: &Option<Res<UnitQuadMesh>>,
    materials: &mut Option<ResMut<Assets<ScreenDropletMaterial>>>,
) {
    let mut rng = rand::rng();
    let diameter = SCREEN_DROPLET_RADIUS * 2.0;

    for _ in 0..count {
        let x: f32 = rng.random_range(-SCREEN_DROPLET_X_HALF..SCREEN_DROPLET_X_HALF);
        let y: f32 = rng.random_range((SCREEN_DROPLET_SPAWN_Y - 40.0)..SCREEN_DROPLET_SPAWN_Y);
        let speed_y = rng.random_range(SCREEN_DROPLET_SPEED_MIN..SCREEN_DROPLET_SPEED_MAX);
        let max_lifetime =
            rng.random_range(SCREEN_DROPLET_LIFETIME_MIN..SCREEN_DROPLET_LIFETIME_MAX);

        let droplet = ScreenDroplet {
            speed_y,
            lifetime: 0.0,
            max_lifetime,
            alpha: SCREEN_DROPLET_COLOR[3],
        };

        match (unit_quad.as_ref(), materials.as_mut()) {
            (Some(quad), Some(mats)) => {
                let color = LinearRgba::new(
                    SCREEN_DROPLET_COLOR[0],
                    SCREEN_DROPLET_COLOR[1],
                    SCREEN_DROPLET_COLOR[2],
                    SCREEN_DROPLET_COLOR[3],
                );
                let mat = mats.add(ScreenDropletMaterial { color });
                commands.spawn((
                    droplet,
                    Mesh2d(quad.0.clone()),
                    MeshMaterial2d(mat),
                    Transform::from_translation(Vec3::new(x, y, SCREEN_DROPLET_Z))
                        .with_scale(Vec3::splat(diameter)),
                ));
            }
            _ => {
                // Sprite fallback (test / headless mode)
                commands.spawn((
                    droplet,
                    Sprite {
                        color: Color::srgba(
                            SCREEN_DROPLET_COLOR[0],
                            SCREEN_DROPLET_COLOR[1],
                            SCREEN_DROPLET_COLOR[2],
                            SCREEN_DROPLET_COLOR[3],
                        ),
                        custom_size: Some(Vec2::splat(diameter)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(x, y, SCREEN_DROPLET_Z)),
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fruit::FruitType;

    #[test]
    fn test_screen_droplet_min_stage_is_peach() {
        // Peach is the 8th fruit (stage index 7, 0-based).
        assert_eq!(
            FruitType::Peach.stage_index(),
            7,
            "Peach should be stage index 7"
        );
        assert!(
            FruitType::Peach.stage_index() >= SCREEN_DROPLET_MIN_STAGE,
            "Peach should trigger screen droplets"
        );
        assert!(
            FruitType::Pear.stage_index() < SCREEN_DROPLET_MIN_STAGE,
            "Pear should NOT trigger screen droplets"
        );
    }

    #[test]
    fn test_screen_droplet_lifetime_range() {
        assert!(
            SCREEN_DROPLET_LIFETIME_MIN < SCREEN_DROPLET_LIFETIME_MAX,
            "Min lifetime must be less than max"
        );
    }

    #[test]
    fn test_screen_droplet_speed_range() {
        assert!(
            SCREEN_DROPLET_SPEED_MIN < SCREEN_DROPLET_SPEED_MAX,
            "Min speed must be less than max"
        );
    }
}
