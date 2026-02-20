//! Game SFX: merge, combo, and game-over sounds.

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use suika_game_core::events::{FruitMergeEvent, ScoreEarnedEvent};

use super::MergeSfxCategory;
use crate::channels::SfxChannel;
use crate::config::{AudioConfig, AudioConfigHandle};
use crate::handles::SfxHandles;

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
pub fn play_merge_sfx(
    mut merge_events: MessageReader<FruitMergeEvent>,
    sfx_channel: Res<AudioChannel<SfxChannel>>,
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
                sfx_channel
                    .play(sfx_handles.merge_small.clone())
                    .with_volume(cfg.sfx_merge_small_volume)
                    .with_playback_rate(cfg.sfx_merge_small_pitch);
            }
            MergeSfxCategory::Medium => {
                sfx_channel
                    .play(sfx_handles.merge_medium.clone())
                    .with_volume(cfg.sfx_merge_medium_volume)
                    .with_playback_rate(cfg.sfx_merge_medium_pitch);
            }
            MergeSfxCategory::Large => {
                sfx_channel
                    .play(sfx_handles.merge_large.clone())
                    .with_volume(cfg.sfx_merge_large_volume)
                    .with_playback_rate(cfg.sfx_merge_large_pitch);
            }
            MergeSfxCategory::Watermelon => {
                // Special fanfare — no pitch shift, played at full original pitch.
                sfx_channel
                    .play(sfx_handles.watermelon.clone())
                    .with_volume(cfg.sfx_watermelon_volume);
                info!("Watermelon! Playing fanfare SFX");
            }
        }
    }
}

/// Plays the combo sound effect whenever a scoring merge is part of a combo.
///
/// A combo is defined as `combo_count >= 2` in [`ScoreEarnedEvent`].  The
/// pitch rises with each additional combo step, capped at a configurable
/// maximum, so longer chains sound increasingly energetic.
///
/// **Pitch formula:**
/// ```text
/// pitch = 1.0 + (combo_count × sfx_combo_pitch_step).min(sfx_combo_pitch_cap)
/// ```
/// With defaults: combo 2 → 1.2×, combo 3 → 1.3×, combo 5+ → 1.5×.
pub fn play_combo_sfx(
    mut score_events: MessageReader<ScoreEarnedEvent>,
    sfx_channel: Res<AudioChannel<SfxChannel>>,
    sfx_handles: Option<Res<SfxHandles>>,
    audio_config_handle: Option<Res<AudioConfigHandle>>,
    audio_config_assets: Res<Assets<AudioConfig>>,
) {
    let Some(sfx_handles) = sfx_handles else {
        return;
    };

    let default_cfg = AudioConfig::default();
    let cfg = audio_config_handle
        .as_ref()
        .and_then(|h| audio_config_assets.get(&h.0))
        .unwrap_or(&default_cfg);

    for event in score_events.read() {
        // Only play the combo sound when a real combo is in progress.
        if event.combo_count < 2 {
            continue;
        }

        let pitch_offset =
            (event.combo_count as f64 * cfg.sfx_combo_pitch_step).min(cfg.sfx_combo_pitch_cap);
        // Guard against misconfigured negative offsets; pitch must stay > 0.
        let pitch = (1.0_f64 + pitch_offset).max(0.1);

        sfx_channel
            .play(sfx_handles.combo.clone())
            .with_volume(cfg.sfx_combo_volume)
            .with_playback_rate(pitch);
    }
}

/// Plays the game-over sound effect once when the game transitions to
/// [`AppState::GameOver`].
///
/// This system is scheduled on [`OnEnter(AppState::GameOver)`] so it fires
/// exactly once per game-over, regardless of frame rate.
pub fn play_gameover_sfx(
    sfx_channel: Res<AudioChannel<SfxChannel>>,
    sfx_handles: Option<Res<SfxHandles>>,
    audio_config_handle: Option<Res<AudioConfigHandle>>,
    audio_config_assets: Res<Assets<AudioConfig>>,
) {
    let Some(sfx_handles) = sfx_handles else {
        return;
    };

    let default_cfg = AudioConfig::default();
    let cfg = audio_config_handle
        .as_ref()
        .and_then(|h| audio_config_assets.get(&h.0))
        .unwrap_or(&default_cfg);

    sfx_channel
        .play(sfx_handles.gameover.clone())
        .with_volume(cfg.sfx_gameover_volume);

    info!("Game-over SFX playing");
}

#[cfg(test)]
mod tests {
    use super::*;
    use suika_game_core::fruit::FruitType;

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
        assert!(matches!(
            MergeSfxCategory::from_fruit(FruitType::Melon),
            MergeSfxCategory::Watermelon
        ));
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

    #[test]
    fn test_combo_pitch_formula_at_various_counts() {
        let cfg = AudioConfig::default();

        // Helper that mirrors the in-system formula.
        let combo_pitch = |count: u32| -> f64 {
            1.0 + (count as f64 * cfg.sfx_combo_pitch_step).min(cfg.sfx_combo_pitch_cap)
        };

        // combo_count = 1 is skipped by the system guard (`< 2`), but the
        // formula would yield 1.1× if it were evaluated.
        assert!((combo_pitch(1) - 1.1_f64).abs() < f64::EPSILON);

        // Pitch increases with combo count.
        assert!(
            combo_pitch(3) > combo_pitch(2),
            "pitch must increase with combo"
        );

        // Pitch caps at 1.0 + sfx_combo_pitch_cap.
        let max = 1.0 + cfg.sfx_combo_pitch_cap;
        assert!(
            (combo_pitch(100) - max).abs() < f64::EPSILON,
            "pitch must be capped at {max}"
        );
    }

    #[test]
    fn test_combo_pitch_step_and_cap_are_positive() {
        let cfg = AudioConfig::default();
        assert!(
            cfg.sfx_combo_pitch_step > 0.0,
            "pitch step must be positive"
        );
        assert!(cfg.sfx_combo_pitch_cap > 0.0, "pitch cap must be positive");
    }
}
