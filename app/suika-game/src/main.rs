mod camera;
mod container;
mod debug;

use bevy::prelude::*;
use bevy_kira_audio::AudioPlugin;
use bevy_rapier2d::prelude::*;

use camera::setup_camera;
use container::setup_container;
use debug::DebugPlugin;
use suika_game_assets::GameAssetsPlugin;
use suika_game_audio::GameAudioPlugin;
use suika_game_core::prelude::*;
use suika_game_core::systems::input::{
    detect_fruit_landing, handle_fruit_drop_input, spawn_held_fruit, update_spawn_position,
};
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
        // This is approximately our target of -980 pixels/s² (configurable in physics.ron)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(AudioPlugin)
        // Game plugins (internal crates)
        // Note: GameCorePlugin initializes AppState and all game resources
        .add_plugins(GameAssetsPlugin) // Load assets first
        .add_plugins(GameConfigPlugin) // Load game configuration
        .add_plugins(GameCorePlugin)
        .add_plugins(GameUIPlugin)
        .add_plugins(GameAudioPlugin)
        // Debug plugin (debug builds only)
        .add_plugins(DebugPlugin)
        // Startup systems
        .add_systems(Startup, (setup_camera, load_highscore_system))
        // setup_container runs on exit from Loading so physics.ron is guaranteed loaded
        .add_systems(OnExit(AppState::Loading), setup_container)
        // Gameplay systems — only run while actively playing
        .add_systems(
            Update,
            (
                update_spawn_position,
                handle_fruit_drop_input.after(update_spawn_position),
                detect_fruit_landing,
                spawn_held_fruit.after(detect_fruit_landing),
            )
                .run_if(in_state(AppState::Playing)),
        )
        .run();
}

/// Loads the highscore from disk at startup.
///
/// Runs once during app initialization and populates [`GameState::highscore`]
/// so the title screen can display the current best score immediately.
fn load_highscore_system(mut game_state: ResMut<GameState>) {
    let highscore_data = load_highscore(std::path::Path::new(constants::storage::SAVE_DIR));
    game_state.highscore = highscore_data.highscore;
    info!("Highscore loaded from disk: {}", highscore_data.highscore);
}
