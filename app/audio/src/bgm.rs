//! BGM (background-music) management system.
//!
//! Listens for [`AppState`] transitions and switches BGM tracks accordingly,
//! with fade-in on the incoming track and fade-out on the outgoing one.
//!
//! # Track mapping
//!
//! | [`AppState`]          | [`BgmTrack`] | Loop | Fade-in |
//! |-----------------------|-------------|------|---------|
//! | [`Loading`]           | `None`      | —    | —       |
//! | [`Title`]             | `Title`     | ✓    | 1.0 s   |
//! | [`Playing`] / [`Paused`] | `Game`  | ✓    | 1.5 s   |
//! | [`GameOver`]          | `GameOver`  | ✗    | —       |
//!
//! [`Loading`]: AppState::Loading
//! [`Title`]: AppState::Title
//! [`Playing`]: AppState::Playing
//! [`Paused`]: AppState::Paused
//! [`GameOver`]: AppState::GameOver

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;
use suika_game_core::prelude::AppState;

use crate::handles::BgmHandles;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Which BGM track is currently playing (or desired).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BgmTrack {
    /// No BGM — used during the Loading screen.
    #[default]
    None,
    /// Title-screen track (loops).
    Title,
    /// In-game track, shared between [`Playing`](AppState::Playing) and
    /// [`Paused`](AppState::Paused) so the music continues during pause.
    Game,
    /// Game-over track (one-shot, no loop).
    GameOver,
}

/// Resource that tracks which BGM track is currently playing.
///
/// Updated by [`switch_bgm_on_state_change`] whenever the track switches.
/// Initialised to [`BgmTrack::None`] at startup.
#[derive(Resource, Default, Debug)]
pub struct CurrentBgm {
    /// The track that is currently playing (or was last requested).
    pub track: BgmTrack,
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Returns the [`BgmTrack`] that should play for the given [`AppState`].
///
/// This is a pure function with no side effects — useful for unit testing.
pub fn desired_track(state: &AppState) -> BgmTrack {
    match state {
        AppState::Loading => BgmTrack::None,
        AppState::Title => BgmTrack::Title,
        // Paused keeps the game track so the music doesn't cut out on pause.
        AppState::Playing | AppState::Paused => BgmTrack::Game,
        AppState::GameOver => BgmTrack::GameOver,
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Switches BGM whenever [`AppState`] transitions to a new track.
///
/// Register this with `.run_if(state_changed::<AppState>())` to avoid polling
/// every frame:
///
/// ```rust,ignore
/// app.add_systems(
///     Update,
///     bgm::switch_bgm_on_state_change.run_if(state_changed::<AppState>()),
/// );
/// ```
///
/// # Behaviour
/// - If the desired track is the same as the current one (e.g. `Playing →
///   Paused` both map to `Game`) the function returns early.
/// - The outgoing track fades out over **0.5 s**.
/// - Incoming `Title` / `Game` tracks fade in over 1.0 s / 1.5 s respectively.
/// - `GameOver` plays immediately (no fade-in) and does not loop.
/// - If [`BgmHandles`] has not yet been inserted (asset loading still in
///   progress) the system returns early rather than panicking.
pub fn switch_bgm_on_state_change(
    current_state: Res<State<AppState>>,
    mut current_bgm: ResMut<CurrentBgm>,
    audio: Res<Audio>,
    bgm_handles: Option<Res<BgmHandles>>,
) {
    let Some(bgm_handles) = bgm_handles else {
        return;
    };

    let desired = desired_track(current_state.get());

    // Nothing to do if the track hasn't changed (e.g. Playing → Paused).
    if current_bgm.track == desired {
        return;
    }

    // Fade out the currently-playing track.
    audio
        .stop()
        .fade_out(AudioTween::linear(Duration::from_secs_f32(0.5)));

    // Start the new track.
    match desired {
        BgmTrack::None => {
            // Already stopped above; nothing more to do.
        }
        BgmTrack::Title => {
            audio
                .play(bgm_handles.title.clone())
                .looped()
                .with_volume(0.6)
                .fade_in(AudioTween::linear(Duration::from_secs_f32(1.0)));
        }
        BgmTrack::Game => {
            audio
                .play(bgm_handles.game.clone())
                .looped()
                .with_volume(0.4)
                .fade_in(AudioTween::linear(Duration::from_secs_f32(1.5)));
        }
        BgmTrack::GameOver => {
            // One-shot: no loop, no fade-in.
            audio.play(bgm_handles.gameover.clone()).with_volume(0.5);
        }
    }

    let prev = current_bgm.track;
    current_bgm.track = desired;
    info!("BGM: {:?} → {:?}", prev, desired);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // BgmTrack
    // ------------------------------------------------------------------

    #[test]
    fn test_bgm_track_default_is_none() {
        assert_eq!(BgmTrack::default(), BgmTrack::None);
    }

    #[test]
    fn test_bgm_track_equality() {
        assert_eq!(BgmTrack::None, BgmTrack::None);
        assert_eq!(BgmTrack::Title, BgmTrack::Title);
        assert_eq!(BgmTrack::Game, BgmTrack::Game);
        assert_eq!(BgmTrack::GameOver, BgmTrack::GameOver);

        assert_ne!(BgmTrack::None, BgmTrack::Title);
        assert_ne!(BgmTrack::Title, BgmTrack::Game);
        assert_ne!(BgmTrack::Game, BgmTrack::GameOver);
    }

    // ------------------------------------------------------------------
    // CurrentBgm
    // ------------------------------------------------------------------

    #[test]
    fn test_current_bgm_default() {
        let bgm = CurrentBgm::default();
        assert_eq!(bgm.track, BgmTrack::None);
    }

    // ------------------------------------------------------------------
    // desired_track
    // ------------------------------------------------------------------

    #[test]
    fn test_desired_track_loading_is_none() {
        assert_eq!(desired_track(&AppState::Loading), BgmTrack::None);
    }

    #[test]
    fn test_desired_track_title_is_title() {
        assert_eq!(desired_track(&AppState::Title), BgmTrack::Title);
    }

    #[test]
    fn test_desired_track_playing_is_game() {
        assert_eq!(desired_track(&AppState::Playing), BgmTrack::Game);
    }

    #[test]
    fn test_desired_track_paused_is_game() {
        // Paused must share the Game track so BGM continues during pause.
        assert_eq!(desired_track(&AppState::Paused), BgmTrack::Game);
    }

    #[test]
    fn test_desired_track_gameover_is_gameover() {
        assert_eq!(desired_track(&AppState::GameOver), BgmTrack::GameOver);
    }

    #[test]
    fn test_playing_and_paused_share_same_track() {
        // Ensures the BGM doesn't restart when the player pauses and resumes.
        assert_eq!(
            desired_track(&AppState::Playing),
            desired_track(&AppState::Paused),
        );
    }

    #[test]
    fn test_all_states_have_a_mapping() {
        let states = [
            AppState::Loading,
            AppState::Title,
            AppState::Playing,
            AppState::Paused,
            AppState::GameOver,
        ];
        // Just confirm every state returns *some* (non-panicking) track.
        for state in &states {
            let _ = desired_track(state);
        }
    }
}
