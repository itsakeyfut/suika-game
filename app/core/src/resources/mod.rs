//! Game state resources
//!
//! This module defines Bevy resources for managing game state,
//! including score tracking, combo system, game over detection,
//! and next fruit preview.

pub mod combo;
pub mod game;
pub mod game_over;
pub mod spawn;

pub use combo::ComboTimer;
pub use game::GameState;
pub use game_over::GameOverTimer;
pub use spawn::NextFruitType;

#[cfg(test)]
mod tests {
    use super::combo::{DEFAULT_COMBO_MAX, DEFAULT_COMBO_WINDOW};
    use super::game_over::DEFAULT_WARNING_THRESHOLD;
    use super::*;
    use crate::fruit::FruitType;

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
        assert_eq!(timer.combo_window, DEFAULT_COMBO_WINDOW);
        assert_eq!(timer.combo_max, DEFAULT_COMBO_MAX);
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
        timer.time_since_last_merge = DEFAULT_COMBO_WINDOW + 1.0;
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

        timer.time_since_last_merge = DEFAULT_COMBO_WINDOW + 1.0;
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
        assert_eq!(timer.warning_threshold, DEFAULT_WARNING_THRESHOLD);
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
        next.randomize(5);

        // Check that it's a spawnable fruit
        let spawnable = FruitType::spawnable_fruits();
        assert!(spawnable.contains(&next.get()));
    }

    #[test]
    fn test_next_fruit_type_randomize_count_limits_range() {
        // With count=1, only Cherry should ever be returned
        let mut next = NextFruitType::default();
        for _ in 0..20 {
            next.randomize(1);
            assert_eq!(
                next.get(),
                FruitType::Cherry,
                "With spawnable_count=1 only Cherry should be returned"
            );
        }
    }

    #[test]
    fn test_next_fruit_type_randomize_clamps_oversized_count() {
        // count > 5 should clamp to 5 without panicking
        let mut next = NextFruitType::default();
        let spawnable = FruitType::spawnable_fruits();
        for _ in 0..20 {
            next.randomize(999);
            assert!(spawnable.contains(&next.get()));
        }
    }
}
