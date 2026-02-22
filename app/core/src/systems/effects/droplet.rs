//! Water droplet particle system
//!
//! Spawns small water-drop particles on fruit merge and fruit landing.
//! Droplets are affected by gravity and bounce off the container walls,
//! fading out before they despawn.
//!
//! When [`crate::shaders::ShadersPlugin`] is loaded the particles are rendered
//! through [`crate::shaders::SoftCircleMaterial`] (SDF soft-circle shader).
//! In headless / test contexts the fallback plain-`Sprite` path is used.

use bevy::prelude::*;
use rand::RngExt;

use crate::components::{Fruit, FruitSpawnState};
use crate::config::{BounceParams, DropletColorMode, DropletConfig, DropletParams, PhysicsParams};
use crate::events::FruitMergeEvent;
use crate::shaders::{SoftCircleMaterial, UnitQuadMesh};
use crate::systems::effects::bounce::SquashStretchAnimation;

// --- Constants ---

/// Number of droplets spawned on a fruit merge
pub const DROPLET_COUNT_MERGE: u32 = 12;
/// Number of droplets spawned when a fruit lands
pub const DROPLET_COUNT_LANDING: u32 = 5;
/// Visual radius of each droplet in pixels
pub const DROPLET_RADIUS: f32 = 2.5;
/// Minimum initial speed of a droplet (pixels/second)
pub const DROPLET_MIN_SPEED: f32 = 80.0;
/// Maximum initial speed of a droplet (pixels/second)
pub const DROPLET_MAX_SPEED: f32 = 350.0;
/// Minimum droplet lifetime in seconds
pub const DROPLET_LIFETIME_MIN: f32 = 0.4;
/// Maximum droplet lifetime in seconds
pub const DROPLET_LIFETIME_MAX: f32 = 0.9;
/// Gravity applied to droplets (pixels/second²)
pub const DROPLET_GRAVITY: f32 = -600.0;
/// Speed multiplier after a wall bounce (energy lost per bounce)
pub const DROPLET_BOUNCE_DAMPING: f32 = 0.55;
/// Base color of water droplets
pub const DROPLET_COLOR: Color = Color::srgba(0.5, 0.78, 0.95, 0.85);
/// Starting alpha for droplets
pub const DROPLET_INITIAL_ALPHA: f32 = 0.85;

// --- Component ---

/// Water droplet particle component
///
/// A short-lived sprite / soft-circle mesh that flies out from merge/landing
/// points, bounces off the container walls, and fades out over its lifetime.
#[derive(Component, Debug)]
pub struct WaterDroplet {
    /// Current velocity in pixels/second
    pub velocity: Vec2,
    /// Elapsed lifetime in seconds
    pub lifetime: f32,
    /// Total lifetime in seconds before despawn
    pub max_lifetime: f32,
    /// Current alpha value (updated each frame, applied to Sprite or Material).
    pub alpha: f32,
}

// --- Internal helpers ---

/// Scales a base droplet count by the fruit's stage index.
///
/// Larger fruits (higher stage) emit proportionally more particles.
/// The multiplier grows linearly from **1×** at stage 0 (Cherry) to
/// **3×** at stage 10 (Watermelon), producing a noticeable visual
/// difference between a small cherry burst and a watermelon explosion.
///
/// | Stage | Fruit       | Multiplier | Merge count (base 12) |
/// |-------|-------------|------------|-----------------------|
/// | 0     | Cherry      | 1.0×       | 12                    |
/// | 5     | Apple       | 2.0×       | 24                    |
/// | 10    | Watermelon  | 3.0×       | 36                    |
fn scale_count_by_fruit(base: u32, fruit_type: crate::fruit::FruitType) -> u32 {
    const MAX_SCALE: f32 = 3.0;
    const STAGE_COUNT: f32 = 10.0; // Watermelon is stage 10
    let stage = fruit_type.stage_index() as f32;
    let scale = 1.0 + (stage / STAGE_COUNT) * (MAX_SCALE - 1.0);
    ((base as f32 * scale).round() as u32).max(1)
}

/// Resolves the droplet spawn color from config mode and fruit color.
///
/// - `Water`: uses the fixed base color defined in `DropletConfig.color`
/// - `Juice`: uses the fruit's own placeholder color
fn resolve_droplet_color(config: Option<&DropletConfig>, fruit_color: Color) -> Color {
    let mode = config
        .map(|c| c.color_mode)
        .unwrap_or(DropletColorMode::Water);
    match mode {
        DropletColorMode::Water => config
            .map(|c| Color::from(c.color))
            .unwrap_or(DROPLET_COLOR),
        DropletColorMode::Juice => fruit_color,
    }
}

/// Spawns `count` droplets radiating from `position`.
///
/// Uses [`SoftCircleMaterial`] when shader assets are available, otherwise
/// falls back to plain [`Sprite`] (headless / test mode).
fn spawn_droplets(
    commands: &mut Commands,
    position: Vec2,
    color: Color,
    count: u32,
    config: Option<&DropletConfig>,
    unit_quad: Option<&UnitQuadMesh>,
    materials: &mut Option<ResMut<Assets<SoftCircleMaterial>>>,
) {
    let radius = config.map(|c| c.radius).unwrap_or(DROPLET_RADIUS);
    let min_speed = config.map(|c| c.min_speed).unwrap_or(DROPLET_MIN_SPEED);
    let max_speed = config.map(|c| c.max_speed).unwrap_or(DROPLET_MAX_SPEED);
    let lifetime_min = config
        .map(|c| c.lifetime_min)
        .unwrap_or(DROPLET_LIFETIME_MIN);
    let lifetime_max = config
        .map(|c| c.lifetime_max)
        .unwrap_or(DROPLET_LIFETIME_MAX);

    // Guard against inverted ranges that would cause rng.random_range to panic
    let speed_max = if max_speed > min_speed {
        max_speed
    } else {
        warn!(
            "DropletConfig: max_speed ({}) <= min_speed ({}), using min_speed + 1.0",
            max_speed, min_speed
        );
        min_speed + 1.0
    };
    let lt_max = if lifetime_max > lifetime_min {
        lifetime_max
    } else {
        warn!(
            "DropletConfig: lifetime_max ({}) <= lifetime_min ({}), using lifetime_min + 0.01",
            lifetime_max, lifetime_min
        );
        lifetime_min + 0.01
    };

    let size = radius * 2.0;
    let linear_color = color.to_linear();

    let mut rng = rand::rng();
    for _ in 0..count {
        let angle = rng.random_range(0.0_f32..std::f32::consts::TAU);
        let speed = rng.random_range(min_speed..speed_max);
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
        let lifetime = rng.random_range(lifetime_min..lt_max);

        let droplet = WaterDroplet {
            velocity,
            lifetime: 0.0,
            max_lifetime: lifetime,
            alpha: DROPLET_INITIAL_ALPHA,
        };

        match (unit_quad, materials.as_mut()) {
            (Some(quad), Some(mats)) => {
                // Shader path: soft-circle Material2d
                let mat = mats.add(SoftCircleMaterial {
                    color: linear_color,
                });
                commands.spawn((
                    droplet,
                    Mesh2d(quad.0.clone()),
                    MeshMaterial2d(mat),
                    Transform::from_translation(position.extend(5.0)).with_scale(Vec3::splat(size)),
                ));
            }
            _ => {
                // Sprite fallback (headless / test contexts)
                commands.spawn((
                    droplet,
                    Sprite {
                        color,
                        custom_size: Some(Vec2::splat(size)),
                        ..default()
                    },
                    Transform::from_translation(position.extend(5.0)),
                ));
            }
        }
    }
}

// --- Systems ---

/// Spawns water droplets on fruit merge events.
///
/// The number of droplets scales with the resulting fruit's stage so that
/// larger merges produce a more dramatic particle burst.  The base count
/// comes from [`DropletConfig::count_merge`] (or [`DROPLET_COUNT_MERGE`] as
/// fallback) and is multiplied by [`scale_count_by_fruit`].
pub fn spawn_merge_droplets(
    mut commands: Commands,
    mut merge_events: MessageReader<FruitMergeEvent>,
    droplet: DropletParams<'_>,
    unit_quad: Option<Res<UnitQuadMesh>>,
    mut soft_circle_materials: Option<ResMut<Assets<SoftCircleMaterial>>>,
) {
    let config = droplet.get();
    let base_count = config.map(|c| c.count_merge).unwrap_or(DROPLET_COUNT_MERGE);

    for event in merge_events.read() {
        let count = scale_count_by_fruit(base_count, event.fruit_type);
        let fruit_color = event.fruit_type.placeholder_color();
        let color = resolve_droplet_color(config, fruit_color);
        spawn_droplets(
            &mut commands,
            event.position,
            color,
            count,
            config,
            unit_quad.as_deref(),
            &mut soft_circle_materials,
        );
    }
}

/// Handles fruit landing: spawns water droplets and inserts the impact bounce animation
///
/// Uses Bevy's change detection (`Changed<FruitSpawnState>`) to detect the
/// moment a falling fruit lands. For each newly-landed fruit it:
/// 1. Spawns a small splash of water droplets
/// 2. Inserts `SquashStretchAnimation::for_landing` on the fruit entity
#[allow(clippy::type_complexity)]
pub fn handle_fruit_landing(
    mut commands: Commands,
    changed_fruits: Query<
        (
            Entity,
            &FruitSpawnState,
            &Transform,
            &crate::fruit::FruitType,
        ),
        (With<Fruit>, Changed<FruitSpawnState>),
    >,
    droplet: DropletParams<'_>,
    bounce: BounceParams<'_>,
    unit_quad: Option<Res<UnitQuadMesh>>,
    mut soft_circle_materials: Option<ResMut<Assets<SoftCircleMaterial>>>,
) {
    let droplet_cfg = droplet.get();
    let bounce_cfg = bounce.get();
    let base_count = droplet_cfg
        .map(|c| c.count_landing)
        .unwrap_or(DROPLET_COUNT_LANDING);

    for (entity, state, transform, fruit_type) in changed_fruits.iter() {
        if *state != FruitSpawnState::Landed {
            continue;
        }

        let count = scale_count_by_fruit(base_count, *fruit_type);
        let pos = transform.translation.truncate();
        let fruit_color = fruit_type.placeholder_color();
        let color = resolve_droplet_color(droplet_cfg, fruit_color);
        spawn_droplets(
            &mut commands,
            pos,
            color,
            count,
            droplet_cfg,
            unit_quad.as_deref(),
            &mut soft_circle_materials,
        );

        // Add landing bounce (squash-and-stretch) to the fruit
        commands
            .entity(entity)
            .insert(SquashStretchAnimation::for_landing(bounce_cfg));
    }
}

/// Advances water droplets: applies gravity, wall bounce, alpha fade, despawn
///
/// Each frame:
/// 1. Integrates velocity (with gravity) into position
/// 2. Bounces off container walls (X bounds and Y floor)
/// 3. Fades out the `WaterDroplet::alpha` over the droplet's lifetime
/// 4. Despawns droplets whose lifetime has expired
///
/// Separate sync systems ([`sync_sprite_droplet_alpha`],
/// [`sync_material_droplet_alpha`]) apply the computed alpha to the visual
/// representation.
pub fn update_water_droplets(
    mut commands: Commands,
    mut droplets: Query<(Entity, &mut WaterDroplet, &mut Transform)>,
    time: Res<Time>,
    physics: PhysicsParams<'_>,
    droplet: DropletParams<'_>,
) {
    let dt = time.delta_secs();

    let (half_w, half_h) = physics
        .get()
        .map(|cfg| (cfg.container_width / 2.0, cfg.container_height / 2.0))
        .unwrap_or((300.0, 400.0));

    let droplet_cfg = droplet.get();
    let gravity = droplet_cfg.map(|c| c.gravity).unwrap_or(DROPLET_GRAVITY);
    let bounce_damping = droplet_cfg
        .map(|c| c.bounce_damping)
        .unwrap_or(DROPLET_BOUNCE_DAMPING);

    for (entity, mut droplet, mut transform) in droplets.iter_mut() {
        // --- Physics integration ---
        droplet.velocity.y += gravity * dt;
        transform.translation.x += droplet.velocity.x * dt;
        transform.translation.y += droplet.velocity.y * dt;

        // --- Wall bouncing ---
        if transform.translation.x < -half_w {
            transform.translation.x = -half_w;
            droplet.velocity.x = droplet.velocity.x.abs() * bounce_damping;
        } else if transform.translation.x > half_w {
            transform.translation.x = half_w;
            droplet.velocity.x = -droplet.velocity.x.abs() * bounce_damping;
        }

        if transform.translation.y < -half_h {
            transform.translation.y = -half_h;
            droplet.velocity.y = droplet.velocity.y.abs() * bounce_damping;
        }

        // --- Alpha fade ---
        droplet.lifetime += dt;
        let progress = if droplet.max_lifetime <= 0.0 {
            1.0
        } else {
            (droplet.lifetime / droplet.max_lifetime).clamp(0.0, 1.0)
        };
        droplet.alpha = (1.0 - progress) * DROPLET_INITIAL_ALPHA;

        // --- Despawn when lifetime expires ---
        if droplet.lifetime >= droplet.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// Syncs `WaterDroplet::alpha` to the `Sprite` colour (fallback / headless mode).
pub fn sync_sprite_droplet_alpha(
    mut droplets: Query<(&WaterDroplet, &mut Sprite), Without<MeshMaterial2d<SoftCircleMaterial>>>,
) {
    for (droplet, mut sprite) in droplets.iter_mut() {
        sprite.color = sprite.color.with_alpha(droplet.alpha);
    }
}

/// Syncs `WaterDroplet::alpha` to [`SoftCircleMaterial`] (shader mode).
pub fn sync_material_droplet_alpha(
    droplets: Query<(&WaterDroplet, &MeshMaterial2d<SoftCircleMaterial>)>,
    mut materials: Option<ResMut<Assets<SoftCircleMaterial>>>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_droplet_count_constants() {
        assert_eq!(DROPLET_COUNT_MERGE, 12);
        assert_eq!(DROPLET_COUNT_LANDING, 5);
    }

    #[test]
    fn test_scale_count_cherry_uses_base() {
        // Cherry is stage 0 → multiplier 1.0 → count equals base
        assert_eq!(
            scale_count_by_fruit(12, crate::fruit::FruitType::Cherry),
            12
        );
    }

    #[test]
    fn test_scale_count_watermelon_is_three_times_base() {
        // Watermelon is stage 10 → multiplier 3.0 → count = base * 3
        assert_eq!(
            scale_count_by_fruit(12, crate::fruit::FruitType::Watermelon),
            36
        );
    }

    #[test]
    fn test_scale_count_increases_with_stage() {
        let fruits = [
            crate::fruit::FruitType::Cherry,
            crate::fruit::FruitType::Grape,
            crate::fruit::FruitType::Apple,
            crate::fruit::FruitType::Peach,
            crate::fruit::FruitType::Watermelon,
        ];
        let counts: Vec<u32> = fruits
            .iter()
            .map(|f| scale_count_by_fruit(12, *f))
            .collect();
        for window in counts.windows(2) {
            assert!(
                window[1] >= window[0],
                "droplet count must be non-decreasing along the evolution chain"
            );
        }
    }

    #[test]
    fn test_scale_count_never_zero() {
        for fruit in [
            crate::fruit::FruitType::Cherry,
            crate::fruit::FruitType::Watermelon,
        ] {
            assert!(
                scale_count_by_fruit(1, fruit) >= 1,
                "scaled count must be at least 1"
            );
        }
    }

    #[test]
    fn test_droplet_speed_range_valid() {
        assert!(
            DROPLET_MIN_SPEED < DROPLET_MAX_SPEED,
            "Min speed must be less than max speed"
        );
    }

    #[test]
    fn test_droplet_lifetime_range_valid() {
        assert!(
            DROPLET_LIFETIME_MIN < DROPLET_LIFETIME_MAX,
            "Min lifetime must be less than max lifetime"
        );
    }

    #[test]
    fn test_spawn_merge_droplets_spawns_correct_count() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<FruitMergeEvent>();
        app.add_systems(Update, spawn_merge_droplets);

        app.world_mut().write_message(FruitMergeEvent {
            entity1: Entity::PLACEHOLDER,
            entity2: Entity::PLACEHOLDER,
            fruit_type: crate::fruit::FruitType::Cherry,
            position: Vec2::ZERO,
        });

        app.update();

        let count = app
            .world_mut()
            .query::<&WaterDroplet>()
            .iter(app.world())
            .count();

        // Cherry is stage 0 → scale 1.0 → count equals the base constant
        assert_eq!(
            count, DROPLET_COUNT_MERGE as usize,
            "Cherry (stage 0) should spawn exactly DROPLET_COUNT_MERGE droplets"
        );
    }

    #[test]
    fn test_spawn_merge_droplets_velocity_in_range() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<FruitMergeEvent>();
        app.add_systems(Update, spawn_merge_droplets);

        app.world_mut().write_message(FruitMergeEvent {
            entity1: Entity::PLACEHOLDER,
            entity2: Entity::PLACEHOLDER,
            fruit_type: crate::fruit::FruitType::Cherry,
            position: Vec2::ZERO,
        });

        app.update();

        for droplet in app.world_mut().query::<&WaterDroplet>().iter(app.world()) {
            let speed = droplet.velocity.length();
            assert!(
                speed >= DROPLET_MIN_SPEED,
                "Speed {speed} below minimum {DROPLET_MIN_SPEED}"
            );
            assert!(
                speed <= DROPLET_MAX_SPEED,
                "Speed {speed} above maximum {DROPLET_MAX_SPEED}"
            );
        }
    }

    #[test]
    fn test_spawn_merge_droplets_lifetime_in_range() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<FruitMergeEvent>();
        app.add_systems(Update, spawn_merge_droplets);

        app.world_mut().write_message(FruitMergeEvent {
            entity1: Entity::PLACEHOLDER,
            entity2: Entity::PLACEHOLDER,
            fruit_type: crate::fruit::FruitType::Apple,
            position: Vec2::new(10.0, -50.0),
        });

        app.update();

        for droplet in app.world_mut().query::<&WaterDroplet>().iter(app.world()) {
            assert!(
                droplet.max_lifetime >= DROPLET_LIFETIME_MIN,
                "Lifetime {} below minimum {}",
                droplet.max_lifetime,
                DROPLET_LIFETIME_MIN
            );
            assert!(
                droplet.max_lifetime <= DROPLET_LIFETIME_MAX,
                "Lifetime {} above maximum {}",
                droplet.max_lifetime,
                DROPLET_LIFETIME_MAX
            );
        }
    }
}
