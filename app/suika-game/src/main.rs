use bevy::prelude::*;
use bevy_kira_audio::AudioPlugin;
use bevy_rapier2d::prelude::*;

use suika_game_assets::GameAssetsPlugin;
use suika_game_audio::GameAudioPlugin;
use suika_game_core::GameCorePlugin;
use suika_game_ui::GameUIPlugin;

fn main() {
    App::new()
        // Bevyデフォルトプラグイン
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "スイカゲーム".to_string(),
                resolution: (800, 600).into(),
                ..default()
            }),
            ..default()
        }))
        // 外部プラグイン
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default()) // デバッグ用
        .add_plugins(AudioPlugin)
        // ゲームプラグイン（内部クレート）
        .add_plugins(GameAssetsPlugin) // 最初にアセットをロード
        .add_plugins(GameCorePlugin)
        .add_plugins(GameUIPlugin)
        .add_plugins(GameAudioPlugin)
        .run();
}
