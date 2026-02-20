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
pub struct AudioConfig {
    // --- BGM ---
    /// Volume for the title-screen BGM track (dB, 0 = full).
    #[serde(default = "default_bgm_title_volume")]
    pub bgm_title_volume: f32,
    /// Volume for the in-game BGM track (dB, 0 = full).
    #[serde(default = "default_bgm_game_volume")]
    pub bgm_game_volume: f32,
    /// Volume for the game-over BGM track (dB, 0 = full).
    #[serde(default = "default_bgm_gameover_volume")]
    pub bgm_gameover_volume: f32,
    /// Fade-out duration when switching away from any BGM track (seconds).
    #[serde(default = "default_bgm_fade_out_secs")]
    pub bgm_fade_out_secs: f32,
    /// Fade-in duration for the title BGM (seconds).
    #[serde(default = "default_bgm_title_fade_in_secs")]
    pub bgm_title_fade_in_secs: f32,
    /// Fade-in duration for the game BGM (seconds).
    #[serde(default = "default_bgm_game_fade_in_secs")]
    pub bgm_game_fade_in_secs: f32,

    // --- SFX ---
    /// Volume for the fruit-drop sound (dB, 0 = full).
    #[serde(default = "default_sfx_drop_volume")]
    pub sfx_drop_volume: f32,
    /// Volume for the small-fruit merge sound (dB, 0 = full).
    #[serde(default = "default_sfx_merge_small_volume")]
    pub sfx_merge_small_volume: f32,
    /// Volume for the medium-fruit merge sound (dB, 0 = full).
    #[serde(default = "default_sfx_merge_medium_volume")]
    pub sfx_merge_medium_volume: f32,
    /// Volume for the large-fruit merge sound (dB, 0 = full).
    #[serde(default = "default_sfx_merge_large_volume")]
    pub sfx_merge_large_volume: f32,
    /// Volume for the watermelon-merge fanfare (dB, 0 = full).
    #[serde(default = "default_sfx_watermelon_volume")]
    pub sfx_watermelon_volume: f32,
    /// Volume for the combo-chain sound (dB, 0 = full).
    #[serde(default = "default_sfx_combo_volume")]
    pub sfx_combo_volume: f32,
    /// Volume for the game-over sting (dB, 0 = full).
    #[serde(default = "default_sfx_gameover_volume")]
    pub sfx_gameover_volume: f32,
    /// Volume for UI button-click sounds (dB, 0 = full).
    #[serde(default = "default_sfx_button_click_volume")]
    pub sfx_button_click_volume: f32,
    /// Volume for UI button-hover sounds (dB, 0 = full).
    #[serde(default = "default_sfx_button_hover_volume")]
    pub sfx_button_hover_volume: f32,

    // --- SFX pitch (playback rate multiplier; 1.0 = original pitch) ---
    /// Playback rate for the small-fruit merge sound (Cherry, Strawberry, Grape).
    #[serde(default = "default_sfx_merge_small_pitch")]
    pub sfx_merge_small_pitch: f64,
    /// Playback rate for the medium-fruit merge sound (Dekopon through Pear).
    #[serde(default = "default_sfx_merge_medium_pitch")]
    pub sfx_merge_medium_pitch: f64,
    /// Playback rate for the large-fruit merge sound (Peach, Pineapple).
    #[serde(default = "default_sfx_merge_large_pitch")]
    pub sfx_merge_large_pitch: f64,
}

// Default values — these match the hard-coded constants that bgm.rs used
// before the config was introduced, so existing behaviour is preserved when
// the RON file is absent or a field is omitted.
pub const DEFAULT_BGM_TITLE_VOLUME: f32 = -4.0;
pub const DEFAULT_BGM_GAME_VOLUME: f32 = -8.0;
pub const DEFAULT_BGM_GAMEOVER_VOLUME: f32 = -6.0;
pub const DEFAULT_BGM_FADE_OUT_SECS: f32 = 0.5;
pub const DEFAULT_BGM_TITLE_FADE_IN_SECS: f32 = 0.3;
pub const DEFAULT_BGM_GAME_FADE_IN_SECS: f32 = 0.3;
pub const DEFAULT_SFX_DROP_VOLUME: f32 = 0.0;
pub const DEFAULT_SFX_MERGE_SMALL_VOLUME: f32 = 0.0;
pub const DEFAULT_SFX_MERGE_MEDIUM_VOLUME: f32 = 0.0;
pub const DEFAULT_SFX_MERGE_LARGE_VOLUME: f32 = 0.0;
pub const DEFAULT_SFX_WATERMELON_VOLUME: f32 = 0.0;
pub const DEFAULT_SFX_COMBO_VOLUME: f32 = 0.0;
pub const DEFAULT_SFX_GAMEOVER_VOLUME: f32 = 0.0;
pub const DEFAULT_SFX_BUTTON_CLICK_VOLUME: f32 = 0.0;
pub const DEFAULT_SFX_BUTTON_HOVER_VOLUME: f32 = 0.0;
pub const DEFAULT_SFX_MERGE_SMALL_PITCH: f64 = 1.2;
pub const DEFAULT_SFX_MERGE_MEDIUM_PITCH: f64 = 1.0;
pub const DEFAULT_SFX_MERGE_LARGE_PITCH: f64 = 0.8;

// serde requires function pointers for #[serde(default = "...")], so each
// constant is exposed through a thin forwarding function.
fn default_bgm_title_volume() -> f32 {
    DEFAULT_BGM_TITLE_VOLUME
}
fn default_bgm_game_volume() -> f32 {
    DEFAULT_BGM_GAME_VOLUME
}
fn default_bgm_gameover_volume() -> f32 {
    DEFAULT_BGM_GAMEOVER_VOLUME
}
fn default_bgm_fade_out_secs() -> f32 {
    DEFAULT_BGM_FADE_OUT_SECS
}
fn default_bgm_title_fade_in_secs() -> f32 {
    DEFAULT_BGM_TITLE_FADE_IN_SECS
}
fn default_bgm_game_fade_in_secs() -> f32 {
    DEFAULT_BGM_GAME_FADE_IN_SECS
}
fn default_sfx_drop_volume() -> f32 {
    DEFAULT_SFX_DROP_VOLUME
}
fn default_sfx_merge_small_volume() -> f32 {
    DEFAULT_SFX_MERGE_SMALL_VOLUME
}
fn default_sfx_merge_medium_volume() -> f32 {
    DEFAULT_SFX_MERGE_MEDIUM_VOLUME
}
fn default_sfx_merge_large_volume() -> f32 {
    DEFAULT_SFX_MERGE_LARGE_VOLUME
}
fn default_sfx_watermelon_volume() -> f32 {
    DEFAULT_SFX_WATERMELON_VOLUME
}
fn default_sfx_combo_volume() -> f32 {
    DEFAULT_SFX_COMBO_VOLUME
}
fn default_sfx_gameover_volume() -> f32 {
    DEFAULT_SFX_GAMEOVER_VOLUME
}
fn default_sfx_button_click_volume() -> f32 {
    DEFAULT_SFX_BUTTON_CLICK_VOLUME
}
fn default_sfx_button_hover_volume() -> f32 {
    DEFAULT_SFX_BUTTON_HOVER_VOLUME
}
fn default_sfx_merge_small_pitch() -> f64 {
    DEFAULT_SFX_MERGE_SMALL_PITCH
}
fn default_sfx_merge_medium_pitch() -> f64 {
    DEFAULT_SFX_MERGE_MEDIUM_PITCH
}
fn default_sfx_merge_large_pitch() -> f64 {
    DEFAULT_SFX_MERGE_LARGE_PITCH
}

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
        ron::de::from_bytes(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
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
)
"#;
        let cfg: AudioConfig = ron::de::from_str(ron_str).expect("RON parse must succeed");
        assert_eq!(cfg.bgm_title_volume, -3.0);
        assert_eq!(cfg.bgm_game_volume, -6.0);
        assert_eq!(cfg.bgm_fade_out_secs, 0.8);
        assert_eq!(cfg.sfx_watermelon_volume, 3.0);
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
}
