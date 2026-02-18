mod debug;

use bevy::prelude::*;
use bevy_kira_audio::AudioPlugin;
use bevy_rapier2d::prelude::*;

use debug::DebugPlugin;
use suika_game_assets::GameAssetsPlugin;
use suika_game_audio::GameAudioPlugin;
use suika_game_core::prelude::*;
use suika_game_ui::GameUIPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "スイカゲーム".to_string(),
                resolution: (800, 600).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(AudioPlugin)
        .add_plugins(GameAssetsPlugin)
        .add_plugins(GameConfigPlugin)
        .add_plugins(GameCorePlugin)
        .add_plugins(GameUIPlugin)
        .add_plugins(GameAudioPlugin)
        .add_plugins(DebugPlugin)
        .run();
}
