mod camera;

use bevy::prelude::*;
use bevy_kira_audio::AudioPlugin;
use bevy_rapier2d::prelude::*;

use camera::setup_camera;
use suika_game_assets::GameAssetsPlugin;
use suika_game_audio::GameAudioPlugin;
use suika_game_core::prelude::*;
use suika_game_ui::GameUIPlugin;

fn main() {
    App::new()
        // Bevy default plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "スイカゲーム".to_string(),
                resolution: (800, 600).into(),
                ..default()
            }),
            ..default()
        }))
        // External plugins
        // Rapier physics with 100 pixels per meter scaling
        // Default gravity is -9.81 m/s², which equals -981 pixels/s²
        // This is approximately our target of -980 pixels/s² from constants::physics::GRAVITY
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default()) // Debug rendering
        .add_plugins(AudioPlugin)
        // Application state
        .init_state::<AppState>()
        // Game resources
        .init_resource::<GameState>()
        .init_resource::<ComboTimer>()
        .init_resource::<GameOverTimer>()
        .init_resource::<NextFruitType>()
        // Game plugins (internal crates)
        .add_plugins(GameAssetsPlugin) // Load assets first
        .add_plugins(GameCorePlugin)
        .add_plugins(GameUIPlugin)
        .add_plugins(GameAudioPlugin)
        // Startup systems
        .add_systems(Startup, (setup_camera, load_highscore_system))
        .run();
}

/// Loads the highscore from disk at startup
///
/// This system runs once during app initialization and loads the
/// saved highscore into the GameState resource.
fn load_highscore_system(mut game_state: ResMut<GameState>) {
    let highscore_data = load_highscore(std::path::Path::new(constants::storage::SAVE_DIR));

    game_state.highscore = highscore_data.highscore;

    info!("Highscore loaded from disk: {}", highscore_data.highscore);
}
