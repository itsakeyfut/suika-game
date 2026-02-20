//! Settings resource — user-configurable game preferences.
//!
//! Persisted to `save/settings.json` via [`crate::persistence`].

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// UI language choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    /// Japanese (日本語)
    #[default]
    Japanese,
    /// English
    English,
}

/// User-configurable settings, persisted to `save/settings.json`.
///
/// All fields have sensible defaults so new installations work without a save
/// file.  Use [`crate::persistence::load_settings`] to populate this resource
/// from disk at startup.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SettingsResource {
    /// BGM volume (0 = muted, 10 = full).  Default: 8 (80 %).
    pub bgm_volume: u8,
    /// Sound-effect volume (0 = muted, 10 = full).  Default: 8 (80 %).
    pub sfx_volume: u8,
    /// Whether particle / flash / shake visual effects are active.
    pub effects_enabled: bool,
    /// UI and text language.
    pub language: Language,
}

impl Default for SettingsResource {
    fn default() -> Self {
        Self {
            bgm_volume: 8,
            sfx_volume: 8,
            effects_enabled: true,
            language: Language::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_resource_default() {
        let s = SettingsResource::default();
        assert_eq!(s.bgm_volume, 8);
        assert_eq!(s.sfx_volume, 8);
        assert!(s.effects_enabled);
        assert_eq!(s.language, Language::Japanese);
    }

    #[test]
    fn test_language_default() {
        assert_eq!(Language::default(), Language::Japanese);
    }

    #[test]
    fn test_settings_resource_serde_roundtrip() {
        let original = SettingsResource {
            bgm_volume: 5,
            sfx_volume: 3,
            effects_enabled: false,
            language: Language::English,
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: SettingsResource = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.bgm_volume, 5);
        assert_eq!(deserialized.sfx_volume, 3);
        assert!(!deserialized.effects_enabled);
        assert_eq!(deserialized.language, Language::English);
    }
}
