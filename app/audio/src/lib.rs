//! Audio management for Suika Game.
//!
//! Wraps [`bevy_kira_audio`] and registers all BGM / SFX systems.
//! The [`GameAudioPlugin`] is the single entry-point: add it to the Bevy
//! [`App`] in `main.rs` — it internally adds [`bevy_kira_audio::AudioPlugin`]
//! so no other audio setup is needed at the application level.
//!
//! # Module layout (future tasks)
//!
//! | Module | Responsibility |
//! |--------|---------------|
//! | `handles` | Load & store `Handle<AudioSource>` for every asset |
//! | `bgm`     | BGM playback, state-driven track switching |
//! | `sfx`     | SFX playback (merge, combo, UI, game-over) |

use bevy::prelude::*;
use bevy_kira_audio::AudioPlugin as KiraAudioPlugin;
use suika_game_core::prelude::AppState;

pub mod bgm;
pub mod handles;

// Future modules (uncomment as tasks are completed):
// pub mod sfx;

/// Integrates [`bevy_kira_audio`] into the game and registers all audio systems.
///
/// Add this plugin to your [`App`] once; it owns the kira audio backend setup.
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use suika_game_audio::GameAudioPlugin;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(GameAudioPlugin)
///     .run();
/// ```
pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        // Add the kira audio backend.  All other audio plugins/systems rely on
        // this being registered first.
        app.add_plugins(KiraAudioPlugin)
            .init_resource::<bgm::CurrentBgm>()
            .add_systems(Startup, handles::load_audio_assets)
            .add_systems(
                Update,
                bgm::switch_bgm_on_state_change.run_if(state_changed::<AppState>),
            );

        info!("GameAudioPlugin initialized (bevy_kira_audio ready)");

        // TODO: register systems when subsequent audio tasks are implemented:
        // app
        //     .add_systems(Update, (sfx::play_merge_sfx, sfx::play_combo_sfx, sfx::play_ui_sfx))
        //     .add_systems(OnEnter(AppState::GameOver), sfx::play_gameover_sfx);
    }
}
