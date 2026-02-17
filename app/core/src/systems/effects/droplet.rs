//! Water droplet particle system
//!
//! Spawns small water-drop particles on fruit merge and fruit landing.
//! Droplets are affected by gravity and bounce off the container walls,
//! fading out before they despawn.

use bevy::prelude::*;
use rand::RngExt;

use crate::components::{Fruit, FruitSpawnState};
use crate::config::{
    BounceConfig, BounceConfigHandle, DropletConfig, DropletConfigHandle, PhysicsConfig,
    PhysicsConfigHandle,
};
use crate::events::FruitMergeEvent;
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

// --- Component ---

/// Water droplet particle component
///
/// A short-lived sprite that flies out from merge/landing points,
/// bounces off the container walls, and fades out over its lifetime.
#[derive(Component, Debug)]
pub struct WaterDroplet {
    /// Current velocity in pixels/second
    pub velocity: Vec2,
    /// Elapsed lifetime in seconds
    pub lifetime: f32,
    /// Total lifetime in seconds before despawn
    pub max_lifetime: f32,
}

// --- Internal helper ---

/// Spawns `count` droplets radiating from `position` using values from `config`
/// (or falling back to the module constants when `config` is `None`).
fn spawn_droplets(
    commands: &mut Commands,
    position: Vec2,
    color: Color,
    count: u32,
    config: Option<&DropletConfig>,
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

    let mut rng = rand::rng();
    for _ in 0..count {
        let angle = rng.random_range(0.0_f32..std::f32::consts::TAU);
        let speed = rng.random_range(min_speed..max_speed);
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
        let lifetime = rng.random_range(lifetime_min..lifetime_max);

        commands.spawn((
            WaterDroplet {
                velocity,
                lifetime: 0.0,
                max_lifetime: lifetime,
            },
            // TODO: 将来的に Material2d + WGSL フラグメントシェーダーで
            //       ソフトな円形（エッジをブラー）に変更する
            Sprite {
                color,
                custom_size: Some(Vec2::splat(radius * 2.0)),
                ..default()
            },
            Transform::from_translation(position.extend(5.0)),
        ));
    }
}

// --- Systems ---

/// Spawns water droplets on fruit merge events
///
/// Reads `FruitMergeEvent` and spawns `DROPLET_COUNT_MERGE` droplets radiating
/// outward from the merge position, using the merged fruit's placeholder color.
pub fn spawn_merge_droplets(
    mut commands: Commands,
    mut merge_events: MessageReader<FruitMergeEvent>,
    droplet_config_handle: Option<Res<DropletConfigHandle>>,
    droplet_config_assets: Option<Res<Assets<DropletConfig>>>,
) {
    let config = droplet_config_handle
        .as_ref()
        .zip(droplet_config_assets.as_ref())
        .and_then(|(h, a)| a.get(&h.0));
    let count = config.map(|c| c.count_merge).unwrap_or(DROPLET_COUNT_MERGE);
    let color = config
        .map(|c| Color::from(c.color))
        .unwrap_or(DROPLET_COLOR);

    for event in merge_events.read() {
        let fruit_color = event.fruit_type.placeholder_color();
        // Blend the droplet base color toward the fruit color for variety
        let _ = color; // base color available; use fruit color for now
        spawn_droplets(&mut commands, event.position, fruit_color, count, config);
    }
}

/// Spawns water droplets when a fruit transitions to the `Landed` state
///
/// Uses Bevy's change detection (`Changed<FruitSpawnState>`) to detect the
/// moment a falling fruit lands, spawns a small splash, and also inserts the
/// landing impact bounce animation on the fruit itself.
#[allow(clippy::type_complexity)]
pub fn spawn_landing_droplets(
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
    droplet_config_handle: Option<Res<DropletConfigHandle>>,
    droplet_config_assets: Option<Res<Assets<DropletConfig>>>,
    bounce_config_handle: Option<Res<BounceConfigHandle>>,
    bounce_config_assets: Option<Res<Assets<BounceConfig>>>,
) {
    let droplet_cfg = droplet_config_handle
        .as_ref()
        .zip(droplet_config_assets.as_ref())
        .and_then(|(h, a)| a.get(&h.0));
    let bounce_cfg = bounce_config_handle
        .as_ref()
        .zip(bounce_config_assets.as_ref())
        .and_then(|(h, a)| a.get(&h.0));

    let count = droplet_cfg
        .map(|c| c.count_landing)
        .unwrap_or(DROPLET_COUNT_LANDING);

    for (entity, state, transform, fruit_type) in changed_fruits.iter() {
        if *state != FruitSpawnState::Landed {
            continue;
        }

        let pos = transform.translation.truncate();
        let color = fruit_type.placeholder_color();
        spawn_droplets(&mut commands, pos, color, count, droplet_cfg);

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
/// 3. Fades out the sprite alpha over the droplet's lifetime
/// 4. Despawns droplets whose lifetime has expired
pub fn update_water_droplets(
    mut commands: Commands,
    mut droplets: Query<(Entity, &mut WaterDroplet, &mut Transform, &mut Sprite)>,
    time: Res<Time>,
    physics_config_handle: Option<Res<PhysicsConfigHandle>>,
    physics_config_assets: Option<Res<Assets<PhysicsConfig>>>,
    droplet_config_handle: Option<Res<DropletConfigHandle>>,
    droplet_config_assets: Option<Res<Assets<DropletConfig>>>,
) {
    let dt = time.delta_secs();

    let (half_w, half_h) = physics_config_handle
        .as_ref()
        .zip(physics_config_assets.as_ref())
        .and_then(|(h, a)| a.get(&h.0))
        .map(|cfg| (cfg.container_width / 2.0, cfg.container_height / 2.0))
        .unwrap_or((300.0, 400.0));

    let droplet_cfg = droplet_config_handle
        .as_ref()
        .zip(droplet_config_assets.as_ref())
        .and_then(|(h, a)| a.get(&h.0));
    let gravity = droplet_cfg.map(|c| c.gravity).unwrap_or(DROPLET_GRAVITY);
    let bounce_damping = droplet_cfg
        .map(|c| c.bounce_damping)
        .unwrap_or(DROPLET_BOUNCE_DAMPING);

    for (entity, mut droplet, mut transform, mut sprite) in droplets.iter_mut() {
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
        let progress = (droplet.lifetime / droplet.max_lifetime).clamp(0.0, 1.0);
        let alpha = (1.0 - progress) * 0.85;
        sprite.color = DROPLET_COLOR.with_alpha(alpha);

        // --- Despawn when lifetime expires ---
        if droplet.lifetime >= droplet.max_lifetime {
            commands.entity(entity).despawn();
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

        assert_eq!(
            count, DROPLET_COUNT_MERGE as usize,
            "Should spawn exactly DROPLET_COUNT_MERGE droplets"
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
