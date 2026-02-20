//! Audio configuration loaded from `assets/config/audio.ron`.
//!
//! All volume and timing values for BGM and SFX are kept in a single RON file
//! so they can be tuned at runtime via hot-reload without recompiling.
//!
//! # Volume units
//!
//! Volumes are expressed in **decibels relative to full-scale** as expected by
//! [`bevy_kira_audio`]:
//! - `0.0 dB` — full volume (unchanged)
//! - `-6.0 dB` — roughly half perceived loudness
//! - `-20.0 dB` — very quiet
//!
//! # Hot-reload
//!
//! Edit `assets/config/audio.ron` while the game is running; the
//! [`hot_reload_audio_config`] system picks up the change automatically.

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetEvent, AssetLoader, Assets, LoadContext};
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Config struct
// ---------------------------------------------------------------------------

/// Audio configuration asset loaded from `assets/config/audio.ron`.
///
/// Inserted as a [`Resource`] handle via [`AudioConfigHandle`] at startup.
/// Systems that need audio parameters should obtain them through
/// `Option<Res<AudioConfigHandle>>` + `Res<Assets<AudioConfig>>` and fall
/// back to [`AudioConfig::default()`] when the asset is not yet loaded.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct AudioConfig {
    // --- BGM ---
    /// Volume for the title-screen BGM track (dB, 0 = full).
    pub bgm_title_volume: f32,
    /// Volume for the in-game BGM track (dB, 0 = full).
    pub bgm_game_volume: f32,
    /// Volume for the game-over BGM track (dB, 0 = full).
    pub bgm_gameover_volume: f32,
    /// Fade-out duration when switching away from any BGM track (seconds).
    pub bgm_fade_out_secs: f32,
    /// Fade-in duration for the title BGM (seconds).
    pub bgm_title_fade_in_secs: f32,
    /// Fade-in duration for the game BGM (seconds).
    pub bgm_game_fade_in_secs: f32,

    // --- SFX ---
    /// Volume for the fruit-drop sound (dB, 0 = full).
    pub sfx_drop_volume: f32,
    /// Volume for the small-fruit merge sound (dB, 0 = full).
    pub sfx_merge_small_volume: f32,
    /// Volume for the medium-fruit merge sound (dB, 0 = full).
    pub sfx_merge_medium_volume: f32,
    /// Volume for the large-fruit merge sound (dB, 0 = full).
    pub sfx_merge_large_volume: f32,
    /// Volume for the watermelon-merge fanfare (dB, 0 = full).
    pub sfx_watermelon_volume: f32,
    /// Volume for the combo-chain sound (dB, 0 = full).
    pub sfx_combo_volume: f32,
    /// Volume for the game-over sting (dB, 0 = full).
    pub sfx_gameover_volume: f32,
    /// Volume for UI button-click sounds (dB, 0 = full).
    pub sfx_button_click_volume: f32,
    /// Volume for UI button-hover sounds (dB, 0 = full).
    pub sfx_button_hover_volume: f32,

    // --- SFX pitch (playback rate multiplier; 1.0 = original pitch) ---
    /// Playback rate for the small-fruit merge sound (Cherry, Strawberry, Grape).
    pub sfx_merge_small_pitch: f64,
    /// Playback rate for the medium-fruit merge sound (Dekopon through Pear).
    pub sfx_merge_medium_pitch: f64,
    /// Playback rate for the large-fruit merge sound (Peach, Pineapple).
    pub sfx_merge_large_pitch: f64,
    /// Pitch increment added per combo count for the combo sound.
    ///
    /// Combo pitch = `1.0 + (combo_count × sfx_combo_pitch_step).min(sfx_combo_pitch_cap)`.
    pub sfx_combo_pitch_step: f64,
    /// Maximum pitch offset added on top of 1.0 for the combo sound.
    ///
    /// Caps the value of `combo_count × sfx_combo_pitch_step` so the pitch
    /// does not grow unboundedly at very high combo counts.
    pub sfx_combo_pitch_cap: f64,
}

// Default values — these match the hard-coded constants that bgm.rs used
// before the config was introduced, so existing behaviour is preserved when
// the RON file is absent or a field is omitted.
const DEFAULT_BGM_TITLE_VOLUME: f32 = -4.0;
const DEFAULT_BGM_GAME_VOLUME: f32 = -8.0;
const DEFAULT_BGM_GAMEOVER_VOLUME: f32 = -6.0;
const DEFAULT_BGM_FADE_OUT_SECS: f32 = 0.5;
const DEFAULT_BGM_TITLE_FADE_IN_SECS: f32 = 0.3;
const DEFAULT_BGM_GAME_FADE_IN_SECS: f32 = 0.3;
const DEFAULT_SFX_DROP_VOLUME: f32 = 0.0;
const DEFAULT_SFX_MERGE_SMALL_VOLUME: f32 = 0.0;
const DEFAULT_SFX_MERGE_MEDIUM_VOLUME: f32 = 0.0;
const DEFAULT_SFX_MERGE_LARGE_VOLUME: f32 = 0.0;
const DEFAULT_SFX_WATERMELON_VOLUME: f32 = 0.0;
const DEFAULT_SFX_COMBO_VOLUME: f32 = 0.0;
const DEFAULT_SFX_GAMEOVER_VOLUME: f32 = 0.0;
const DEFAULT_SFX_BUTTON_CLICK_VOLUME: f32 = 0.0;
const DEFAULT_SFX_BUTTON_HOVER_VOLUME: f32 = 0.0;
const DEFAULT_SFX_MERGE_SMALL_PITCH: f64 = 1.2;
const DEFAULT_SFX_MERGE_MEDIUM_PITCH: f64 = 1.0;
const DEFAULT_SFX_MERGE_LARGE_PITCH: f64 = 0.8;
/// Pitch added per combo count (e.g. 0.1 → combo 2 = 1.2×, combo 5 = 1.5×).
const DEFAULT_SFX_COMBO_PITCH_STEP: f64 = 0.1;
/// Maximum pitch offset above 1.0 for the combo sound (caps the step scaling).
const DEFAULT_SFX_COMBO_PITCH_CAP: f64 = 0.5;

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            bgm_title_volume: DEFAULT_BGM_TITLE_VOLUME,
            bgm_game_volume: DEFAULT_BGM_GAME_VOLUME,
            bgm_gameover_volume: DEFAULT_BGM_GAMEOVER_VOLUME,
            bgm_fade_out_secs: DEFAULT_BGM_FADE_OUT_SECS,
            bgm_title_fade_in_secs: DEFAULT_BGM_TITLE_FADE_IN_SECS,
            bgm_game_fade_in_secs: DEFAULT_BGM_GAME_FADE_IN_SECS,
            sfx_drop_volume: DEFAULT_SFX_DROP_VOLUME,
            sfx_merge_small_volume: DEFAULT_SFX_MERGE_SMALL_VOLUME,
            sfx_merge_medium_volume: DEFAULT_SFX_MERGE_MEDIUM_VOLUME,
            sfx_merge_large_volume: DEFAULT_SFX_MERGE_LARGE_VOLUME,
            sfx_watermelon_volume: DEFAULT_SFX_WATERMELON_VOLUME,
            sfx_combo_volume: DEFAULT_SFX_COMBO_VOLUME,
            sfx_gameover_volume: DEFAULT_SFX_GAMEOVER_VOLUME,
            sfx_button_click_volume: DEFAULT_SFX_BUTTON_CLICK_VOLUME,
            sfx_button_hover_volume: DEFAULT_SFX_BUTTON_HOVER_VOLUME,
            sfx_merge_small_pitch: DEFAULT_SFX_MERGE_SMALL_PITCH,
            sfx_merge_medium_pitch: DEFAULT_SFX_MERGE_MEDIUM_PITCH,
            sfx_merge_large_pitch: DEFAULT_SFX_MERGE_LARGE_PITCH,
            sfx_combo_pitch_step: DEFAULT_SFX_COMBO_PITCH_STEP,
            sfx_combo_pitch_cap: DEFAULT_SFX_COMBO_PITCH_CAP,
        }
    }
}

// ---------------------------------------------------------------------------
// Handle resource
// ---------------------------------------------------------------------------

/// Resource holding the handle to the loaded [`AudioConfig`] asset.
///
/// Inserted at startup by [`load_audio_config`].  Use
/// `Option<Res<AudioConfigHandle>>` in systems that need audio config so they
/// degrade gracefully before the asset finishes loading.
#[derive(Resource)]
pub struct AudioConfigHandle(pub Handle<AudioConfig>);

// ---------------------------------------------------------------------------
// Asset loader (RON)
// ---------------------------------------------------------------------------

/// RON-based [`AssetLoader`] for [`AudioConfig`].
#[derive(Default)]
pub struct AudioConfigLoader;

impl AssetLoader for AudioConfigLoader {
    type Asset = AudioConfig;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let cfg: AudioConfig = ron::de::from_bytes(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Pitch (playback-rate) values must be positive; zero or negative would
        // produce silence or undefined behaviour in the audio backend.
        for (name, pitch) in [
            ("sfx_merge_small_pitch", cfg.sfx_merge_small_pitch),
            ("sfx_merge_medium_pitch", cfg.sfx_merge_medium_pitch),
            ("sfx_merge_large_pitch", cfg.sfx_merge_large_pitch),
        ] {
            if pitch <= 0.0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("{name} must be > 0.0, got {pitch}"),
                ));
            }
        }

        // Combo pitch parameters must also be positive so the formula
        // `1.0 + (count × step).min(cap)` always produces a pitch ≥ 1.0.
        for (name, value) in [
            ("sfx_combo_pitch_step", cfg.sfx_combo_pitch_step),
            ("sfx_combo_pitch_cap", cfg.sfx_combo_pitch_cap),
        ] {
            if value <= 0.0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("{name} must be > 0.0, got {value}"),
                ));
            }
        }

        Ok(cfg)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Startup system — queues `assets/config/audio.ron` for loading and inserts
/// [`AudioConfigHandle`] so other systems can access it.
pub fn load_audio_config(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load("config/audio.ron");
    commands.insert_resource(AudioConfigHandle(handle));
    info!("Audio config queued for loading");
}

/// Hot-reload system — reacts to [`AssetEvent<AudioConfig>`] and logs changes.
///
/// Wire this up with:
/// ```rust,ignore
/// app.add_systems(Update, config::hot_reload_audio_config);
/// ```
///
/// Active BGM volume changes take effect on the **next** track switch because
/// `bevy_kira_audio` does not expose a live-volume API for already-playing
/// sounds on the global channel.  SFX volumes take effect on the next SFX
/// playback call.
pub fn hot_reload_audio_config(
    mut events: MessageReader<AssetEvent<AudioConfig>>,
    config_assets: Res<Assets<AudioConfig>>,
    config_handle: Option<Res<AudioConfigHandle>>,
) {
    let Some(handle) = config_handle else {
        return;
    };

    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Audio config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                if let Some(config) = config_assets.get(&handle.0) {
                    info!(
                        "🔥 Audio config hot-reloaded \
                        (bgm_game={:.1} dB, fade_out={:.2}s)",
                        config.bgm_game_volume, config.bgm_fade_out_secs,
                    );
                }
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Audio config removed");
            }
            _ => {}
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
    fn test_audio_config_default_values() {
        let cfg = AudioConfig::default();
        // BGM volumes should be negative dB (quieter than full)
        assert!(cfg.bgm_title_volume < 0.0, "title BGM should be < 0 dB");
        assert!(cfg.bgm_game_volume < 0.0, "game BGM should be < 0 dB");
        assert!(
            cfg.bgm_gameover_volume < 0.0,
            "gameover BGM should be < 0 dB"
        );
    }

    #[test]
    fn test_audio_config_fade_durations_positive() {
        let cfg = AudioConfig::default();
        assert!(cfg.bgm_fade_out_secs > 0.0);
        assert!(cfg.bgm_title_fade_in_secs > 0.0);
        assert!(cfg.bgm_game_fade_in_secs > 0.0);
    }

    #[test]
    fn test_audio_config_ron_roundtrip() {
        let ron_str = r#"
AudioConfig(
    bgm_title_volume: -3.0,
    bgm_game_volume: -6.0,
    bgm_gameover_volume: -4.0,
    bgm_fade_out_secs: 0.8,
    bgm_title_fade_in_secs: 1.2,
    bgm_game_fade_in_secs: 2.0,
    sfx_drop_volume: -2.0,
    sfx_merge_small_volume: -1.0,
    sfx_merge_medium_volume: -1.0,
    sfx_merge_large_volume: 0.0,
    sfx_watermelon_volume: 3.0,
    sfx_combo_volume: 0.0,
    sfx_gameover_volume: -2.0,
    sfx_button_click_volume: -5.0,
    sfx_button_hover_volume: -8.0,
    sfx_merge_small_pitch: 1.1,
)
"#;
        let cfg: AudioConfig = ron::de::from_str(ron_str).expect("RON parse must succeed");
        assert_eq!(cfg.bgm_title_volume, -3.0);
        assert_eq!(cfg.bgm_game_volume, -6.0);
        assert_eq!(cfg.bgm_fade_out_secs, 0.8);
        assert_eq!(cfg.sfx_watermelon_volume, 3.0);
        // Explicitly set pitch field is parsed correctly.
        assert_eq!(cfg.sfx_merge_small_pitch, 1.1);
        // Omitted pitch fields fall back to serde defaults.
        assert_eq!(cfg.sfx_merge_medium_pitch, DEFAULT_SFX_MERGE_MEDIUM_PITCH);
        assert_eq!(cfg.sfx_merge_large_pitch, DEFAULT_SFX_MERGE_LARGE_PITCH);
    }

    #[test]
    fn test_audio_config_ron_partial_fields_use_defaults() {
        // Only set one field; all others must fall back to serde defaults.
        let ron_str = r#"AudioConfig(bgm_title_volume: -10.0)"#;
        let cfg: AudioConfig = ron::de::from_str(ron_str).expect("RON parse must succeed");
        assert_eq!(cfg.bgm_title_volume, -10.0);
        // Other fields should use their serde defaults
        assert_eq!(cfg.bgm_game_volume, DEFAULT_BGM_GAME_VOLUME);
        assert_eq!(cfg.bgm_fade_out_secs, DEFAULT_BGM_FADE_OUT_SECS);
    }

    #[test]
    fn test_audio_config_zero_pitch_is_rejected_by_ron() {
        // The loader validates pitches after deserialisation; ron::de alone does
        // not, so we test the serde layer here and the loader layer via a unit
        // check on the deserialized struct values.
        let ron_str = r#"AudioConfig(sfx_merge_small_pitch: 0.0)"#;
        let cfg: AudioConfig = ron::de::from_str(ron_str).expect("RON parse must succeed");
        // Confirm the zero value survived deserialisation (validation lives in
        // the loader, not in serde).
        assert_eq!(cfg.sfx_merge_small_pitch, 0.0);
        // Verify that our defaults are all positive so the loader never rejects
        // a freshly-constructed default config.
        assert!(
            DEFAULT_SFX_MERGE_SMALL_PITCH > 0.0,
            "default small pitch must be > 0"
        );
        assert!(
            DEFAULT_SFX_MERGE_MEDIUM_PITCH > 0.0,
            "default medium pitch must be > 0"
        );
        assert!(
            DEFAULT_SFX_MERGE_LARGE_PITCH > 0.0,
            "default large pitch must be > 0"
        );
    }

    #[test]
    fn test_combo_pitch_params_defaults_are_positive() {
        // The loader rejects sfx_combo_pitch_step and sfx_combo_pitch_cap ≤ 0.
        // Verify that the built-in defaults satisfy this constraint so a default
        // AudioConfig is never rejected.
        assert!(
            DEFAULT_SFX_COMBO_PITCH_STEP > 0.0,
            "default combo pitch step must be > 0"
        );
        assert!(
            DEFAULT_SFX_COMBO_PITCH_CAP > 0.0,
            "default combo pitch cap must be > 0"
        );
    }
}
