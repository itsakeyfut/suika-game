//! Game state resources
//!
//! This module defines Bevy resources for managing game state,
//! including score tracking, combo system, game over detection,
//! and next fruit preview.

use bevy::prelude::*;

use crate::fruit::FruitType;

/// Main game state resource
///
/// Tracks the player's current score, all-time high score,
/// and elapsed time in the current game session.
#[derive(Resource, Debug, Clone)]
pub struct GameState {
    /// Current score in this game session
    pub score: u32,
    /// All-time high score (persisted across sessions)
    pub highscore: u32,
    /// Elapsed time in seconds since game started
    pub elapsed_time: f32,
    /// Set to `true` on `OnEnter(GameOver)` when the current score beats the
    /// previous highscore.  Consumed by the game-over screen to show the
    /// "NEW RECORD!" banner.  Cleared on every game reset.
    pub is_new_record: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            score: 0,
            highscore: 0,
            elapsed_time: 0.0,
            is_new_record: false,
        }
    }
}

/// Combo timer resource
///
/// Manages the combo system by tracking time since the last merge
/// and maintaining the current combo count.
///
/// A combo occurs when fruits merge within the combo window (default 2.0 seconds).
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
            combo_window: 2.0,
            combo_max: 10,
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
}

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
            warning_threshold: 0.5,
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

    /// Returns true if the game over condition has been met
    pub fn is_game_over(&self) -> bool {
        self.time_over_boundary >= self.warning_threshold
    }

    /// Returns the progress toward game over (0.0 to 1.0)
    pub fn warning_progress(&self) -> f32 {
        (self.time_over_boundary / self.warning_threshold).min(1.0)
    }
}

/// Next fruit type resource
///
/// Stores the type of fruit that will be spawned next.
/// This allows the UI to display a preview of the upcoming fruit.
#[derive(Resource, Debug, Clone)]
pub struct NextFruitType(pub FruitType);

impl Default for NextFruitType {
    fn default() -> Self {
        Self(FruitType::Cherry)
    }
}

impl NextFruitType {
    /// Gets the current next fruit type
    pub fn get(&self) -> FruitType {
        self.0
    }

    /// Sets a new next fruit type
    pub fn set(&mut self, fruit_type: FruitType) {
        self.0 = fruit_type;
    }

    /// Generates a random spawnable fruit type
    ///
    /// Returns one of the 5 spawnable fruit types (Cherry through Persimmon)
    /// with equal probability.
    pub fn random() -> FruitType {
        use rand::RngExt;
        let spawnable = FruitType::spawnable_fruits();
        let index = rand::rng().random_range(0..spawnable.len());
        spawnable[index]
    }

    /// Updates to a new random fruit type
    pub fn randomize(&mut self) {
        self.0 = Self::random();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_default() {
        let state = GameState::default();
        assert_eq!(state.score, 0);
        assert_eq!(state.highscore, 0);
        assert_eq!(state.elapsed_time, 0.0);
    }

    #[test]
    fn test_combo_timer_default() {
        let timer = ComboTimer::default();
        assert_eq!(timer.time_since_last_merge, f32::MAX);
        assert_eq!(timer.combo_window, 2.0); // Default value
        assert_eq!(timer.combo_max, 10); // Default value
        assert_eq!(timer.current_combo, 1);
        assert!(!timer.is_combo());
    }

    #[test]
    fn test_combo_timer_register_merge() {
        let mut timer = ComboTimer::default();

        // First merge - starts combo system (time_since_last_merge is f32::MAX)
        timer.register_merge();
        assert_eq!(timer.current_combo, 1);
        assert_eq!(timer.time_since_last_merge, 0.0);
        assert!(!timer.is_combo());

        // Second merge within window - combo!
        timer.time_since_last_merge = 1.0;
        timer.register_merge();
        assert_eq!(timer.current_combo, 2);
        assert!(timer.is_combo());

        // Third merge within window
        timer.time_since_last_merge = 0.5;
        timer.register_merge();
        assert_eq!(timer.current_combo, 3);

        // Merge after window expires - reset to 1
        timer.time_since_last_merge = 3.0;
        timer.register_merge();
        assert_eq!(timer.current_combo, 1);
        assert!(!timer.is_combo());
    }

    #[test]
    fn test_combo_timer_check_and_reset() {
        let mut timer = ComboTimer::default();

        timer.current_combo = 5;
        timer.time_since_last_merge = 1.0;
        timer.check_and_reset();
        assert_eq!(timer.current_combo, 5); // Still in window

        timer.time_since_last_merge = 3.0;
        timer.check_and_reset();
        assert_eq!(timer.current_combo, 1); // Window expired, reset
    }

    #[test]
    fn test_combo_timer_max() {
        let mut timer = ComboTimer::default();

        // Simulate many merges to hit max combo
        for _ in 0..20 {
            timer.time_since_last_merge = 0.5;
            timer.register_merge();
        }

        assert_eq!(timer.current_combo, 10); // Default max combo
    }

    #[test]
    fn test_game_over_timer_default() {
        let timer = GameOverTimer::default();
        assert_eq!(timer.time_over_boundary, 0.0);
        assert_eq!(timer.warning_threshold, 0.5); // Short threshold for near-immediate game-over
        assert!(!timer.is_warning);
        assert!(!timer.is_game_over());
    }

    #[test]
    fn test_game_over_timer_progression() {
        let mut timer = GameOverTimer::default();

        // Start warning
        timer.tick_warning(0.2);
        assert!(timer.is_warning);
        assert!(!timer.is_game_over());
        assert_eq!(timer.warning_progress(), 0.2 / 0.5);

        // Continue warning — still below threshold
        timer.tick_warning(0.2);
        assert_eq!(timer.time_over_boundary, 0.4);
        assert!(!timer.is_game_over());

        // Reach game over
        timer.tick_warning(0.2);
        assert!(timer.is_game_over());
        assert_eq!(timer.warning_progress(), 1.0);

        // Reset
        timer.reset();
        assert_eq!(timer.time_over_boundary, 0.0);
        assert!(!timer.is_warning);
        assert!(!timer.is_game_over());
    }

    #[test]
    fn test_next_fruit_type_default() {
        let next = NextFruitType::default();
        assert_eq!(next.get(), FruitType::Cherry);
    }

    #[test]
    fn test_next_fruit_type_set_get() {
        let mut next = NextFruitType::default();

        next.set(FruitType::Strawberry);
        assert_eq!(next.get(), FruitType::Strawberry);

        next.set(FruitType::Grape);
        assert_eq!(next.get(), FruitType::Grape);
    }

    #[test]
    fn test_next_fruit_type_random() {
        // Test that random returns only spawnable fruits
        for _ in 0..20 {
            let fruit = NextFruitType::random();
            let spawnable = FruitType::spawnable_fruits();
            assert!(
                spawnable.contains(&fruit),
                "Random fruit should be spawnable"
            );
        }
    }

    #[test]
    fn test_next_fruit_type_randomize() {
        let mut next = NextFruitType::default();
        next.randomize();

        // Check that it's a spawnable fruit
        let spawnable = FruitType::spawnable_fruits();
        assert!(spawnable.contains(&next.get()));
    }
}
