//! Game constants and configuration values
//!
//! This module contains constant values that are not part of the hot-reloadable
//! configuration system. Most game parameters are now loaded from RON files:
//! - Physics: `assets/config/physics.ron` (PhysicsConfig)
//! - Game rules: `assets/config/game_rules.ron` (GameRulesConfig)
//! - Fruit parameters: `assets/config/fruits.ron` (FruitsConfig)

/// Persistence and storage constants
pub mod storage {
    /// Directory where save files are stored
    ///
    /// This directory will be created if it doesn't exist when
    /// saving game data (e.g., highscore).
    pub const SAVE_DIR: &str = "save";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_constants() {
        // Verify storage directory path
        assert_eq!(storage::SAVE_DIR, "save");
    }
}
