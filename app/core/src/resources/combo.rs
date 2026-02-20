//! Combo timer resource

use bevy::prelude::*;

/// Default combo window in seconds — mirrors `game_rules.ron` `combo_window`.
pub(crate) const DEFAULT_COMBO_WINDOW: f32 = 5.0;
/// Default maximum combo count — mirrors `game_rules.ron` `combo_max`.
pub(crate) const DEFAULT_COMBO_MAX: u32 = 10;

/// Combo timer resource
///
/// Manages the combo system by tracking time since the last merge
/// and maintaining the current combo count.
///
/// A combo occurs when fruits merge within the combo window (default 5.0 seconds).
/// The combo counter increases with each merge in the window and resets
/// when the window expires.
#[derive(Resource, Debug, Clone)]
pub struct ComboTimer {
    /// Time in seconds since the last merge occurred
    pub time_since_last_merge: f32,
    /// Duration of the combo window in seconds (loaded from game_rules.ron)
    pub combo_window: f32,
    /// Maximum combo count (loaded from game_rules.ron)
    pub combo_max: u32,
    /// Current combo count (starts at 1, increases with consecutive merges)
    pub current_combo: u32,
}

impl Default for ComboTimer {
    fn default() -> Self {
        Self {
            // Start with max value so first merge doesn't count as combo
            time_since_last_merge: f32::MAX,
            // Default values (updated from game_rules.ron at runtime)
            combo_window: DEFAULT_COMBO_WINDOW,
            combo_max: DEFAULT_COMBO_MAX,
            current_combo: 1,
        }
    }
}

impl ComboTimer {
    /// Updates the timer with delta time
    ///
    /// Should be called every frame to track time progression.
    pub fn tick(&mut self, delta: f32) {
        self.time_since_last_merge += delta;
    }

    /// Registers a merge event
    ///
    /// If within the combo window, increments the combo counter.
    /// Otherwise, resets to combo of 1.
    pub fn register_merge(&mut self) {
        if self.time_since_last_merge <= self.combo_window {
            self.current_combo = (self.current_combo + 1).min(self.combo_max);
        } else {
            self.current_combo = 1;
        }
        self.time_since_last_merge = 0.0;
    }

    /// Checks if the combo window has expired and resets if needed
    pub fn check_and_reset(&mut self) {
        if self.time_since_last_merge > self.combo_window && self.current_combo > 1 {
            self.current_combo = 1;
        }
    }

    /// Returns true if currently in a combo (2+ merges)
    pub fn is_combo(&self) -> bool {
        self.current_combo >= 2
    }

    /// Resets session state while preserving config values.
    ///
    /// Clears `time_since_last_merge` and `current_combo` back to their
    /// initial values, but keeps `combo_window` and `combo_max` as loaded
    /// from the RON config.  Use this instead of `*self = ComboTimer::default()`
    /// when resetting between games.
    pub fn reset_session(&mut self) {
        self.time_since_last_merge = f32::MAX;
        self.current_combo = 1;
    }
}
