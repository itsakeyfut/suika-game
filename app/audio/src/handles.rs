//! Audio asset handle resources.
//!
//! Defines [`BgmHandles`] and [`SfxHandles`] resources that hold pre-loaded
//! [`Handle<AudioSource>`] values for every audio file used in the game.
//! Loading happens once at [`Startup`] via [`load_audio_assets`]; all
//! subsequent audio systems read these handles rather than hitting the asset
//! server each frame.
//!
//! # Asset paths (relative to the `assets/` directory)
//!
//! ## BGM
//! | Field | Path |
//! |-------|------|
//! | `title`    | `sounds/bgm/title_bgm.ogg`    |
//! | `game`     | `sounds/bgm/game_bgm.ogg`     |
//! | `gameover` | `sounds/bgm/gameover_bgm.ogg` |
//!
//! ## SFX
//! | Field | Path |
//! |-------|------|
//! | `drop`          | `sounds/sfx/drop.wav`          |
//! | `merge_small`   | `sounds/sfx/merge_small.wav`   |
//! | `merge_medium`  | `sounds/sfx/merge_medium.wav`  |
//! | `merge_large`   | `sounds/sfx/merge_large.wav`   |
//! | `watermelon`    | `sounds/sfx/watermelon.wav`    |
//! | `combo`         | `sounds/sfx/combo.wav`         |
//! | `gameover`      | `sounds/sfx/gameover.wav`      |
//! | `button_click`  | `sounds/sfx/button_click.wav`  |
//! | `button_hover`  | `sounds/sfx/button_hover.wav`  |

use bevy::prelude::*;
use bevy_kira_audio::AudioSource;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Handles for all background-music tracks.
///
/// Inserted as a [`Resource`] by [`load_audio_assets`] at startup.
#[derive(Resource, Debug)]
pub struct BgmHandles {
    /// Title-screen BGM (`sounds/bgm/title_bgm.ogg`).
    pub title: Handle<AudioSource>,
    /// In-game BGM (`sounds/bgm/game_bgm.ogg`).
    pub game: Handle<AudioSource>,
    /// Game-over BGM (`sounds/bgm/gameover_bgm.ogg`).
    pub gameover: Handle<AudioSource>,
}

/// Handles for all sound-effect clips.
///
/// Inserted as a [`Resource`] by [`load_audio_assets`] at startup.
#[derive(Resource, Debug)]
pub struct SfxHandles {
    /// Fruit-drop sound (`sounds/sfx/drop.wav`).
    pub drop: Handle<AudioSource>,
    /// Merge sound for small fruits — Cherry, Strawberry, Grape
    /// (`sounds/sfx/merge_small.wav`).
    pub merge_small: Handle<AudioSource>,
    /// Merge sound for medium fruits — Dekopon through Pear
    /// (`sounds/sfx/merge_medium.wav`).
    pub merge_medium: Handle<AudioSource>,
    /// Merge sound for large fruits — Peach, Pineapple
    /// (`sounds/sfx/merge_large.wav`).
    pub merge_large: Handle<AudioSource>,
    /// Special fanfare when two Melons merge into a Watermelon
    /// (`sounds/sfx/watermelon.wav`).
    pub watermelon: Handle<AudioSource>,
    /// Combo-chain sound (`sounds/sfx/combo.wav`).
    pub combo: Handle<AudioSource>,
    /// Game-over sting (`sounds/sfx/gameover.wav`).
    pub gameover: Handle<AudioSource>,
    /// UI button-click sound (`sounds/sfx/button_click.wav`).
    pub button_click: Handle<AudioSource>,
    /// UI button-hover sound (`sounds/sfx/button_hover.wav`).
    pub button_hover: Handle<AudioSource>,
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Startup system — loads all audio assets and inserts [`BgmHandles`] and
/// [`SfxHandles`] as resources.
///
/// The `AssetServer` returns weak handles immediately; the actual audio data
/// is loaded asynchronously in the background.  Audio systems that use these
/// handles will silently skip playback if the asset has not yet finished
/// loading (this is the default bevy_kira_audio behaviour).
pub fn load_audio_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BgmHandles {
        title: asset_server.load("sounds/bgm/title_bgm.ogg"),
        game: asset_server.load("sounds/bgm/game_bgm.ogg"),
        gameover: asset_server.load("sounds/bgm/gameover_bgm.ogg"),
    });

    commands.insert_resource(SfxHandles {
        drop: asset_server.load("sounds/sfx/drop.wav"),
        merge_small: asset_server.load("sounds/sfx/merge_small.wav"),
        merge_medium: asset_server.load("sounds/sfx/merge_medium.wav"),
        merge_large: asset_server.load("sounds/sfx/merge_large.wav"),
        watermelon: asset_server.load("sounds/sfx/watermelon.wav"),
        combo: asset_server.load("sounds/sfx/combo.wav"),
        gameover: asset_server.load("sounds/sfx/gameover.wav"),
        button_click: asset_server.load("sounds/sfx/button_click.wav"),
        button_hover: asset_server.load("sounds/sfx/button_hover.wav"),
    });

    info!("Audio assets queued for loading (BGM: 3, SFX: 9)");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // AssetServer requires the asset infrastructure provided by AssetPlugin.
        app.add_plugins(bevy::asset::AssetPlugin::default());
        // Register the AudioSource asset type so that AssetServer::load can
        // allocate handles in tests (no audio hardware is initialized here).
        app.init_asset::<AudioSource>();
        app.add_systems(Startup, load_audio_assets);
        app
    }

    #[test]
    fn test_bgm_handles_resource_inserted() {
        let mut app = setup_app();
        app.update();

        assert!(
            app.world().get_resource::<BgmHandles>().is_some(),
            "BgmHandles resource must exist after load_audio_assets runs"
        );
    }

    #[test]
    fn test_sfx_handles_resource_inserted() {
        let mut app = setup_app();
        app.update();

        assert!(
            app.world().get_resource::<SfxHandles>().is_some(),
            "SfxHandles resource must exist after load_audio_assets runs"
        );
    }

    #[test]
    fn test_bgm_handles_are_valid() {
        let mut app = setup_app();
        app.update();

        let handles = app
            .world()
            .get_resource::<BgmHandles>()
            .expect("BgmHandles should be present");

        // Handles returned by AssetServer::load are always valid (non-default).
        assert_ne!(
            handles.title,
            Handle::default(),
            "title BGM handle must be non-default"
        );
        assert_ne!(
            handles.game,
            Handle::default(),
            "game BGM handle must be non-default"
        );
        assert_ne!(
            handles.gameover,
            Handle::default(),
            "gameover BGM handle must be non-default"
        );
    }

    #[test]
    fn test_sfx_handles_are_valid() {
        let mut app = setup_app();
        app.update();

        let handles = app
            .world()
            .get_resource::<SfxHandles>()
            .expect("SfxHandles should be present");

        // Each handle must be distinct (different asset paths → different ids).
        let all = [
            &handles.drop,
            &handles.merge_small,
            &handles.merge_medium,
            &handles.merge_large,
            &handles.watermelon,
            &handles.combo,
            &handles.gameover,
            &handles.button_click,
            &handles.button_hover,
        ];

        for handle in &all {
            assert_ne!(
                *handle,
                &Handle::default(),
                "SFX handle must be non-default"
            );
        }
    }

    #[test]
    fn test_sfx_handles_are_unique() {
        let mut app = setup_app();
        app.update();

        let handles = app
            .world()
            .get_resource::<SfxHandles>()
            .expect("SfxHandles should be present");

        // Every SFX path is different, so every handle id must be different.
        let ids = [
            handles.drop.id(),
            handles.merge_small.id(),
            handles.merge_medium.id(),
            handles.merge_large.id(),
            handles.watermelon.id(),
            handles.combo.id(),
            handles.gameover.id(),
            handles.button_click.id(),
            handles.button_hover.id(),
        ];

        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                assert_ne!(
                    ids[i], ids[j],
                    "SFX handles at index {i} and {j} must differ"
                );
            }
        }
    }

    #[test]
    fn test_bgm_handles_are_unique() {
        let mut app = setup_app();
        app.update();

        let handles = app
            .world()
            .get_resource::<BgmHandles>()
            .expect("BgmHandles should be present");

        assert_ne!(
            handles.title.id(),
            handles.game.id(),
            "title and game BGM handles must differ"
        );
        assert_ne!(
            handles.game.id(),
            handles.gameover.id(),
            "game and gameover BGM handles must differ"
        );
        assert_ne!(
            handles.title.id(),
            handles.gameover.id(),
            "title and gameover BGM handles must differ"
        );
    }
}
