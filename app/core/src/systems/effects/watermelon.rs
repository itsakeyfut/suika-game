//! Special visual effects for Watermelon merge events
//!
//! When two Watermelons merge (the final evolution stage), both fruits disappear
//! and this module fires an over-the-top celebration effect consisting of:
//!
//! - **Explosion ring**: a large sprite that expands outward and fades, giving a
//!   shockwave feel.
//! - **Burst particles**: dozens of short-lived sprites in watermelon colours
//!   (green rind, red flesh, white sparkle) that fly outward with gravity.
//! - **Extra camera trauma**: directly adds to [`CameraShake`] to guarantee the
//!   camera shake is at maximum regardless of the regular `add_camera_shake` result.
//!
//! All parameters are read from `assets/config/effects/watermelon.ron` when
//! loaded, falling back to the `DEFAULT_*` constants otherwise.

use bevy::prelude::*;
use rand::RngExt;

use crate::config::WatermelonParams;
use crate::events::FruitMergeEvent;
use crate::fruit::FruitType;
use crate::systems::effects::shake::CameraShake;

// ---------------------------------------------------------------------------
// Constants (fallbacks when watermelon.ron is not yet loaded)
// ---------------------------------------------------------------------------

/// Fallback: duration of the explosion ring animation in seconds
pub const DEFAULT_RING_DURATION: f32 = 0.7;
/// Fallback: initial diameter of the explosion ring in pixels
pub const DEFAULT_RING_INITIAL_DIAMETER: f32 = 160.0;
/// Fallback: how many times larger the ring grows relative to initial diameter
pub const DEFAULT_RING_EXPAND_MULTIPLIER: f32 = 5.0;
/// Fallback: starting alpha of the explosion ring sprite
pub const DEFAULT_RING_INITIAL_ALPHA: f32 = 0.75;

/// Fallback: number of burst particles spawned on Watermelon merge
pub const DEFAULT_BURST_COUNT: u32 = 48;
/// Fallback: minimum speed of burst particles in pixels/second
pub const DEFAULT_BURST_MIN_SPEED: f32 = 150.0;
/// Fallback: maximum speed of burst particles in pixels/second
pub const DEFAULT_BURST_MAX_SPEED: f32 = 500.0;
/// Fallback: visual diameter of each burst particle in pixels
pub const DEFAULT_BURST_PARTICLE_SIZE: f32 = 5.0;
/// Fallback: maximum lifetime of burst particles in seconds
pub const DEFAULT_BURST_LIFETIME: f32 = 0.9;
/// Gravity applied to burst particles (pixels/s², negative = downward)
const BURST_GRAVITY: f32 = -500.0;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Expanding shockwave ring spawned when two Watermelons merge
///
/// The sprite starts at `initial_size`, expands to `final_size` over `duration`
/// seconds, and fades from `initial_alpha` → 0. Despawned automatically when done.
#[derive(Component, Debug)]
pub struct WatermelonExplosionRing {
    /// Elapsed time in seconds
    pub elapsed: f32,
    /// Total animation duration in seconds
    pub duration: f32,
    /// Starting sprite size (diameter) in pixels
    pub initial_size: f32,
    /// Final sprite size (diameter) in pixels
    pub final_size: f32,
    /// Starting alpha value
    pub initial_alpha: f32,
}

/// Short-lived burst particle spawned when two Watermelons merge
///
/// Flies outward from the merge point with random velocity, affected by gravity,
/// and fades out over its lifetime.
#[derive(Component, Debug)]
pub struct WatermelonBurstParticle {
    /// Current velocity in pixels/second
    pub velocity: Vec2,
    /// Elapsed lifetime in seconds
    pub lifetime: f32,
    /// Total lifetime in seconds before despawn
    pub max_lifetime: f32,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns special effects for Watermelon-related merge events
///
/// Triggers on **two** distinct events:
/// - [`FruitType::Melon`] merge → a Watermelon is **born** (two Melons produce one Watermelon)
/// - [`FruitType::Watermelon`] merge → a Watermelon **vanishes** (two Watermelons disappear)
///
/// For each matched event it:
/// 1. Adds maximum camera trauma (ensuring a dramatic shake)
/// 2. Spawns an expanding [`WatermelonExplosionRing`]
/// 3. Spawns a burst of [`WatermelonBurstParticle`] in watermelon colours
///
/// Values come from `assets/config/effects/watermelon.ron` with
/// `DEFAULT_*` constants as fallback.
pub fn spawn_watermelon_effects(
    mut commands: Commands,
    mut merge_events: MessageReader<FruitMergeEvent>,
    mut shake_query: Query<&mut CameraShake>,
    config: WatermelonParams<'_>,
) {
    let cfg = config.get();

    let ring_duration = cfg
        .map(|c| c.ring_duration)
        .unwrap_or(DEFAULT_RING_DURATION);
    let ring_initial_diameter = cfg
        .map(|c| c.ring_initial_diameter)
        .unwrap_or(DEFAULT_RING_INITIAL_DIAMETER);
    let ring_expand = cfg
        .map(|c| c.ring_expand_multiplier)
        .unwrap_or(DEFAULT_RING_EXPAND_MULTIPLIER);
    let ring_alpha = cfg
        .map(|c| c.ring_initial_alpha)
        .unwrap_or(DEFAULT_RING_INITIAL_ALPHA);
    let burst_count = cfg.map(|c| c.burst_count).unwrap_or(DEFAULT_BURST_COUNT);
    let burst_min = cfg
        .map(|c| c.burst_min_speed)
        .unwrap_or(DEFAULT_BURST_MIN_SPEED);
    let burst_max = cfg
        .map(|c| c.burst_max_speed)
        .unwrap_or(DEFAULT_BURST_MAX_SPEED);
    let particle_size = cfg
        .map(|c| c.burst_particle_size)
        .unwrap_or(DEFAULT_BURST_PARTICLE_SIZE);
    let burst_lifetime = cfg
        .map(|c| c.burst_lifetime)
        .unwrap_or(DEFAULT_BURST_LIFETIME);

    for event in merge_events.read() {
        // Trigger on Melon merge (Watermelon is born) or Watermelon merge (Watermelon vanishes)
        let triggers =
            event.fruit_type == FruitType::Melon || event.fruit_type == FruitType::Watermelon;
        if !triggers {
            continue;
        }

        // TODO: Differentiate effects between birth and vanish events.
        //   - Birth  (Melon merge)      : celebratory ring + burst (current)
        //   - Vanish (Watermelon merge) : larger ring, more particles, distinct colour scheme
        let pos = event.position;

        // Max camera trauma ensures a dramatic shake on every Watermelon merge
        if let Ok(mut shake) = shake_query.single_mut() {
            shake.add_trauma(1.0);
        }

        // Expanding shockwave ring at Z=6 (above fruits/local-flash, below screen-flash)
        let final_size = ring_initial_diameter * ring_expand;
        commands.spawn((
            WatermelonExplosionRing {
                elapsed: 0.0,
                duration: ring_duration,
                initial_size: ring_initial_diameter,
                final_size,
                initial_alpha: ring_alpha,
            },
            Sprite {
                // Watermelon green ring
                color: Color::srgba(0.18, 0.78, 0.25, ring_alpha),
                custom_size: Some(Vec2::splat(ring_initial_diameter)),
                ..default()
            },
            Transform::from_translation(pos.extend(6.0)),
        ));

        // Burst particles
        let safe_max = if burst_max > burst_min {
            burst_max
        } else {
            burst_min + 1.0
        };

        let mut rng = rand::rng();
        for i in 0..burst_count {
            let angle = rng.random_range(0.0_f32..std::f32::consts::TAU);
            let speed = rng.random_range(burst_min..safe_max);
            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
            let lifetime = rng.random_range((burst_lifetime * 0.5)..burst_lifetime);

            // Cycle through watermelon colours: green rind → red flesh → white sparkle
            let color = match i % 3 {
                0 => Color::srgba(0.18, 0.78, 0.25, 1.0), // green rind
                1 => Color::srgba(0.92, 0.18, 0.18, 1.0), // red flesh
                _ => Color::srgba(1.00, 0.97, 0.97, 1.0), // white sparkle
            };

            commands.spawn((
                WatermelonBurstParticle {
                    velocity,
                    lifetime: 0.0,
                    max_lifetime: lifetime,
                },
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(particle_size)),
                    ..default()
                },
                Transform::from_translation(pos.extend(7.0)),
            ));
        }
    }
}

/// Animates the [`WatermelonExplosionRing`]: expands and fades out, then despawns
pub fn animate_watermelon_explosion(
    mut commands: Commands,
    mut rings: Query<(Entity, &mut WatermelonExplosionRing, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut ring, mut sprite) in rings.iter_mut() {
        ring.elapsed += time.delta_secs();

        if ring.elapsed >= ring.duration {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = (ring.elapsed / ring.duration).clamp(0.0, 1.0);
        let size = ring.initial_size + (ring.final_size - ring.initial_size) * progress;
        let alpha = ring.initial_alpha * (1.0 - progress);

        sprite.custom_size = Some(Vec2::splat(size));
        sprite.color = sprite.color.with_alpha(alpha);
    }
}

/// Updates [`WatermelonBurstParticle`] entities: applies gravity, fades alpha, despawns
pub fn update_watermelon_burst_particles(
    mut commands: Commands,
    mut particles: Query<(
        Entity,
        &mut WatermelonBurstParticle,
        &mut Transform,
        &mut Sprite,
    )>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut particle, mut transform, mut sprite) in particles.iter_mut() {
        // Integrate velocity (gravity pulls downward)
        particle.velocity.y += BURST_GRAVITY * dt;
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Fade alpha linearly over lifetime
        particle.lifetime += dt;
        let progress = if particle.max_lifetime <= 0.0 {
            1.0
        } else {
            (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0)
        };
        sprite.color = sprite.color.with_alpha(1.0 - progress);

        if particle.lifetime >= particle.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_expand_multiplier_gives_larger_final_size() {
        let initial = DEFAULT_RING_INITIAL_DIAMETER;
        let final_size = initial * DEFAULT_RING_EXPAND_MULTIPLIER;
        assert!(
            final_size > initial,
            "Final ring size ({final_size}) must be larger than initial ({initial})"
        );
    }

    #[test]
    fn test_ring_alpha_reaches_zero_at_duration() {
        let ring = WatermelonExplosionRing {
            elapsed: DEFAULT_RING_DURATION,
            duration: DEFAULT_RING_DURATION,
            initial_size: DEFAULT_RING_INITIAL_DIAMETER,
            final_size: DEFAULT_RING_INITIAL_DIAMETER * DEFAULT_RING_EXPAND_MULTIPLIER,
            initial_alpha: DEFAULT_RING_INITIAL_ALPHA,
        };
        let progress = (ring.elapsed / ring.duration).clamp(0.0, 1.0);
        let alpha = ring.initial_alpha * (1.0 - progress);
        assert!(
            alpha.abs() < f32::EPSILON,
            "Ring alpha should reach 0 at end of duration, got {alpha}"
        );
    }

    #[test]
    fn test_burst_speed_range_valid() {
        assert!(
            DEFAULT_BURST_MIN_SPEED < DEFAULT_BURST_MAX_SPEED,
            "Burst min speed must be less than max speed"
        );
    }

    #[test]
    fn test_spawn_watermelon_effects_ignores_unrelated_fruits() {
        // Verify that unrelated merge events (e.g. Cherry) produce no explosion rings
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<FruitMergeEvent>();
        app.add_systems(Update, spawn_watermelon_effects);

        app.world_mut().write_message(FruitMergeEvent {
            entity1: Entity::PLACEHOLDER,
            entity2: Entity::PLACEHOLDER,
            fruit_type: FruitType::Cherry,
            position: Vec2::ZERO,
        });

        app.update();

        let ring_count = app
            .world_mut()
            .query::<&WatermelonExplosionRing>()
            .iter(app.world())
            .count();

        assert_eq!(
            ring_count, 0,
            "Cherry merge must not spawn an explosion ring"
        );
    }

    #[test]
    fn test_spawn_watermelon_effects_triggers_for_watermelon_vanish() {
        // Watermelon + Watermelon → both disappear; effects must fire
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<FruitMergeEvent>();
        app.add_systems(Update, spawn_watermelon_effects);

        app.world_mut().write_message(FruitMergeEvent {
            entity1: Entity::PLACEHOLDER,
            entity2: Entity::PLACEHOLDER,
            fruit_type: FruitType::Watermelon,
            position: Vec2::ZERO,
        });

        app.update();

        let ring_count = app
            .world_mut()
            .query::<&WatermelonExplosionRing>()
            .iter(app.world())
            .count();

        assert_eq!(
            ring_count, 1,
            "Watermelon merge must spawn exactly one ring"
        );

        let particle_count = app
            .world_mut()
            .query::<&WatermelonBurstParticle>()
            .iter(app.world())
            .count();

        assert_eq!(
            particle_count, DEFAULT_BURST_COUNT as usize,
            "Watermelon merge must spawn exactly DEFAULT_BURST_COUNT particles"
        );
    }

    #[test]
    fn test_spawn_watermelon_effects_triggers_for_melon_birth() {
        // Melon + Melon → Watermelon is born; effects must also fire
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<FruitMergeEvent>();
        app.add_systems(Update, spawn_watermelon_effects);

        app.world_mut().write_message(FruitMergeEvent {
            entity1: Entity::PLACEHOLDER,
            entity2: Entity::PLACEHOLDER,
            fruit_type: FruitType::Melon,
            position: Vec2::ZERO,
        });

        app.update();

        let ring_count = app
            .world_mut()
            .query::<&WatermelonExplosionRing>()
            .iter(app.world())
            .count();

        assert_eq!(
            ring_count, 1,
            "Melon merge (Watermelon birth) must spawn exactly one ring"
        );

        let particle_count = app
            .world_mut()
            .query::<&WatermelonBurstParticle>()
            .iter(app.world())
            .count();

        assert_eq!(
            particle_count, DEFAULT_BURST_COUNT as usize,
            "Melon merge (Watermelon birth) must spawn exactly DEFAULT_BURST_COUNT particles"
        );
    }

    #[test]
    fn test_animate_watermelon_explosion_despawns_when_done() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, animate_watermelon_explosion);

        let ring = WatermelonExplosionRing {
            elapsed: DEFAULT_RING_DURATION, // already finished
            duration: DEFAULT_RING_DURATION,
            initial_size: DEFAULT_RING_INITIAL_DIAMETER,
            final_size: DEFAULT_RING_INITIAL_DIAMETER * DEFAULT_RING_EXPAND_MULTIPLIER,
            initial_alpha: DEFAULT_RING_INITIAL_ALPHA,
        };

        let entity = app
            .world_mut()
            .spawn((ring, Sprite::default(), Transform::default()))
            .id();

        app.update();

        assert!(
            app.world().get_entity(entity).is_err(),
            "Ring entity should be despawned when duration is reached"
        );
    }
}
