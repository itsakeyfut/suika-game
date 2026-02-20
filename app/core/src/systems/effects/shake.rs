//! Camera shake effect system
//!
//! Implements a trauma-based camera shake triggered by fruit merge events.
//! The shake intensity scales with the size of the merged fruit, giving
//! small merges a subtle jolt and large merges (Watermelon) a dramatic shake.
//!
//! # Algorithm
//!
//! Uses the "trauma" pattern:
//! 1. Each merge event adds `trauma` (0–1) proportional to fruit size
//! 2. Every frame, `trauma` decays at [`DEFAULT_SHAKE_DECAY`] per second
//! 3. Camera offset = `trauma²` × [`DEFAULT_SHAKE_MAX_OFFSET`] × random direction
//!
//! Squaring trauma makes mild shakes very subtle while large events feel
//! dramatic — the response curve is non-linear.

use bevy::prelude::*;
use rand::RngExt;

use crate::config::ShakeParams;
use crate::events::FruitMergeEvent;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Fallback: trauma lost per second when `shake.ron` is not yet loaded
pub const DEFAULT_SHAKE_DECAY: f32 = 3.5;

/// Fallback: maximum camera displacement in pixels when `shake.ron` is not yet loaded
pub const DEFAULT_SHAKE_MAX_OFFSET: f32 = 12.0;

/// Fallback: minimum fruit index that triggers a shake when `shake.ron` is not yet loaded.
/// Index 4 = Persimmon (the 5th fruit). Smaller fruits cause no shake.
pub const DEFAULT_SHAKE_MIN_INDEX: usize = 4;

/// Fallback: trauma added per fruit-index step above the minimum when `shake.ron` is not yet loaded
pub const DEFAULT_SHAKE_INTENSITY_STEP: f32 = 0.2;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Camera shake state component
///
/// Attach this to the camera entity. `trauma` accumulates on merge events and
/// decays each frame. The camera's `Transform` is offset in proportion to
/// `trauma²`.
///
/// When `trauma` reaches zero the camera snaps back to its origin (0, 0).
#[derive(Component, Debug, Default)]
pub struct CameraShake {
    /// Current trauma level in `[0.0, 1.0]`
    pub trauma: f32,
}

impl CameraShake {
    /// Adds `amount` to trauma, clamping the result to `[0.0, 1.0]`
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Adds camera trauma on fruit merge events
///
/// For each `FruitMergeEvent` involving a fruit at or above the configured
/// `min_fruit_index` (default [`DEFAULT_SHAKE_MIN_INDEX`]), increments the
/// `CameraShake.trauma` on the camera entity by an amount proportional to
/// the fruit's index in the evolution chain.
///
/// Values are read from `assets/config/effects/shake.ron` when loaded,
/// falling back to the module constants otherwise.
pub fn add_camera_shake(
    mut merge_events: MessageReader<FruitMergeEvent>,
    mut shake_query: Query<&mut CameraShake>,
    shake: ShakeParams<'_>,
) {
    let cfg = shake.get();

    let min_index = cfg
        .map(|c| c.min_fruit_index)
        .unwrap_or(DEFAULT_SHAKE_MIN_INDEX);
    let intensity_step = cfg
        .map(|c| c.intensity_step)
        .unwrap_or(DEFAULT_SHAKE_INTENSITY_STEP);

    for event in merge_events.read() {
        let fruit_index = event.fruit_type as usize;
        if fruit_index < min_index {
            continue;
        }

        let steps_above_min = (fruit_index - min_index + 1) as f32;
        let intensity = (steps_above_min * intensity_step).clamp(0.0, 1.0);

        if let Ok(mut shake) = shake_query.single_mut() {
            shake.add_trauma(intensity);
        }
    }
}

/// Applies camera shake each frame and decays trauma
///
/// Each frame this system:
/// 1. Decays `trauma` by `decay` × `delta_secs` (from config or [`DEFAULT_SHAKE_DECAY`])
/// 2. Computes `shake_amount = trauma²`
/// 3. Applies a random X/Y offset to the camera `Transform`, scaled by
///    `shake_amount × max_offset` (from config or [`DEFAULT_SHAKE_MAX_OFFSET`])
/// 4. Snaps the camera back to the origin `(0, 0)` once trauma is negligible
///
/// The Z coordinate of the camera is never modified.
/// Runs every frame regardless of game state so that trauma always decays
/// and the camera returns to center even during Paused and GameOver.
pub fn apply_camera_shake(
    mut query: Query<(&mut Transform, &mut CameraShake), With<Camera2d>>,
    time: Res<Time>,
    shake: ShakeParams<'_>,
) {
    let Ok((mut transform, mut shake_state)) = query.single_mut() else {
        return;
    };

    let cfg = shake.get();

    let decay = cfg.map(|c| c.decay).unwrap_or(DEFAULT_SHAKE_DECAY);
    let max_offset = cfg
        .map(|c| c.max_offset)
        .unwrap_or(DEFAULT_SHAKE_MAX_OFFSET);

    // Decay trauma each frame regardless of state
    if shake_state.trauma > 0.0 {
        shake_state.trauma = (shake_state.trauma - decay * time.delta_secs()).max(0.0);
    }

    let shake_amount = shake_state.trauma * shake_state.trauma;

    if shake_amount < 0.001 {
        // Snap camera back to world origin
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
        return;
    }

    // Random X/Y offset in [-max_offset, +max_offset], scaled by shake_amount
    let mut rng = rand::rng();
    let offset_x = shake_amount * max_offset * rng.random_range(-1.0_f32..1.0);
    let offset_y = shake_amount * max_offset * rng.random_range(-1.0_f32..1.0);

    transform.translation.x = offset_x;
    transform.translation.y = offset_y;
    // Z is intentionally left unchanged (camera depth must stay at 999.9)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_trauma_clamps_to_one() {
        let mut shake = CameraShake::default();
        shake.add_trauma(0.8);
        shake.add_trauma(0.8); // Would be 1.6 without clamp
        assert!(
            shake.trauma <= 1.0,
            "Trauma must never exceed 1.0, got {}",
            shake.trauma
        );
    }

    #[test]
    fn test_add_trauma_accumulates() {
        let mut shake = CameraShake::default();
        shake.add_trauma(0.3);
        shake.add_trauma(0.2);
        assert!(
            (shake.trauma - 0.5).abs() < f32::EPSILON,
            "Trauma should accumulate: expected 0.5, got {}",
            shake.trauma
        );
    }

    #[test]
    fn test_small_fruits_do_not_trigger_shake() {
        // Drive the actual system: a sub-threshold merge event must leave trauma at 0.
        // Cherry (index 0) is well below DEFAULT_SHAKE_MIN_INDEX.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<FruitMergeEvent>();
        app.add_systems(Update, add_camera_shake);

        let entity = app.world_mut().spawn(CameraShake::default()).id();

        app.world_mut().write_message(FruitMergeEvent {
            entity1: Entity::PLACEHOLDER,
            entity2: Entity::PLACEHOLDER,
            fruit_type: crate::fruit::FruitType::Cherry,
            position: Vec2::ZERO,
        });

        app.update();

        let shake = app.world().get::<CameraShake>(entity).unwrap();
        assert_eq!(
            shake.trauma, 0.0,
            "Cherry (index 0) is below the shake threshold and must not add trauma"
        );
    }

    #[test]
    fn test_large_fruit_intensity_scales_up() {
        // Watermelon is index 10, well above DEFAULT_SHAKE_MIN_INDEX (4)
        let watermelon_index = 10_usize;
        let steps = (watermelon_index - DEFAULT_SHAKE_MIN_INDEX + 1) as f32;
        let intensity = (steps * DEFAULT_SHAKE_INTENSITY_STEP).clamp(0.0, 1.0);
        assert!(
            intensity > 0.5,
            "Watermelon should produce high intensity (>0.5), got {intensity}"
        );
    }

    #[test]
    fn test_shake_amount_is_trauma_squared() {
        // The non-linear response: at trauma=0.5, shake_amount = 0.25
        let trauma: f32 = 0.5;
        let shake_amount = trauma * trauma;
        assert!(
            (shake_amount - 0.25).abs() < f32::EPSILON,
            "shake_amount should equal trauma², got {shake_amount}"
        );
    }

    #[test]
    fn test_apply_camera_shake_decays_trauma() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, apply_camera_shake);

        let entity = app
            .world_mut()
            .spawn((
                Camera2d,
                Transform::from_xyz(0.0, 0.0, 999.9),
                CameraShake { trauma: 1.0 },
            ))
            .id();

        app.update();
        app.update(); // second frame has non-zero delta

        let shake = app.world().get::<CameraShake>(entity).unwrap();
        assert!(
            shake.trauma < 1.0,
            "Trauma should decay after a frame, got {}",
            shake.trauma
        );
    }

    #[test]
    fn test_apply_camera_shake_snaps_back_when_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, apply_camera_shake);

        // Spawn camera with zero trauma
        app.world_mut().spawn((
            Camera2d,
            Transform::from_xyz(5.0, -3.0, 999.9), // offset to confirm snap-back
            CameraShake { trauma: 0.0 },
        ));

        app.update();

        let mut q = app.world_mut().query::<(&Transform, &CameraShake)>();
        let Ok((transform, _)) = q.single(app.world()) else {
            panic!("Camera not found");
        };
        assert!(
            transform.translation.x.abs() < f32::EPSILON,
            "Camera x should snap to 0 when trauma=0, got {}",
            transform.translation.x
        );
        assert!(
            transform.translation.y.abs() < f32::EPSILON,
            "Camera y should snap to 0 when trauma=0, got {}",
            transform.translation.y
        );
    }
}
