//! # game-ui
//!
//! ユーザーインターフェース：画面実装、UIコンポーネント、スタイル

use bevy::prelude::*;
use suika_game_core::prelude::AppState;

pub mod camera;
pub mod components;
pub mod config;
pub mod screens;
pub mod styles;

/// UIプラグイン
pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        info!("GameUIPlugin initialized");

        // UI config: asset loading + hot-reload (mirrors GameConfigPlugin in core)
        app.add_plugins(config::UiConfigPlugin);

        // Background color comes from the UI style palette
        app.insert_resource(ClearColor(styles::BG_COLOR));

        app.add_systems(Startup, camera::setup_camera)
            .init_resource::<components::KeyboardFocusIndex>()
            // Title screen
            .add_systems(OnEnter(AppState::Title), screens::title::setup_title_screen)
            // HUD: spawn layout on enter Playing, run widget updates each frame
            .add_systems(OnEnter(AppState::Playing), screens::hud::setup_hud)
            .add_systems(
                Update,
                (
                    screens::hud::best_score::update_best_score,
                    screens::hud::score::update_score,
                    screens::hud::next::update_next,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            // Button interaction (all states)
            .add_systems(
                Update,
                (
                    components::handle_button_interaction,
                    components::handle_keyboard_menu_navigation,
                ),
            );

        // TODO: UIシステムの登録
        // app
        //     .add_systems(OnEnter(AppState::GameOver), setup_game_over_screen);
    }
}
