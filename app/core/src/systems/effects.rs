//! Visual effects systems
//!
//! This module implements transient visual effects applied to game entities.
//! Sub-modules provide squash-and-stretch bounce, water droplet particles,
//! and flash effects for merges and landings.

pub mod bounce;
pub mod droplet;
pub mod flash;
pub mod shake;

use bevy::prelude::*;

/// Scale pop-in animation played on a newly merged fruit
///
/// Added to a fruit entity immediately after it is spawned by the merge system.
/// The entity starts at scale 0 and grows to its natural size (1.0) over
/// `duration` seconds using an ease-out quadratic curve, giving a satisfying
/// "pop" feel without the complexity of a full animation framework.
///
/// # Lifecycle
///
/// The component is inserted by `handle_fruit_merge` and removed automatically
/// by `animate_merge_scale` once the animation completes.
#[derive(Component, Debug, Clone)]
pub struct MergeAnimation {
    /// Elapsed time in seconds since the animation started
    pub elapsed: f32,
    /// Total duration of the animation in seconds
    pub duration: f32,
}

impl MergeAnimation {
    /// Creates a new animation with the given duration
    pub fn new(duration: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration,
        }
    }

    /// Default animation duration (0.25 seconds)
    pub const DEFAULT_DURATION: f32 = 0.25;

    /// Returns the normalized progress in `[0.0, 1.0]`
    ///
    /// Returns `1.0` immediately when `duration <= 0.0` to avoid `NaN`
    /// from `0.0 / 0.0` (which `clamp` does not sanitize in Rust).
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            return 1.0;
        }
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// Returns the ease-out quadratic scale factor for the current progress
    ///
    /// Uses `f(t) = t * (2 - t)` which starts fast and decelerates,
    /// giving a natural "pop-in" appearance.
    pub fn scale_factor(&self) -> f32 {
        let t = self.progress();
        t * (2.0 - t)
    }
}

/// Advances `MergeAnimation` components and applies the resulting scale
///
/// Each frame:
/// 1. Increments `elapsed` by `delta_secs`
/// 2. Computes the ease-out scale factor from the current progress
/// 3. Writes the scale to the entity's `Transform`
/// 4. Removes `MergeAnimation` once the animation has finished
pub fn animate_merge_scale(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MergeAnimation, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut anim, mut transform) in query.iter_mut() {
        anim.elapsed += time.delta_secs();

        let scale = anim.scale_factor();
        transform.scale = Vec3::splat(scale);

        if anim.elapsed >= anim.duration {
            // Snap to full size and remove the transient component
            transform.scale = Vec3::ONE;
            commands.entity(entity).remove::<MergeAnimation>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- MergeAnimation unit tests ---

    #[test]
    fn test_merge_animation_initial_state() {
        let anim = MergeAnimation::new(0.25);
        assert_eq!(anim.elapsed, 0.0);
        assert_eq!(anim.duration, 0.25);
        assert_eq!(anim.progress(), 0.0);
        assert_eq!(anim.scale_factor(), 0.0);
    }

    #[test]
    fn test_merge_animation_midpoint() {
        let mut anim = MergeAnimation::new(1.0);
        anim.elapsed = 0.5; // 50% through
        assert!((anim.progress() - 0.5).abs() < f32::EPSILON);
        // ease-out at 0.5: 0.5 * (2 - 0.5) = 0.5 * 1.5 = 0.75
        assert!((anim.scale_factor() - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_merge_animation_complete() {
        let mut anim = MergeAnimation::new(0.25);
        anim.elapsed = 0.25;
        assert_eq!(anim.progress(), 1.0);
        assert_eq!(anim.scale_factor(), 1.0);
    }

    #[test]
    fn test_merge_animation_progress_clamped() {
        let mut anim = MergeAnimation::new(0.25);
        anim.elapsed = 10.0; // Way past the end
        assert_eq!(anim.progress(), 1.0);
        assert_eq!(anim.scale_factor(), 1.0);
    }

    #[test]
    fn test_merge_animation_progress_zero_duration_no_nan() {
        // duration == 0 would cause 0/0 = NaN without the guard
        let anim = MergeAnimation::new(0.0);
        let p = anim.progress();
        assert!(
            p.is_finite(),
            "progress() must not return NaN or Inf when duration is 0"
        );
        assert_eq!(p, 1.0, "zero-duration animation should be immediately done");
        assert_eq!(anim.scale_factor(), 1.0);
    }

    #[test]
    fn test_merge_animation_scale_increases_monotonically() {
        let duration = 1.0_f32;
        let steps = 10;
        let mut prev = -1.0_f32;
        for i in 0..=steps {
            let mut anim = MergeAnimation::new(duration);
            anim.elapsed = (i as f32 / steps as f32) * duration;
            let scale = anim.scale_factor();
            assert!(
                scale >= prev,
                "Scale should be non-decreasing: step {} scale {} < prev {}",
                i,
                scale,
                prev
            );
            prev = scale;
        }
    }

    // --- animate_merge_scale system tests ---

    #[test]
    fn test_animate_merge_scale_grows_entity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, animate_merge_scale);

        // Spawn an entity with a MergeAnimation (0 elapsed, 1.0s duration)
        let entity = app
            .world_mut()
            .spawn((MergeAnimation::new(1.0), Transform::default()))
            .id();

        // Run twice: first update initialises the frame clock (delta = 0),
        // second update has a non-zero delta so elapsed advances and scale grows.
        app.update();
        app.update();

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert!(
            transform.scale.x > 0.0,
            "Scale should be positive after time has advanced"
        );
    }

    #[test]
    fn test_animate_merge_scale_removes_component_when_done() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, animate_merge_scale);

        // Create animation that is already past its duration
        let mut anim = MergeAnimation::new(0.25);
        anim.elapsed = 0.25; // Already at 100%

        let entity = app.world_mut().spawn((anim, Transform::default())).id();

        app.update();

        // MergeAnimation should have been removed
        assert!(
            app.world().get::<MergeAnimation>(entity).is_none(),
            "MergeAnimation component should be removed after animation completes"
        );

        // Scale should be snapped to 1.0
        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_eq!(transform.scale, Vec3::ONE);
    }
}
