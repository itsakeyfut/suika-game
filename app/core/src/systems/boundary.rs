//! Boundary overflow detection and game-over effects
//!
//! This module monitors whether any in-play fruit has risen above the
//! boundary line.  When the overflow condition persists for the warning
//! threshold (default 0.5 s) the game transitions to `AppState::GameOver`.
//! While in warning state the boundary line sprite blinks red.

use bevy::prelude::*;

use crate::components::{BoundaryLine, Fruit, FruitSpawnState};
use crate::config::{PhysicsConfig, PhysicsConfigHandle};
use crate::resources::GameOverTimer;
use crate::states::AppState;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Returns the boundary-line Y threshold from config, falling back to the
/// physics.ron default when the asset is not yet loaded.
fn boundary_y(
    physics_handle: Option<&Res<PhysicsConfigHandle>>,
    physics_assets: Option<&Res<Assets<PhysicsConfig>>>,
) -> f32 {
    physics_handle
        .and_then(|h| physics_assets.and_then(|a| a.get(&h.0)))
        .map(|c| c.boundary_line_y)
        .unwrap_or(300.0)
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Checks whether any in-play fruit is above the boundary line.
///
/// `Held` fruits are excluded because they sit above the container top by
/// design and must not trigger the warning.  Both `Falling` and `Landed`
/// fruits are included so that fruits pushed upward by physics (which may
/// never transition back to `Landed`) are still detected.
///
/// When overflow is detected the `GameOverTimer` advances.  The short
/// threshold (0.5 s default) filters out the brief window when a newly
/// dropped fruit passes through the boundary area before settling.
/// When no overflow is detected the timer resets.
pub fn check_boundary_overflow(
    fruit_query: Query<(&Transform, &FruitSpawnState), With<Fruit>>,
    mut game_over_timer: ResMut<GameOverTimer>,
    time: Res<Time>,
    physics_handle: Option<Res<PhysicsConfigHandle>>,
    physics_assets: Option<Res<Assets<PhysicsConfig>>>,
) {
    let threshold = boundary_y(physics_handle.as_ref(), physics_assets.as_ref());

    // Held fruits sit above the drop zone by design — exclude them only.
    let any_overflow = fruit_query
        .iter()
        .filter(|(_, state)| **state != FruitSpawnState::Held)
        .any(|(t, _)| t.translation.y > threshold);

    if any_overflow {
        game_over_timer.tick_warning(time.delta_secs());
    } else {
        game_over_timer.reset();
    }
}

/// Transitions to `AppState::GameOver` when the timer exceeds its threshold.
///
/// Only fires from `AppState::Playing` to guard against double-triggering.
pub fn trigger_game_over(
    game_over_timer: Res<GameOverTimer>,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if *current_state.get() != AppState::Playing {
        return;
    }

    if game_over_timer.is_game_over() {
        info!("Game Over! Fruit exceeded boundary line.");
        next_state.set(AppState::GameOver);
    }
}

/// Animates the boundary line sprite to blink red while in warning state.
///
/// - Warning active: alternates between bright and dark red at ~2 Hz
/// - Warning inactive: restores the default semi-transparent red color
pub fn animate_boundary_warning(
    game_over_timer: Res<GameOverTimer>,
    mut boundary_query: Query<&mut Sprite, With<BoundaryLine>>,
    time: Res<Time>,
) {
    for mut sprite in boundary_query.iter_mut() {
        if game_over_timer.is_warning {
            // Smooth sine-based blink at ~2 Hz
            let blink = (time.elapsed_secs() * 4.0).sin();
            let alpha = 0.4 + 0.4 * blink; // range [0.0, 0.8]
            sprite.color = Color::srgba(1.0, 0.0, 0.0, alpha);
        } else {
            // Default: semi-transparent red
            sprite.color = Color::srgba(1.0, 0.0, 0.0, 0.5);
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
    fn test_game_over_timer_triggers_at_threshold() {
        let mut timer = GameOverTimer::default();
        assert!(!timer.is_game_over());

        timer.tick_warning(0.5);
        assert!(timer.is_game_over());
    }

    #[test]
    fn test_game_over_timer_resets() {
        let mut timer = GameOverTimer::default();
        timer.tick_warning(0.3);
        assert!(timer.is_warning);

        timer.reset();
        assert!(!timer.is_warning);
        assert_eq!(timer.time_over_boundary, 0.0);
        assert!(!timer.is_game_over());
    }
}
