//! # game-core
//!
//! コアゲームロジック：フルーツシステム、物理、状態管理、エフェクト

use bevy::prelude::*;

// Modules
pub mod constants;
pub mod fruit;
pub mod resources;

// TODO: 追加モジュール
// pub mod physics;
// pub mod components;
// pub mod states;
// pub mod systems;

/// ゲームコアプラグイン
pub struct GameCorePlugin;

impl Plugin for GameCorePlugin {
    fn build(&self, _app: &mut App) {
        info!("GameCorePlugin initialized");

        // TODO: システムとリソースの登録
        // app
        //     .init_state::<AppState>()
        //     .init_resource::<GameState>()
        //     .add_systems(Startup, setup_systems)
        //     .add_systems(Update, game_systems);
    }
}
