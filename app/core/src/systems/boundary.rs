//! Boundary overflow detection and game-over warning effects
//!
//! This module monitors whether any fruit has risen above the boundary line.
//! When fruits stay above the line long enough, the game transitions to
//! `AppState::GameOver`.  While in warning state the boundary line sprite
//! blinks red to alert the player.

use bevy::prelude::*;

use crate::components::{BoundaryLine, Fruit};
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

/// Checks whether any fruit is above the boundary line and updates the timer.
///
/// - If at least one fruit is above the line, the `GameOverTimer` advances and
///   the warning flag is set.
/// - If no fruits are above the line, the timer and warning flag are reset.
///
/// The boundary Y position is read from `PhysicsConfig`; the default 300.0 is
/// used while the asset loads.
pub fn check_boundary_overflow(
    fruit_query: Query<&Transform, With<Fruit>>,
    mut game_over_timer: ResMut<GameOverTimer>,
    time: Res<Time>,
    physics_handle: Option<Res<PhysicsConfigHandle>>,
    physics_assets: Option<Res<Assets<PhysicsConfig>>>,
) {
    let threshold = boundary_y(physics_handle.as_ref(), physics_assets.as_ref());

    let any_overflow = fruit_query.iter().any(|t| t.translation.y > threshold);

    if any_overflow {
        game_over_timer.tick_warning(time.delta_secs());
    } else {
        game_over_timer.reset();
    }
}

/// Transitions to `AppState::GameOver` when the timer exceeds its threshold.
///
/// Only fires from `AppState::Playing` and guards against double-triggering.
pub fn trigger_game_over(
    game_over_timer: Res<GameOverTimer>,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if *current_state.get() != AppState::Playing {
        return;
    }

    if game_over_timer.is_game_over() {
        info!(
            "Game Over! Fruit exceeded boundary for {:.2}s",
            game_over_timer.time_over_boundary
        );
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
            // Smooth sine-based blink at ~2 Hz (visible even at low frame rates)
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

        timer.tick_warning(3.0);
        assert!(timer.is_game_over());
    }

    #[test]
    fn test_game_over_timer_resets() {
        let mut timer = GameOverTimer::default();
        timer.tick_warning(2.0);
        assert!(timer.is_warning);

        timer.reset();
        assert!(!timer.is_warning);
        assert_eq!(timer.time_over_boundary, 0.0);
        assert!(!timer.is_game_over());
    }
}
