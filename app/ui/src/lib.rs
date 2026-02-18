//! # game-ui
//!
//! ユーザーインターフェース：画面実装、UIコンポーネント、スタイル

use bevy::prelude::*;
use suika_game_core::prelude::{AppState, GameOverSet};

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
            // Game-over screen — must run AFTER core saves the highscore so
            // that GameState::is_new_record and highscore are up-to-date.
            .add_systems(
                OnEnter(AppState::GameOver),
                screens::game_over::setup_game_over_screen.after(GameOverSet::SaveHighscore),
            )
            // Pause menu
            .add_systems(OnEnter(AppState::Paused), screens::pause::setup_pause_menu)
            // ESC toggles Playing ↔ Paused (runs every frame, ignores other states)
            .add_systems(Update, screens::pause::toggle_pause)
            // Button interaction (all states)
            .add_systems(
                Update,
                (
                    components::handle_button_interaction,
                    components::handle_keyboard_menu_navigation,
                ),
            );
    }
}
