//! Application state management
//!
//! This module defines the application states for the game lifecycle.
//! States control which systems run and manage transitions between
//! different screens and gameplay modes.

use bevy::prelude::*;

/// Application state enum
///
/// Represents the high-level state of the application, controlling
/// which systems are active and what screen is displayed.
///
/// # State Transitions
///
/// - `Title` → `Playing`: Player starts a new game
/// - `Playing` → `Paused`: Player pauses the game
/// - `Paused` → `Playing`: Player resumes the game
/// - `Playing` → `GameOver`: Game over condition is met
/// - `GameOver` → `Title`: Player returns to title screen
/// - `GameOver` → `Playing`: Player starts a new game
///
/// # Usage
///
/// ```no_run
/// use bevy::prelude::*;
/// use suika_game_core::states::AppState;
///
/// fn setup_title_screen(mut commands: Commands) {
///     // Setup title screen UI
/// }
///
/// fn main() {
///     App::new()
///         .init_state::<AppState>()
///         .add_systems(OnEnter(AppState::Title), setup_title_screen)
///         .run();
/// }
/// ```
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// Title screen state
    ///
    /// Displays the game title, menu options, and high score.
    /// This is the initial state when the application starts.
    #[default]
    Title,

    /// Active gameplay state
    ///
    /// The main game loop is running. Player can drop fruits
    /// and interact with the game.
    Playing,

    /// Paused state
    ///
    /// Game is paused. Physics simulation and gameplay systems
    /// are suspended, but the game state is preserved.
    Paused,

    /// Game over state
    ///
    /// Displays final score, high score update, and options
    /// to retry or return to title.
    GameOver,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert_eq!(state, AppState::Title);
    }

    #[test]
    fn test_app_state_equality() {
        assert_eq!(AppState::Title, AppState::Title);
        assert_eq!(AppState::Playing, AppState::Playing);
        assert_eq!(AppState::Paused, AppState::Paused);
        assert_eq!(AppState::GameOver, AppState::GameOver);

        assert_ne!(AppState::Title, AppState::Playing);
        assert_ne!(AppState::Playing, AppState::Paused);
        assert_ne!(AppState::Paused, AppState::GameOver);
    }

    #[test]
    fn test_app_state_clone() {
        let state = AppState::Playing;
        let cloned = state;
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_app_state_debug() {
        // Verify that Debug is implemented
        let state = AppState::Title;
        let debug_str = format!("{:?}", state);
        assert_eq!(debug_str, "Title");

        let state = AppState::Playing;
        let debug_str = format!("{:?}", state);
        assert_eq!(debug_str, "Playing");
    }

    #[test]
    fn test_app_state_all_variants() {
        // Ensure all variants are covered
        let states = vec![
            AppState::Title,
            AppState::Playing,
            AppState::Paused,
            AppState::GameOver,
        ];

        // All states should be distinct
        for (i, state1) in states.iter().enumerate() {
            for (j, state2) in states.iter().enumerate() {
                if i == j {
                    assert_eq!(state1, state2);
                } else {
                    assert_ne!(state1, state2);
                }
            }
        }
    }
}
