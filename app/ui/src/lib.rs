//! # game-ui
//!
//! ユーザーインターフェース：画面実装、UIコンポーネント、スタイル

use bevy::prelude::*;
use suika_game_core::prelude::AppState;

pub mod components;
pub mod screens;
pub mod styles;

/// UIプラグイン
pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        info!("GameUIPlugin initialized");

        app.init_resource::<components::KeyboardFocusIndex>()
            .add_systems(OnEnter(AppState::Title), screens::title::setup_title_screen)
            .add_systems(
                Update,
                (
                    components::handle_button_interaction,
                    components::handle_keyboard_menu_navigation,
                ),
            );

        // TODO: UIシステムの登録
        // app
        //     .add_systems(OnEnter(AppState::Playing), setup_hud)
        //     .add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);
    }
}
