//! Main game-state resource

use bevy::prelude::*;

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
