//! Typed audio channels for BGM and SFX.
//!
//! Using two separate [`AudioChannel`] buses means the user's volume slider
//! controls **all** sounds on that bus at once, independently of the
//! per-sound design volumes set in `assets/config/audio.ron`.
//!
//! # Architecture
//!
//! ```text
//! AudioChannel<BgmChannel>  ← set_volume(user_bgm_dB)
//!   └─ bgm_handles.title / game / gameover   ← with_volume(design_dB)
//!
//! AudioChannel<SfxChannel>  ← set_volume(user_sfx_dB)
//!   └─ sfx_handles.*                         ← with_volume(design_dB)
//! ```
//!
//! When the user sets volume to 0 the channel is driven to −100 dB, which
//! bevy_kira_audio / kira rounds to silence, guaranteeing no audio leaks
//! through regardless of the per-sound design levels.

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use suika_game_core::resources::settings::SettingsResource;

// ---------------------------------------------------------------------------
// Channel marker types
// ---------------------------------------------------------------------------

/// Marker resource identifying the BGM audio channel.
///
/// Register with `app.add_audio_channel::<BgmChannel>()` and inject as
/// `Res<AudioChannel<BgmChannel>>` in systems.
#[derive(Resource)]
pub struct BgmChannel;

/// Marker resource identifying the SFX audio channel.
///
/// Register with `app.add_audio_channel::<SfxChannel>()` and inject as
/// `Res<AudioChannel<SfxChannel>>` in systems.
#[derive(Resource)]
pub struct SfxChannel;

// ---------------------------------------------------------------------------
// Volume helper
// ---------------------------------------------------------------------------

/// Converts a 0–10 volume-slider value to a dB level for [`AudioChannel::set_volume`].
///
/// | Slider | dB      | Perceived |
/// |--------|---------|-----------|
/// | 10     |   0 dB  | full      |
/// |  8     |  −8 dB  | default   |
/// |  5     | −20 dB  | half      |
/// |  1     | −36 dB  | very soft |
/// |  0     | −100 dB | silence   |
///
/// Formula: `(vol / 10 − 1) × 40`.  At vol = 0 a hard floor of −100 dB is
/// returned so kira mutes the channel completely.
pub fn volume_to_db(vol: u8) -> f32 {
    if vol == 0 {
        return -100.0;
    }
    (vol as f32 / 10.0 - 1.0) * 40.0
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Applies the user's volume settings to the BGM and SFX channels.
///
/// Schedule this with `.run_if(resource_changed::<SettingsResource>())` so it
/// runs on the first frame (when the resource is first inserted / loaded from
/// disk) **and** whenever the user adjusts a slider in the settings screen.
pub fn apply_volume_settings(
    settings: Res<SettingsResource>,
    bgm_channel: Res<AudioChannel<BgmChannel>>,
    sfx_channel: Res<AudioChannel<SfxChannel>>,
) {
    bgm_channel.set_volume(volume_to_db(settings.bgm_volume));
    sfx_channel.set_volume(volume_to_db(settings.sfx_volume));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_to_db_full() {
        assert!((volume_to_db(10) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_volume_to_db_zero_is_silence() {
        assert!(
            volume_to_db(0) <= -50.0,
            "vol=0 must produce silence-level dB"
        );
    }

    #[test]
    fn test_volume_to_db_default() {
        // Default setting (vol=8) should give approximately −8 dB.
        let db = volume_to_db(8);
        assert!((db - (-8.0)).abs() < 1e-4, "got {db}");
    }

    #[test]
    fn test_volume_to_db_monotone() {
        let mut prev = f32::NEG_INFINITY;
        for v in 0u8..=10 {
            let db = volume_to_db(v);
            assert!(db > prev, "dB must increase with vol (failed at vol={v})");
            prev = db;
        }
    }

    #[test]
    fn test_volume_to_db_mid() {
        // vol=5 → (0.5 − 1.0) × 40 = −20 dB
        let db = volume_to_db(5);
        assert!((db - (-20.0)).abs() < 1e-4, "got {db}");
    }
}
