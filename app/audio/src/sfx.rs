//! Sound-effect (SFX) playback systems.
//!
//! This module contains Bevy systems that play one-shot sound effects in
//! response to game events.  Each system is driven by an event reader so it
//! fires only when something actually happens, adding no per-frame overhead
//! during quiet moments.
//!
//! # Registered systems
//!
//! | System | Trigger | Description |
//! |--------|---------|-------------|
//! | [`play_merge_sfx`] | [`FruitMergeEvent`] | Plays a size-appropriate merge sound with pitch adjustment |
//!
//! # Pitch mapping
//!
//! | Fruit group | Handle | Default pitch |
//! |-------------|--------|---------------|
//! | Cherry, Strawberry, Grape | `merge_small.wav` | 1.2× |
//! | Dekopon, Persimmon, Apple, Pear | `merge_medium.wav` | 1.0× |
//! | Peach, Pineapple | `merge_large.wav` | 0.8× |
//! | Melon (→ Watermelon) | `watermelon.wav` | 1.0× (no pitch shift) |
//!
//! All pitch and volume values are read from [`AudioConfig`] at call time, so
//! they can be adjusted via hot-reload in `assets/config/audio.ron`.

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use suika_game_core::events::FruitMergeEvent;
use suika_game_core::fruit::FruitType;

use crate::config::{AudioConfig, AudioConfigHandle};
use crate::handles::SfxHandles;

// ---------------------------------------------------------------------------
// Merge SFX category
// ---------------------------------------------------------------------------

/// Internal category used to select the right handle and pitch for a merge.
enum MergeSfxCategory {
    /// Cherry, Strawberry, Grape — high-pitched pop.
    Small,
    /// Dekopon, Persimmon, Apple, Pear — mid-pitched pop.
    Medium,
    /// Peach, Pineapple — low-pitched thud.
    Large,
    /// Melon → Watermelon — special fanfare, no pitch shift.
    Watermelon,
}

impl MergeSfxCategory {
    /// Classifies a [`FruitType`] into the appropriate SFX category.
    ///
    /// `FruitType::Watermelon` is included for completeness but cannot
    /// appear in a [`FruitMergeEvent`] in practice (Watermelon is the final
    /// evolution and has no further merge target).
    fn from_fruit(fruit: FruitType) -> Self {
        match fruit {
            FruitType::Cherry | FruitType::Strawberry | FruitType::Grape => Self::Small,
            FruitType::Dekopon | FruitType::Persimmon | FruitType::Apple | FruitType::Pear => {
                Self::Medium
            }
            FruitType::Peach | FruitType::Pineapple => Self::Large,
            // Two Melons merging → Watermelon fanfare.
            FruitType::Melon | FruitType::Watermelon => Self::Watermelon,
        }
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Plays a merge sound effect in response to each [`FruitMergeEvent`].
///
/// Selects one of three merge clips (`merge_small`, `merge_medium`,
/// `merge_large`) based on the fruit size, then applies a configurable
/// playback-rate (pitch) shift.  When two Melons merge into a Watermelon,
/// the special `watermelon.wav` fanfare is played at full pitch instead.
///
/// Volume and pitch values are read from [`AudioConfig`] at call time, so
/// they take effect immediately on the next merge after editing
/// `assets/config/audio.ron` (hot-reload).
///
/// # Parameters
///
/// - `merge_events` — event reader for [`FruitMergeEvent`]s produced by the
///   fruit collision system.
/// - `audio` — the global [`bevy_kira_audio`] audio channel.
/// - `sfx_handles` — optional resource holding pre-loaded SFX handles; the
///   system is a no-op if handles are not yet available.
/// - `audio_config_handle` / `audio_config_assets` — used to resolve the
///   current [`AudioConfig`]; falls back to `AudioConfig::default()` while
///   the asset loads.
pub fn play_merge_sfx(
    mut merge_events: MessageReader<FruitMergeEvent>,
    audio: Res<Audio>,
    sfx_handles: Option<Res<SfxHandles>>,
    audio_config_handle: Option<Res<AudioConfigHandle>>,
    audio_config_assets: Res<Assets<AudioConfig>>,
) {
    let Some(sfx_handles) = sfx_handles else {
        return;
    };

    // Resolve config, falling back to defaults while the asset loads.
    let default_cfg = AudioConfig::default();
    let cfg = audio_config_handle
        .as_ref()
        .and_then(|h| audio_config_assets.get(&h.0))
        .unwrap_or(&default_cfg);

    for event in merge_events.read() {
        match MergeSfxCategory::from_fruit(event.fruit_type) {
            MergeSfxCategory::Small => {
                audio
                    .play(sfx_handles.merge_small.clone())
                    .with_volume(cfg.sfx_merge_small_volume)
                    .with_playback_rate(cfg.sfx_merge_small_pitch);
            }
            MergeSfxCategory::Medium => {
                audio
                    .play(sfx_handles.merge_medium.clone())
                    .with_volume(cfg.sfx_merge_medium_volume)
                    .with_playback_rate(cfg.sfx_merge_medium_pitch);
            }
            MergeSfxCategory::Large => {
                audio
                    .play(sfx_handles.merge_large.clone())
                    .with_volume(cfg.sfx_merge_large_volume)
                    .with_playback_rate(cfg.sfx_merge_large_pitch);
            }
            MergeSfxCategory::Watermelon => {
                // Special fanfare — no pitch shift, played at full original pitch.
                audio
                    .play(sfx_handles.watermelon.clone())
                    .with_volume(cfg.sfx_watermelon_volume);
                info!("Watermelon! Playing fanfare SFX");
            }
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
    fn test_merge_sfx_category_small_fruits() {
        assert!(matches!(
            MergeSfxCategory::from_fruit(FruitType::Cherry),
            MergeSfxCategory::Small
        ));
        assert!(matches!(
            MergeSfxCategory::from_fruit(FruitType::Strawberry),
            MergeSfxCategory::Small
        ));
        assert!(matches!(
            MergeSfxCategory::from_fruit(FruitType::Grape),
            MergeSfxCategory::Small
        ));
    }

    #[test]
    fn test_merge_sfx_category_medium_fruits() {
        assert!(matches!(
            MergeSfxCategory::from_fruit(FruitType::Dekopon),
            MergeSfxCategory::Medium
        ));
        assert!(matches!(
            MergeSfxCategory::from_fruit(FruitType::Persimmon),
            MergeSfxCategory::Medium
        ));
        assert!(matches!(
            MergeSfxCategory::from_fruit(FruitType::Apple),
            MergeSfxCategory::Medium
        ));
        assert!(matches!(
            MergeSfxCategory::from_fruit(FruitType::Pear),
            MergeSfxCategory::Medium
        ));
    }

    #[test]
    fn test_merge_sfx_category_large_fruits() {
        assert!(matches!(
            MergeSfxCategory::from_fruit(FruitType::Peach),
            MergeSfxCategory::Large
        ));
        assert!(matches!(
            MergeSfxCategory::from_fruit(FruitType::Pineapple),
            MergeSfxCategory::Large
        ));
    }

    #[test]
    fn test_merge_sfx_category_watermelon() {
        // Two Melons merging triggers the Watermelon fanfare.
        assert!(matches!(
            MergeSfxCategory::from_fruit(FruitType::Melon),
            MergeSfxCategory::Watermelon
        ));
        // Watermelon itself is also handled defensively (cannot appear in a
        // real FruitMergeEvent, but the match arm covers it explicitly).
        assert!(matches!(
            MergeSfxCategory::from_fruit(FruitType::Watermelon),
            MergeSfxCategory::Watermelon
        ));
    }

    #[test]
    fn test_all_fruit_types_have_a_category() {
        let all = [
            FruitType::Cherry,
            FruitType::Strawberry,
            FruitType::Grape,
            FruitType::Dekopon,
            FruitType::Persimmon,
            FruitType::Apple,
            FruitType::Pear,
            FruitType::Peach,
            FruitType::Pineapple,
            FruitType::Melon,
            FruitType::Watermelon,
        ];
        // Simply calling from_fruit on each type must not panic.
        for fruit in all {
            let _ = MergeSfxCategory::from_fruit(fruit);
        }
    }

    #[test]
    fn test_default_pitch_values() {
        let cfg = AudioConfig::default();
        assert!(
            cfg.sfx_merge_small_pitch > cfg.sfx_merge_medium_pitch,
            "small pitch must be higher than medium"
        );
        assert!(
            cfg.sfx_merge_medium_pitch > cfg.sfx_merge_large_pitch,
            "medium pitch must be higher than large"
        );
        assert!(
            cfg.sfx_merge_large_pitch > 0.0,
            "large pitch must be positive"
        );
    }
}
