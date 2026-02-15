//! # game-audio
//!
//! オーディオ管理：BGMと効果音の再生

use bevy::prelude::*;

// TODO: モジュールの追加
// pub mod bgm;
// pub mod sfx;

/// オーディオプラグイン
pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, _app: &mut App) {
        info!("GameAudioPlugin initialized");

        // TODO: オーディオシステムの登録
        // app
        //     .add_systems(Startup, load_audio_assets)
        //     .add_systems(Update, (switch_bgm, play_sfx));
    }
}
