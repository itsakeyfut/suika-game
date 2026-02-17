//! Squash & Stretch bounce animation system
//!
//! Provides a physically-plausible squash-and-stretch animation for fruits.
//! Two modes are supported:
//! - `SpawnIn`: merge pop-in (scale 0→1 with bounce overshoot)
//! - `Impact`: landing impact (scale stays at 1, squashes then recovers)

use bevy::prelude::*;

use crate::config::BounceConfig;

/// The mode of the bounce animation
#[derive(Debug, Clone, PartialEq)]
pub enum BounceMode {
    /// Merge spawn-in: scale grows from 0→1 with a spring overshoot
    SpawnIn,
    /// Landing impact: scale stays at ~1 but squashes then recovers
    Impact,
}

/// Squash & Stretch spring animation component
///
/// Added to a fruit entity to play a squash-and-stretch bounce.
/// Automatically removed when the animation settles.
///
/// # Lifecycle
///
/// Inserted by `handle_fruit_merge` (SpawnIn) or `spawn_landing_droplets` (Impact).
/// Removed automatically by `animate_squash_stretch` once the spring settles.
#[derive(Component, Debug, Clone)]
pub struct SquashStretchAnimation {
    /// Elapsed time in seconds since animation started
    pub elapsed: f32,
    /// Animation mode
    pub mode: BounceMode,
    /// Spring oscillation amplitude
    pub amplitude: f32,
    /// Spring oscillation frequency (radians/second)
    pub frequency: f32,
    /// Damping coefficient (higher = settles faster)
    pub damping: f32,
    /// Deformation magnitude below which the animation is considered settled
    pub settle_threshold: f32,
    /// Minimum elapsed time (s) before the settle check activates
    pub settle_min_elapsed: f32,
}

// Default parameter constants used as fallback when config is not yet loaded
const DEFAULT_MERGE_AMPLITUDE: f32 = 0.3;
const DEFAULT_MERGE_FREQUENCY: f32 = 18.0;
const DEFAULT_MERGE_DAMPING: f32 = 6.0;
const DEFAULT_LANDING_AMPLITUDE: f32 = 0.18;
const DEFAULT_LANDING_FREQUENCY: f32 = 22.0;
const DEFAULT_LANDING_DAMPING: f32 = 9.0;
const DEFAULT_SETTLE_THRESHOLD: f32 = 0.01;
const DEFAULT_SETTLE_MIN_ELAPSED: f32 = 0.3;

impl SquashStretchAnimation {
    /// Squash-stretch for merge spawn-in
    ///
    /// Parameters are read from `BounceConfig` if provided; otherwise falls back
    /// to built-in defaults so the animation works even before assets are loaded.
    pub fn for_merge(config: Option<&BounceConfig>) -> Self {
        let (amplitude, frequency, damping, settle_threshold, settle_min_elapsed) = config
            .map(|c| {
                (
                    c.merge_amplitude,
                    c.merge_frequency,
                    c.merge_damping,
                    c.settle_threshold,
                    c.settle_min_elapsed,
                )
            })
            .unwrap_or((
                DEFAULT_MERGE_AMPLITUDE,
                DEFAULT_MERGE_FREQUENCY,
                DEFAULT_MERGE_DAMPING,
                DEFAULT_SETTLE_THRESHOLD,
                DEFAULT_SETTLE_MIN_ELAPSED,
            ));
        Self {
            elapsed: 0.0,
            mode: BounceMode::SpawnIn,
            amplitude,
            frequency,
            damping,
            settle_threshold,
            settle_min_elapsed,
        }
    }

    /// Squash-stretch for fruit landing impact
    ///
    /// Parameters are read from `BounceConfig` if provided; otherwise falls back
    /// to built-in defaults so the animation works even before assets are loaded.
    pub fn for_landing(config: Option<&BounceConfig>) -> Self {
        let (amplitude, frequency, damping, settle_threshold, settle_min_elapsed) = config
            .map(|c| {
                (
                    c.landing_amplitude,
                    c.landing_frequency,
                    c.landing_damping,
                    c.settle_threshold,
                    c.settle_min_elapsed,
                )
            })
            .unwrap_or((
                DEFAULT_LANDING_AMPLITUDE,
                DEFAULT_LANDING_FREQUENCY,
                DEFAULT_LANDING_DAMPING,
                DEFAULT_SETTLE_THRESHOLD,
                DEFAULT_SETTLE_MIN_ELAPSED,
            ));
        Self {
            elapsed: 0.0,
            mode: BounceMode::Impact,
            amplitude,
            frequency,
            damping,
            settle_threshold,
            settle_min_elapsed,
        }
    }

    /// Computes the deformation value at the current elapsed time
    ///
    /// `deform(t) = amplitude × sin(freq × t) × exp(-damping × t)`
    fn deform(&self) -> f32 {
        self.amplitude
            * (self.frequency * self.elapsed).sin()
            * (-self.damping * self.elapsed).exp()
    }

    /// Returns true when the animation has settled enough to remove
    ///
    /// Uses the threshold stored on the animation component, which is set from
    /// `BounceConfig` at construction time (or falls back to the default).
    pub fn is_settled(&self) -> bool {
        self.deform().abs() < self.settle_threshold && self.elapsed > self.settle_min_elapsed
    }

    /// Computes the (scale_x, scale_y) pair for the current state
    ///
    /// Volume is approximately conserved: stretching Y compresses X by half
    /// the deformation amount.
    pub fn scales(&self) -> (f32, f32) {
        let d = self.deform();
        match self.mode {
            BounceMode::SpawnIn => {
                // base grows 0→1 via exponential approach
                let base = 1.0 - (-12.0 * self.elapsed).exp();
                let scale_y = (base + d).max(0.0);
                let scale_x = (base - d * 0.5).max(0.0);
                (scale_x, scale_y)
            }
            BounceMode::Impact => {
                // −d so that positive deform squashes Y (first half-wave squashes)
                let scale_y = (1.0 - d).max(0.0);
                let scale_x = (1.0 + d * 0.5).max(0.0);
                (scale_x, scale_y)
            }
        }
    }
}

/// Advances `SquashStretchAnimation` components and applies the computed scale
///
/// Each frame:
/// 1. Increments `elapsed` by `delta_secs`
/// 2. Computes (scale_x, scale_y) for the current state
/// 3. Writes the scale to the entity's `Transform`
/// 4. Removes `SquashStretchAnimation` once the spring has settled
pub fn animate_squash_stretch(
    mut commands: Commands,
    mut query: Query<(Entity, &mut SquashStretchAnimation, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut anim, mut transform) in query.iter_mut() {
        anim.elapsed += time.delta_secs();

        if anim.is_settled() {
            transform.scale = Vec3::ONE;
            commands.entity(entity).remove::<SquashStretchAnimation>();
            continue;
        }

        let (sx, sy) = anim.scales();
        transform.scale = Vec3::new(sx, sy, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_in_initial_scale_near_zero() {
        let anim = SquashStretchAnimation::for_merge(None);
        let (sx, sy) = anim.scales();
        // At t=0: base = 1 - exp(0) = 0, deform = amplitude * sin(0) * 1 = 0
        assert!(sx.abs() < 0.001, "scale_x should be ~0 at start, got {sx}");
        assert!(sy.abs() < 0.001, "scale_y should be ~0 at start, got {sy}");
    }

    #[test]
    fn test_spawn_in_scale_approaches_one() {
        let mut anim = SquashStretchAnimation::for_merge(None);
        anim.elapsed = 2.0; // Well past settling time
        let (sx, sy) = anim.scales();
        assert!(
            (sx - 1.0).abs() < 0.05,
            "scale_x should approach 1.0 after settling, got {sx}"
        );
        assert!(
            (sy - 1.0).abs() < 0.05,
            "scale_y should approach 1.0 after settling, got {sy}"
        );
    }

    #[test]
    fn test_impact_scale_starts_near_one() {
        let anim = SquashStretchAnimation::for_landing(None);
        let (sx, sy) = anim.scales();
        // At t=0: deform = 0, so both scales = 1.0
        assert!(
            (sx - 1.0).abs() < 0.001,
            "scale_x should be 1.0 at t=0, got {sx}"
        );
        assert!(
            (sy - 1.0).abs() < 0.001,
            "scale_y should be 1.0 at t=0, got {sy}"
        );
    }

    #[test]
    fn test_impact_scale_squashes_then_recovers() {
        let mut anim = SquashStretchAnimation::for_landing(None);

        // At peak squash (roughly t = π / (2 * freq)), scale_y < 1
        anim.elapsed = std::f32::consts::PI / (2.0 * anim.frequency);
        let (_, sy_squash) = anim.scales();
        assert!(
            sy_squash < 1.0,
            "scale_y should be squashed below 1.0 at peak, got {sy_squash}"
        );

        // After settling, both scales should be ~1
        anim.elapsed = 2.0;
        let (sx_settled, sy_settled) = anim.scales();
        assert!(
            (sx_settled - 1.0).abs() < 0.05,
            "scale_x should recover to 1.0, got {sx_settled}"
        );
        assert!(
            (sy_settled - 1.0).abs() < 0.05,
            "scale_y should recover to 1.0, got {sy_settled}"
        );
    }

    #[test]
    fn test_is_settled_false_at_start() {
        let anim = SquashStretchAnimation::for_merge(None);
        assert!(
            !anim.is_settled(),
            "should not be settled immediately after creation"
        );
    }

    #[test]
    fn test_is_settled_true_after_long_time() {
        let mut anim = SquashStretchAnimation::for_merge(None);
        anim.elapsed = 5.0;
        assert!(anim.is_settled(), "should be settled after a long time");
    }

    #[test]
    fn test_scales_are_non_negative() {
        for mode in [BounceMode::SpawnIn, BounceMode::Impact] {
            let base = SquashStretchAnimation::for_merge(None);
            let mut anim = SquashStretchAnimation {
                mode,
                ..base.clone()
            };
            for i in 0..100 {
                anim.elapsed = i as f32 * 0.01;
                let (sx, sy) = anim.scales();
                assert!(sx >= 0.0, "scale_x negative at t={}", anim.elapsed);
                assert!(sy >= 0.0, "scale_y negative at t={}", anim.elapsed);
            }
        }
    }

    #[test]
    fn test_system_removes_component_when_settled() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, animate_squash_stretch);

        let mut anim = SquashStretchAnimation::for_merge(None);
        anim.elapsed = 5.0; // Pre-settled

        let entity = app.world_mut().spawn((anim, Transform::default())).id();
        app.update();

        assert!(
            app.world().get::<SquashStretchAnimation>(entity).is_none(),
            "SquashStretchAnimation should be removed when settled"
        );
        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_eq!(transform.scale, Vec3::ONE, "scale should be snapped to 1.0");
    }

    #[test]
    fn test_system_updates_scale_while_animating() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, animate_squash_stretch);

        let entity = app
            .world_mut()
            .spawn((
                SquashStretchAnimation::for_merge(None),
                Transform::default(),
            ))
            .id();

        app.update();
        app.update();

        assert!(app.world().get::<SquashStretchAnimation>(entity).is_some());
    }
}
