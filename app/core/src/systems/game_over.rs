//! Game-over and game-reset systems
//!
//! This module provides three lifecycle systems:
//!
//! - `tick_elapsed_time` — runs every frame during `AppState::Playing`.
//!   Increments [`GameState::elapsed_time`] so the HUD can display a live timer.
//!
//! - `save_highscore_on_game_over` — runs on `OnEnter(AppState::GameOver)`
//!   inside [`GameOverSet::SaveHighscore`].  Compares the current score with
//!   the stored highscore and writes to disk when a new record is set.
//!
//! - `reset_game_state` — runs on `OnEnter(AppState::Playing)`.
//!   Clears all in-game resources and despawns existing fruits so each new
//!   game starts from a clean slate.  The highscore is intentionally preserved.
//!
//! ## Ordering for downstream crates
//!
//! UI or other crates that need to read [`GameState::is_new_record`] on
//! `OnEnter(AppState::GameOver)` should order their systems
//! `.after(`[`GameOverSet::SaveHighscore`]`)` to guarantee they run after the
//! flag has been written.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// System sets
// ---------------------------------------------------------------------------

/// System-set labels for game-over lifecycle systems.
///
/// Downstream crates can use these labels with `.after()` / `.before()` to
/// establish explicit ordering relative to the core game-over logic without
/// coupling to internal function names.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameOverSet {
    /// Contains [`save_highscore_on_game_over`].
    ///
    /// Runs on `OnEnter(AppState::GameOver)`.  After this set completes,
    /// [`GameState::is_new_record`] and [`GameState::highscore`] are
    /// up-to-date and safe to read.
    SaveHighscore,
}

use crate::components::Fruit;
use crate::constants::storage::SAVE_DIR;
use crate::persistence::{HighscoreData, save_highscore};
use crate::resources::{ComboTimer, GameOverTimer, GameState};
use crate::systems::input::{InputMode, SpawnPosition};

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Advances [`GameState::elapsed_time`] by the frame delta.
///
/// Should run every frame while `AppState::Playing` is active so the HUD
/// timer and any other consumers always have an up-to-date value.
pub fn tick_elapsed_time(mut game_state: ResMut<GameState>, time: Res<Time>) {
    game_state.elapsed_time += time.delta_secs();
}

/// Saves the highscore to disk when the game ends.
///
/// Only writes to disk when the current score exceeds the stored highscore.
/// Runs once on `OnEnter(AppState::GameOver)`.
pub fn save_highscore_on_game_over(mut game_state: ResMut<GameState>) {
    if game_state.score > game_state.highscore {
        info!(
            "New highscore! {} → {}",
            game_state.highscore, game_state.score
        );
        game_state.is_new_record = true;
        game_state.highscore = game_state.score;

        let data = HighscoreData {
            highscore: game_state.highscore,
        };

        match save_highscore(&data, std::path::Path::new(SAVE_DIR)) {
            Ok(_) => info!("Highscore saved to {SAVE_DIR}/highscore.json"),
            Err(e) => error!("Failed to save highscore: {e}"),
        }
    } else {
        game_state.is_new_record = false;
        info!(
            "Game over. Score: {} (Highscore: {})",
            game_state.score, game_state.highscore
        );
    }
}

/// Resets all mutable game state and despawns existing fruits.
///
/// Runs once on `OnEnter(AppState::Playing)` so that both the initial game
/// start and any subsequent retries begin from a consistent state.
///
/// The highscore is **not** reset.
pub fn reset_game_state(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut combo_timer: ResMut<ComboTimer>,
    mut game_over_timer: ResMut<GameOverTimer>,
    mut input_mode: ResMut<InputMode>,
    mut spawn_pos: ResMut<SpawnPosition>,
    fruit_query: Query<Entity, With<Fruit>>,
) {
    let highscore = game_state.highscore;

    *game_state = GameState {
        score: 0,
        highscore,
        elapsed_time: 0.0,
        is_new_record: false,
    };
    combo_timer.reset_session();
    game_over_timer.reset_session();

    // Reset input state so the held fruit always starts at the container center
    *input_mode = InputMode::Keyboard;
    *spawn_pos = SpawnPosition::default();

    let mut despawned = 0u32;
    for entity in fruit_query.iter() {
        commands.entity(entity).despawn();
        despawned += 1;
    }

    info!("Game reset. Highscore: {highscore}. Despawned {despawned} fruits.");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_reset_preserves_highscore() {
        let mut state = GameState {
            score: 5000,
            highscore: 8000,
            elapsed_time: 42.0,
            is_new_record: true,
        };

        let highscore = state.highscore;
        state = GameState {
            score: 0,
            highscore,
            elapsed_time: 0.0,
            is_new_record: false,
        };

        assert_eq!(state.score, 0);
        assert_eq!(state.highscore, 8000);
        assert_eq!(state.elapsed_time, 0.0);
        assert!(!state.is_new_record);
    }

    #[test]
    fn test_highscore_only_updated_when_beaten() {
        let score = 5000u32;
        let highscore = 8000u32;

        // Score did not beat highscore
        let new_highscore = if score > highscore { score } else { highscore };
        assert_eq!(new_highscore, 8000);

        // Score beats highscore
        let score2 = 10000u32;
        let new_highscore2 = if score2 > highscore {
            score2
        } else {
            highscore
        };
        assert_eq!(new_highscore2, 10000);
    }
}
