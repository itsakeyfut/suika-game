//! Game-over timer resource

use bevy::prelude::*;

/// Default game-over warning threshold in seconds — mirrors `game_rules.ron` `game_over_timer`.
pub(crate) const DEFAULT_WARNING_THRESHOLD: f32 = 0.5;

/// Game over timer resource
///
/// Tracks how long fruits have been above the boundary line.
/// Game over occurs when fruits stay above the line for the full duration
/// of the warning threshold (default 3.0 seconds).
#[derive(Resource, Debug, Clone)]
pub struct GameOverTimer {
    /// Time in seconds that fruits have been above the boundary
    pub time_over_boundary: f32,
    /// Duration in seconds before game over triggers
    pub warning_threshold: f32,
    /// Whether currently in warning state (fruit above boundary)
    pub is_warning: bool,
}

impl Default for GameOverTimer {
    fn default() -> Self {
        Self {
            time_over_boundary: 0.0,
            // Short threshold: long enough to ignore newly dropped fruits
            // passing through the boundary area, fast enough to feel immediate.
            // Can be overridden from game_rules.ron at runtime.
            warning_threshold: DEFAULT_WARNING_THRESHOLD,
            is_warning: false,
        }
    }
}

impl GameOverTimer {
    /// Updates the timer when fruits are above the boundary
    pub fn tick_warning(&mut self, delta: f32) {
        self.time_over_boundary += delta;
        self.is_warning = true;
    }

    /// Resets the timer when all fruits are below the boundary
    pub fn reset(&mut self) {
        self.time_over_boundary = 0.0;
        self.is_warning = false;
    }

    /// Resets session state while preserving config values.
    ///
    /// Clears `time_over_boundary` and `is_warning`, but keeps
    /// `warning_threshold` as loaded from the RON config.
    pub fn reset_session(&mut self) {
        self.time_over_boundary = 0.0;
        self.is_warning = false;
    }

    /// Returns true if the game over condition has been met
    pub fn is_game_over(&self) -> bool {
        self.time_over_boundary >= self.warning_threshold
    }

    /// Returns the progress toward game over (0.0 to 1.0)
    pub fn warning_progress(&self) -> f32 {
        (self.time_over_boundary / self.warning_threshold).min(1.0)
    }
}
