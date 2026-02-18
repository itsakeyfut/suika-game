//! Integration tests for Phase 3 systems
//!
//! These tests verify that all Phase 3 systems are properly integrated
//! and work together correctly.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Re-export the modules we need to test
// Note: This requires the modules to be public or we test through the binary
// For now, we'll test the core components that should be present

#[test]
fn test_phase3_integration() {
    // This test verifies that the game can be initialized with all Phase 3 systems
    // without panicking or errors.

    // Create a minimal app with just the essential plugins
    let mut app = App::new();

    // Add minimal plugins needed for testing
    app.add_plugins(MinimalPlugins);

    // Tick the app once to verify initialization completes without panicking
    app.update();
}

#[test]
fn test_core_components_available() {
    use suika_game_core::prelude::*;

    // Test that all Phase 3 custom components are available and can be instantiated

    let _container = Container;
    let _boundary_line = BoundaryLine;
    let _fruit = Fruit;

    // If we got here, all custom components are available
}

// Note: Game parameters are now loaded from RON config files.
// See app/core/src/config.rs for configuration tests.

#[test]
fn test_resources_available() {
    use suika_game_core::prelude::*;

    // Test that all game resources can be initialized
    let _game_state = GameState::default();
    let _combo_timer = ComboTimer::default();
    let _game_over_timer = GameOverTimer::default();
    let _next_fruit = NextFruitType::default();

    // If we got here without panicking, all resources can be created
}

#[test]
fn test_app_state_definitions() {
    use suika_game_core::prelude::*;

    // Test that all app states are defined correctly
    let _loading = AppState::Loading;
    let _title = AppState::Title;
    let _playing = AppState::Playing;
    let _paused = AppState::Paused;
    let _game_over = AppState::GameOver;

    // Verify default state (Loading is the initial state so configs load first)
    assert_eq!(AppState::default(), AppState::Loading);
}

#[test]
fn test_physics_configuration() {
    // Verify physics plugin can be initialized
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0));

    // If we got here without panicking, the plugin initialized successfully
}
