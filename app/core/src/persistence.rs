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

/// Saves the highscore data to a JSON file in the specified directory
///
/// This function will:
/// 1. Create the save directory if it doesn't exist
/// 2. Serialize the highscore data to pretty-printed JSON
/// 3. Write the JSON to `{save_dir}/highscore.json`
///
/// # Arguments
///
/// * `data` - The highscore data to save
/// * `save_dir` - The directory where the highscore file should be saved
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
/// # use std::path::Path;
/// let data = HighscoreData { highscore: 10000 };
/// save_highscore(&data, Path::new("save")).expect("Failed to save highscore");
/// ```
pub fn save_highscore(
    data: &HighscoreData,
    save_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create save directory if it doesn't exist
    fs::create_dir_all(save_dir)?;

    // Serialize to pretty-printed JSON
    let json = serde_json::to_string_pretty(data)?;

    // Write to file
    let file_path = save_dir.join("highscore.json");
    fs::write(file_path, json)?;

    Ok(())
}

/// Loads the highscore data from a JSON file in the specified directory
///
/// This function will:
/// 1. Check if the highscore file exists in the directory
/// 2. If it exists, read and deserialize the JSON
/// 3. If it doesn't exist or there's an error, return default (0)
///
/// # Arguments
///
/// * `save_dir` - The directory where the highscore file is located
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
/// # use std::path::Path;
/// let data = load_highscore(Path::new("save"));
/// println!("Current highscore: {}", data.highscore);
/// ```
pub fn load_highscore(save_dir: &Path) -> HighscoreData {
    let file_path = save_dir.join("highscore.json");

    // Return default if file doesn't exist
    if !file_path.exists() {
        return HighscoreData::default();
    }

    // Try to read and deserialize the file
    match fs::read_to_string(&file_path) {
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
/// 1. Loads the current highscore from the specified directory
/// 2. Compares it with the new score
/// 3. Saves the new score if it's higher
/// 4. Returns whether a new highscore was set
///
/// # Arguments
///
/// * `new_score` - The score to potentially save as the new highscore
/// * `save_dir` - The directory where the highscore file is located
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
/// # use std::path::Path;
/// match update_highscore(15000, Path::new("save")) {
///     Ok(true) => println!("New highscore!"),
///     Ok(false) => println!("Try again!"),
///     Err(e) => eprintln!("Failed to save: {}", e),
/// }
/// ```
pub fn update_highscore(
    new_score: u32,
    save_dir: &Path,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut data = load_highscore(save_dir);

    if new_score > data.highscore {
        data.highscore = new_score;
        save_highscore(&data, save_dir)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path();

        let data = HighscoreData { highscore: 54321 };

        // Save using the actual function
        save_highscore(&data, save_path).unwrap();

        // Load using the actual function
        let loaded = load_highscore(save_path);

        assert_eq!(loaded.highscore, 54321);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path();

        // Load from directory with no file should return default
        let result = load_highscore(save_path);

        assert_eq!(result.highscore, 0);
    }

    #[test]
    fn test_load_corrupted_file() {
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path();
        let file_path = save_path.join("highscore.json");

        // Write invalid JSON
        fs::write(&file_path, "{ invalid json }").unwrap();

        // Load should return default on parse error
        let result = load_highscore(save_path);

        assert_eq!(result.highscore, 0);
    }

    #[test]
    fn test_update_highscore_new_high() {
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path();

        // Set initial highscore
        let initial = HighscoreData { highscore: 1000 };
        save_highscore(&initial, save_path).unwrap();

        // Update with higher score
        let updated = update_highscore(2000, save_path).unwrap();
        assert!(updated);

        // Verify the highscore was saved
        let loaded = load_highscore(save_path);
        assert_eq!(loaded.highscore, 2000);
    }

    #[test]
    fn test_update_highscore_not_higher() {
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path();

        // Set initial highscore
        let initial = HighscoreData { highscore: 1000 };
        save_highscore(&initial, save_path).unwrap();

        // Update with lower score
        let updated = update_highscore(500, save_path).unwrap();
        assert!(!updated);

        // Verify the highscore was not changed
        let loaded = load_highscore(save_path);
        assert_eq!(loaded.highscore, 1000);

        // Update with equal score
        let updated_equal = update_highscore(1000, save_path).unwrap();
        assert!(!updated_equal);
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
