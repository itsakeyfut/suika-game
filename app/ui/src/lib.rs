//! # game-ui
//!
//! ユーザーインターフェース：画面実装、UIコンポーネント、スタイル

use bevy::prelude::*;

pub mod styles;

// TODO: モジュールの追加
// pub mod components;
// pub mod screens;

/// UIプラグイン
pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, _app: &mut App) {
        info!("GameUIPlugin initialized");

        // TODO: UIシステムの登録
        // app
        //     .add_systems(OnEnter(AppState::Title), setup_title_screen)
        //     .add_systems(OnEnter(AppState::Playing), setup_hud)
        //     .add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);
    }
}
