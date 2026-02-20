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
use bevy_kira_audio::{AudioApp, AudioPlugin as KiraAudioPlugin};
use suika_game_core::prelude::{AppState, SettingsResource};

pub mod bgm;
pub mod channels;
pub mod config;
pub mod handles;
pub mod sfx;

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
            // Typed audio channels — BGM bus and SFX bus.
            // User volume (from SettingsResource) is applied to these channels;
            // individual sound volumes remain the designer's RON-defined levels.
            .add_audio_channel::<channels::BgmChannel>()
            .add_audio_channel::<channels::SfxChannel>()
            // Audio config asset type + loader
            .init_asset::<config::AudioConfig>()
            .register_asset_loader(config::AudioConfigLoader)
            // Resources
            .init_resource::<bgm::CurrentBgm>()
            // Startup systems
            .add_systems(
                Startup,
                (handles::load_audio_assets, config::load_audio_config),
            )
            // Update systems
            .add_systems(
                Update,
                (
                    // Apply user volume to channels whenever settings change
                    // (also fires on the first frame after SettingsResource loads).
                    channels::apply_volume_settings.run_if(resource_changed::<SettingsResource>),
                    bgm::switch_bgm_on_state_change.run_if(state_changed::<AppState>),
                    config::hot_reload_audio_config,
                    sfx::play_merge_sfx,
                    sfx::play_combo_sfx,
                    sfx::play_ui_sfx,
                    sfx::play_keyboard_ui_sfx,
                ),
            )
            // One-shot systems triggered by state transitions
            .add_systems(OnEnter(AppState::GameOver), sfx::play_gameover_sfx);

        info!("GameAudioPlugin initialized (bevy_kira_audio ready)");
    }
}
