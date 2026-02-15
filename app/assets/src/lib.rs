//! # game-assets
//!
//! アセット管理：スプライト、サウンド、フォントの読み込み

use bevy::prelude::*;

// TODO: モジュールの追加
// pub mod sprites;
// pub mod sounds;
// pub mod fonts;

/// アセット管理プラグイン
pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, _app: &mut App) {
        info!("GameAssetsPlugin initialized");

        // TODO: アセットローディングシステムの登録
        // app
        //     .add_systems(Startup, load_all_assets);
    }
}
