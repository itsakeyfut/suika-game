//! Sound-effect (SFX) playback systems.
//!
//! This module contains Bevy systems that play one-shot sound effects in
//! response to game events.  Each system is driven by an event reader so it
//! fires only when something actually happens, adding no per-frame overhead
//! during quiet moments.
//!
//! # Registered systems
//!
//! | System | Schedule | Trigger | Description |
//! |--------|----------|---------|-------------|
//! | [`play_merge_sfx`]   | `Update`                  | [`FruitMergeEvent`]   | Size-appropriate merge sound with pitch adjustment |
//! | [`play_combo_sfx`]   | `Update`                  | [`ScoreEarnedEvent`]  | Combo chime with rising pitch |
//! | [`play_gameover_sfx`]| `OnEnter(GameOver)`       | state transition      | One-shot game-over sting |
//! | [`play_ui_sfx`]          | `Update`              | [`Interaction`] change | Mouse button hover / click sounds |
//! | [`play_keyboard_ui_sfx`] | `Update`              | W/S/Arrow/Enter keys   | Keyboard button nav / confirm sounds |
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
use suika_game_core::events::{FruitMergeEvent, ScoreEarnedEvent};
use suika_game_core::fruit::FruitType;
use suika_game_ui::components::{KeyboardFocusIndex, MenuButton};

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
// Combo SFX
// ---------------------------------------------------------------------------

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
///
/// Volume and pitch parameters are read from [`AudioConfig`] at call time and
/// support hot-reload via `assets/config/audio.ron`.
pub fn play_combo_sfx(
    mut score_events: MessageReader<ScoreEarnedEvent>,
    audio: Res<Audio>,
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

        audio
            .play(sfx_handles.combo.clone())
            .with_volume(cfg.sfx_combo_volume)
            .with_playback_rate(pitch);
    }
}

// ---------------------------------------------------------------------------
// Game-over SFX
// ---------------------------------------------------------------------------

/// Plays the game-over sound effect once when the game transitions to
/// [`AppState::GameOver`].
///
/// This system is scheduled on [`OnEnter(AppState::GameOver)`] so it fires
/// exactly once per game-over, regardless of frame rate.
///
/// Volume is read from [`AudioConfig`] at call time.
pub fn play_gameover_sfx(
    audio: Res<Audio>,
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

    audio
        .play(sfx_handles.gameover.clone())
        .with_volume(cfg.sfx_gameover_volume);

    info!("Game-over SFX playing");
}

// ---------------------------------------------------------------------------
// UI SFX
// ---------------------------------------------------------------------------

/// Plays sound effects in response to button hover and click interactions.
///
/// Queries every [`MenuButton`] entity whose [`Interaction`] component changed
/// this frame (Bevy change-detection) and plays the appropriate clip:
///
/// - [`Interaction::Hovered`] → `button_hover.wav` at a low volume so the
///   sound is noticeable but not intrusive.
/// - [`Interaction::Pressed`] → `button_click.wav` at a slightly higher
///   volume to confirm the press.
///
/// Volume values are read from [`AudioConfig`] at call time, so they can be
/// tuned via hot-reload in `assets/config/audio.ron`.
pub fn play_ui_sfx(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<MenuButton>)>,
    audio: Res<Audio>,
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

    for interaction in interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                audio
                    .play(sfx_handles.button_click.clone())
                    .with_volume(cfg.sfx_button_click_volume);
            }
            Interaction::Hovered => {
                audio
                    .play(sfx_handles.button_hover.clone())
                    .with_volume(cfg.sfx_button_hover_volume);
            }
            Interaction::None => {}
        }
    }
}

/// Plays sound effects in response to keyboard menu navigation.
///
/// This system is the keyboard counterpart to [`play_ui_sfx`].  Bevy's
/// [`Interaction`] component is only updated by pointer devices, so
/// W / S / Arrow keys and Enter must be handled independently.
///
/// - W / S / Up / Down → `button_hover.wav`, **only when the focused button
///   index actually changes**.  Pressing a navigation key at a list boundary
///   (where focus cannot move further) does not produce a sound.
/// - Enter → `button_click.wav` (confirms the currently focused button).
///
/// The system is a no-op when no [`MenuButton`] entities are present (e.g.
/// during gameplay), preventing stray sounds from key presses on the
/// playing field.
///
/// [`prev_focus`] is a system-local value that tracks the focus index from the
/// previous frame so that boundary presses can be detected without relying on
/// execution order relative to the navigation system.
#[allow(clippy::too_many_arguments)]
pub fn play_keyboard_ui_sfx(
    keyboard: Res<ButtonInput<KeyCode>>,
    button_query: Query<(), With<MenuButton>>,
    focus: Option<Res<KeyboardFocusIndex>>,
    mut prev_focus: Local<Option<usize>>,
    audio: Res<Audio>,
    sfx_handles: Option<Res<SfxHandles>>,
    audio_config_handle: Option<Res<AudioConfigHandle>>,
    audio_config_assets: Res<Assets<AudioConfig>>,
) {
    // No menu buttons on screen — reset tracking and bail.
    if button_query.is_empty() {
        *prev_focus = None;
        return;
    }

    let current = focus.as_ref().map(|r| r.0).unwrap_or(0);

    let Some(sfx_handles) = sfx_handles else {
        *prev_focus = Some(current);
        return;
    };

    let default_cfg = AudioConfig::default();
    let cfg = audio_config_handle
        .as_ref()
        .and_then(|h| audio_config_assets.get(&h.0))
        .unwrap_or(&default_cfg);

    // Hover sound: only when the focus index actually moved to a different
    // button.  Comparing against the locally cached previous value avoids
    // spurious sounds when a navigation key is pressed at a list boundary
    // (where the index stays the same).
    let old = prev_focus.replace(current);
    if old.is_some_and(|p| p != current) {
        audio
            .play(sfx_handles.button_hover.clone())
            .with_volume(cfg.sfx_button_hover_volume);
    }

    // Confirm key → click sound.
    if keyboard.just_pressed(KeyCode::Enter) {
        audio
            .play(sfx_handles.button_click.clone())
            .with_volume(cfg.sfx_button_click_volume);
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

    #[test]
    fn test_ui_sfx_volumes_are_audible_and_quiet() {
        // UI button sounds should be quieter than full (≤ 0 dB) but still
        // clearly audible (> -30 dB).  A value like -60 dB would be
        // effectively silent and would pass a <= 0.0 check alone.
        let cfg = AudioConfig::default();
        assert!(
            cfg.sfx_button_click_volume <= 0.0,
            "button click volume should be ≤ 0 dB (quiet)"
        );
        assert!(
            cfg.sfx_button_click_volume > -30.0,
            "button click volume should be > -30 dB (audible)"
        );
        assert!(
            cfg.sfx_button_hover_volume <= 0.0,
            "button hover volume should be ≤ 0 dB (quiet)"
        );
        assert!(
            cfg.sfx_button_hover_volume > -30.0,
            "button hover volume should be > -30 dB (audible)"
        );
    }
}
