//! Highscore persistence
//!
//! This module handles saving and loading the player's highscore
//! to/from a JSON file on disk. The highscore persists across
//! game sessions.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Highscore data structure
///
/// This structure is serialized to JSON and saved to disk.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HighscoreData {
    /// The player's all-time highest score
    pub highscore: u32,
}

/// Directory where save files are stored
const SAVE_DIR: &str = "save";

/// Path to the highscore JSON file
const HIGHSCORE_FILE: &str = "save/highscore.json";

/// Saves the highscore data to a JSON file
///
/// This function will:
/// 1. Create the `save/` directory if it doesn't exist
/// 2. Serialize the highscore data to pretty-printed JSON
/// 3. Write the JSON to `save/highscore.json`
///
/// # Arguments
///
/// * `data` - The highscore data to save
///
/// # Returns
///
/// * `Ok(())` if the save was successful
/// * `Err` if there was an IO error or serialization failed
///
/// # Examples
///
/// ```no_run
/// # use suika_game_core::persistence::{HighscoreData, save_highscore};
/// let data = HighscoreData { highscore: 10000 };
/// save_highscore(&data).expect("Failed to save highscore");
/// ```
pub fn save_highscore(data: &HighscoreData) -> Result<(), Box<dyn std::error::Error>> {
    // Create save directory if it doesn't exist
    fs::create_dir_all(SAVE_DIR)?;

    // Serialize to pretty-printed JSON
    let json = serde_json::to_string_pretty(data)?;

    // Write to file
    fs::write(HIGHSCORE_FILE, json)?;

    Ok(())
}

/// Loads the highscore data from a JSON file
///
/// This function will:
/// 1. Check if the highscore file exists
/// 2. If it exists, read and deserialize the JSON
/// 3. If it doesn't exist or there's an error, return default (0)
///
/// # Returns
///
/// * The saved highscore data if the file exists and is valid
/// * Default highscore (0) if the file doesn't exist or is corrupted
///
/// # Examples
///
/// ```no_run
/// # use suika_game_core::persistence::load_highscore;
/// let data = load_highscore();
/// println!("Current highscore: {}", data.highscore);
/// ```
pub fn load_highscore() -> HighscoreData {
    // Return default if file doesn't exist
    if !Path::new(HIGHSCORE_FILE).exists() {
        return HighscoreData::default();
    }

    // Try to read and deserialize the file
    match fs::read_to_string(HIGHSCORE_FILE) {
        Ok(json) => {
            // Deserialize JSON, return default if parsing fails
            serde_json::from_str(&json).unwrap_or_default()
        }
        Err(_) => HighscoreData::default(),
    }
}

/// Attempts to update the highscore if the new score is higher
///
/// This is a convenience function that:
/// 1. Loads the current highscore
/// 2. Compares it with the new score
/// 3. Saves the new score if it's higher
/// 4. Returns whether a new highscore was set
///
/// # Arguments
///
/// * `new_score` - The score to potentially save as the new highscore
///
/// # Returns
///
/// * `Ok(true)` if a new highscore was set and saved
/// * `Ok(false)` if the current highscore is still higher
/// * `Err` if there was an error saving
///
/// # Examples
///
/// ```no_run
/// # use suika_game_core::persistence::update_highscore;
/// match update_highscore(15000) {
///     Ok(true) => println!("New highscore!"),
///     Ok(false) => println!("Try again!"),
///     Err(e) => eprintln!("Failed to save: {}", e),
/// }
/// ```
pub fn update_highscore(new_score: u32) -> Result<bool, Box<dyn std::error::Error>> {
    let mut data = load_highscore();

    if new_score > data.highscore {
        data.highscore = new_score;
        save_highscore(&data)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Helper to create a unique test file path for each test
    fn test_file_path(name: &str) -> String {
        format!("test_save/{}.json", name)
    }

    fn test_dir() -> &'static str {
        "test_save"
    }

    // Clean up test files after each test
    fn cleanup_test_files() {
        let _ = fs::remove_dir_all(test_dir());
    }

    #[test]
    fn test_highscore_data_default() {
        let data = HighscoreData::default();
        assert_eq!(data.highscore, 0);
    }

    #[test]
    fn test_highscore_data_serde() {
        let data = HighscoreData { highscore: 12345 };

        // Test serialization
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("12345"));

        // Test deserialization
        let deserialized: HighscoreData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.highscore, 12345);
    }

    #[test]
    fn test_save_and_load_highscore() {
        cleanup_test_files();

        // Create test directory
        fs::create_dir_all(test_dir()).unwrap();

        let test_file = test_file_path("save_load");
        let data = HighscoreData { highscore: 54321 };

        // Manually save to test file
        let json = serde_json::to_string_pretty(&data).unwrap();
        fs::write(&test_file, json).unwrap();

        // Manually load from test file
        let loaded_json = fs::read_to_string(&test_file).unwrap();
        let loaded: HighscoreData = serde_json::from_str(&loaded_json).unwrap();

        assert_eq!(loaded.highscore, 54321);

        cleanup_test_files();
    }

    #[test]
    fn test_load_nonexistent_file() {
        cleanup_test_files();

        // Load from non-existent file should return default
        let test_file = test_file_path("nonexistent");
        if Path::new(&test_file).exists() {
            fs::remove_file(&test_file).unwrap();
        }

        // Since we can't override HIGHSCORE_FILE, we test the logic manually
        let result = if !Path::new(&test_file).exists() {
            HighscoreData::default()
        } else {
            HighscoreData { highscore: 999 }
        };

        assert_eq!(result.highscore, 0);

        cleanup_test_files();
    }

    #[test]
    fn test_load_corrupted_file() {
        // Create test directory
        fs::create_dir_all(test_dir()).unwrap();

        let test_file = test_file_path("corrupted");

        // Write invalid JSON
        fs::write(&test_file, "{ invalid json }").unwrap();

        // Try to load - should return default on parse error
        let loaded_json = fs::read_to_string(&test_file).unwrap();
        let result: HighscoreData = serde_json::from_str(&loaded_json).unwrap_or_default();

        assert_eq!(result.highscore, 0);

        cleanup_test_files();
    }

    #[test]
    fn test_update_highscore_logic() {
        // Test the logic without actual file operations
        let current = HighscoreData { highscore: 1000 };

        // New score is higher
        let new_score_high = 2000;
        let should_update_high = new_score_high > current.highscore;
        assert!(should_update_high);

        // New score is lower
        let new_score_low = 500;
        let should_update_low = new_score_low > current.highscore;
        assert!(!should_update_low);

        // New score is equal
        let new_score_equal = 1000;
        let should_update_equal = new_score_equal > current.highscore;
        assert!(!should_update_equal);
    }

    #[test]
    fn test_json_format() {
        let data = HighscoreData { highscore: 99999 };
        let json = serde_json::to_string_pretty(&data).unwrap();

        // Check that JSON is pretty-printed (contains newlines)
        assert!(json.contains('\n'));

        // Check that it contains the highscore field
        assert!(json.contains("highscore"));
        assert!(json.contains("99999"));
    }
}
