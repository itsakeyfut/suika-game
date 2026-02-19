//! # Suika Game Core
//!
//! Core game logic for the Suika Game (Watermelon Game) clone.
//!
//! This crate provides the fundamental game systems including:
//! - Fruit evolution system (11 fruit types)
//! - Game state management (Title, Playing, Paused, GameOver)
//! - Resource management (score, combo, game over timer)
//! - Entity components (Fruit, Container, BoundaryLine, etc.)
//! - Game constants (physics, scoring, timing)
//! - Highscore persistence (JSON-based save/load)
//!
//! ## Usage
//!
//! Add the `GameCorePlugin` to your Bevy app:
//!
//! ```no_run
//! use bevy::prelude::*;
//! use suika_game_core::GameCorePlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(GameCorePlugin)
//!         .run();
//! }
//! ```
//!
//! ## Module Organization
//!
//! - [`components`]: ECS components for game entities
//! - [`constants`]: Game configuration constants
//! - [`events`]: Custom game events for event-driven architecture
//! - [`fruit`]: Fruit type definitions and parameters
//! - [`persistence`]: Highscore save/load functionality
//! - [`resources`]: Bevy resources for game state
//! - [`states`]: Application state definitions

use bevy::prelude::*;

// Module declarations
pub mod components;
pub mod config;
pub mod constants;
pub mod events;
pub mod fruit;
pub mod persistence;
pub mod resources;
pub mod states;
pub mod systems;

// Re-export commonly used types for convenience
pub mod prelude {
    //! Common imports for working with the core game systems.
    //!
    //! Use `use suika_game_core::prelude::*;` to import the most commonly
    //! used types and traits.

    // Components
    pub use crate::components::{
        BottomWall, BoundaryLine, Container, Dropping, Fruit, FruitSpawnState, LeftWall,
        MergeCandidate, NextFruitPreview, RightWall,
    };

    // Fruit system
    pub use crate::fruit::{FruitParams, FruitType};

    // Resources
    pub use crate::resources::{ComboTimer, GameOverTimer, GameState, NextFruitType};
    pub use crate::systems::input::{InputMode, LastCursorPosition, SpawnPosition};

    // States
    pub use crate::states::AppState;

    // Constants (re-export module for namespaced access)
    pub use crate::constants;

    // Persistence
    pub use crate::persistence::{HighscoreData, load_highscore, save_highscore, update_highscore};

    // Systems
    pub use crate::systems;

    // Config
    pub use crate::config::{
        BounceConfig, BounceConfigHandle, DropletConfig, DropletConfigHandle, FlashConfig,
        FlashConfigHandle, FruitConfigEntry, FruitsConfig, FruitsConfigHandle, GameConfigPlugin,
        GameRulesConfig, GameRulesConfigHandle, PhysicsConfig, PhysicsConfigHandle, RonColor,
        ShakeConfig, ShakeConfigHandle, WatermelonConfig, WatermelonConfigHandle,
    };

    // Events
    pub use crate::events::FruitMergeEvent;

    // Collision
    pub use crate::systems::collision::ProcessedCollisions;

    // Score
    pub use crate::systems::score::combo_multiplier;

    // System sets
    pub use crate::systems::game_over::GameOverSet;

    // Effects
    pub use crate::systems::effects::MergeAnimation;
    pub use crate::systems::effects::bounce::SquashStretchAnimation;
    pub use crate::systems::effects::droplet::{DropletColorMode, WaterDroplet};
    pub use crate::systems::effects::flash::{LocalFlashAnimation, ScreenFlashAnimation};
    pub use crate::systems::effects::shake::CameraShake;
    pub use crate::systems::effects::watermelon::{
        WatermelonBurstParticle, WatermelonExplosionRing,
    };

    // Plugin
    pub use crate::GameCorePlugin;
}

/// Core game plugin
///
/// This plugin initializes the core game systems and registers
/// all necessary resources and states with the Bevy app.
///
/// # What it does
///
/// Currently, this plugin serves as a placeholder for future system registration.
/// In upcoming phases, it will:
/// - Initialize application state (`AppState`)
/// - Register game resources (`GameState`, `ComboTimer`, etc.)
/// - Set up physics systems
/// - Configure collision detection
/// - Register game logic systems
///
/// # Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use suika_game_core::GameCorePlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(GameCorePlugin)
///         .run();
/// }
/// ```
pub struct GameCorePlugin;

impl Plugin for GameCorePlugin {
    fn build(&self, app: &mut App) {
        info!("GameCorePlugin initialized");

        // Initialize application state
        app.init_state::<states::AppState>();

        // Initialize game resources
        app.init_resource::<resources::GameState>()
            .init_resource::<resources::ComboTimer>()
            .init_resource::<resources::GameOverTimer>()
            .init_resource::<resources::NextFruitType>()
            .init_resource::<systems::input::SpawnPosition>()
            .init_resource::<systems::input::InputMode>()
            .init_resource::<systems::input::LastCursorPosition>();

        // Load persisted highscore into GameState at startup
        app.add_systems(Startup, persistence::load_highscore_startup);

        // Register events
        app.add_message::<events::FruitMergeEvent>();

        // Initialize collision detection resources
        app.init_resource::<systems::collision::ProcessedCollisions>();

        // Collision detection, merge, and score systems (Phase 5)
        app.add_systems(
            Update,
            (
                systems::collision::detect_fruit_collision,
                systems::merge::handle_fruit_merge
                    .after(systems::collision::detect_fruit_collision),
                systems::score::update_score_on_merge
                    .after(systems::collision::detect_fruit_collision),
                systems::collision::clear_processed_collisions
                    .after(systems::merge::handle_fruit_merge)
                    .after(systems::score::update_score_on_merge),
            ),
        );

        // Combo timer tick (must run after merge scoring to avoid premature combo resets)
        app.add_systems(
            Update,
            systems::score::tick_combo_timer.after(systems::score::update_score_on_merge),
        );

        // Visual effects — all gated on Playing so they freeze during Paused.
        app.add_systems(
            Update,
            (
                // Merge scale animation
                systems::effects::animate_merge_scale.after(systems::merge::handle_fruit_merge),
                // Squash-and-stretch bounce
                systems::effects::bounce::animate_squash_stretch
                    .after(systems::merge::handle_fruit_merge),
                // Water droplet particles
                systems::effects::droplet::spawn_merge_droplets
                    .after(systems::merge::handle_fruit_merge),
                systems::effects::droplet::handle_fruit_landing,
                systems::effects::droplet::update_water_droplets,
                // Flash effects
                systems::effects::flash::spawn_merge_flash
                    .after(systems::merge::handle_fruit_merge),
                systems::effects::flash::animate_local_flash,
                systems::effects::flash::animate_screen_flash,
                // Camera shake — trauma accumulates on merge (Playing only)
                systems::effects::shake::add_camera_shake.after(systems::merge::handle_fruit_merge),
                // Watermelon special effects
                systems::effects::watermelon::spawn_watermelon_effects
                    .after(systems::merge::handle_fruit_merge),
                systems::effects::watermelon::animate_watermelon_explosion,
                systems::effects::watermelon::update_watermelon_burst_particles,
            )
                .run_if(in_state(states::AppState::Playing)),
        );

        // Camera shake apply runs every frame (not gated on Playing) so that
        // trauma decays and the camera snaps back even while Paused or in GameOver.
        app.add_systems(Update, systems::effects::shake::apply_camera_shake);

        // Elapsed-time tick (Playing state only)
        app.add_systems(
            Update,
            systems::game_over::tick_elapsed_time.run_if(in_state(states::AppState::Playing)),
        );

        // Phase 6: boundary overflow detection and game-over transition
        // All three run only during active gameplay.
        app.add_systems(
            Update,
            (
                systems::boundary::check_boundary_overflow,
                systems::boundary::trigger_game_over
                    .after(systems::boundary::check_boundary_overflow),
                systems::boundary::animate_boundary_warning
                    .after(systems::boundary::check_boundary_overflow),
            )
                .run_if(in_state(states::AppState::Playing)),
        );

        // Phase 6: highscore persistence on game over.
        // Registered inside GameOverSet::SaveHighscore so that other crates
        // (e.g. UI) can order their OnEnter(GameOver) systems after this set
        // and safely read GameState::is_new_record / highscore.
        app.add_systems(
            OnEnter(states::AppState::GameOver),
            systems::game_over::save_highscore_on_game_over
                .in_set(systems::game_over::GameOverSet::SaveHighscore),
        );

        // Reset game state in two places to cover all "new game" entry paths
        // while NOT resetting on Paused → Playing (resume):
        //   • OnExit(GameOver)  — GameOver → Playing  /  GameOver → Title → Playing
        //   • OnExit(Title)     — Title → Playing  /  (Paused → Title → Playing)
        // Paused → Playing never passes through either of these, so the current
        // session is preserved on resume.
        app.add_systems(
            OnExit(states::AppState::GameOver),
            systems::game_over::reset_game_state,
        );
        app.add_systems(
            OnExit(states::AppState::Title),
            systems::game_over::reset_game_state,
        );

        // Pause / resume: freeze the physics pipeline while paused.
        // All gameplay input and scoring systems already gate on Playing, so
        // this is the only change needed to fully suspend the simulation.
        app.add_systems(
            OnEnter(states::AppState::Paused),
            systems::pause::pause_physics,
        );
        app.add_systems(
            OnExit(states::AppState::Paused),
            systems::pause::resume_physics,
        );

        // Spawn the physics container walls once all configs are loaded
        app.add_systems(
            OnExit(states::AppState::Loading),
            systems::container::setup_container,
        );

        // Gameplay input systems — only active while Playing
        app.add_systems(
            Update,
            (
                systems::input::update_spawn_position,
                systems::input::handle_fruit_drop_input
                    .after(systems::input::update_spawn_position),
                systems::input::detect_fruit_landing,
                systems::input::spawn_held_fruit.after(systems::input::detect_fruit_landing),
            )
                .run_if(in_state(states::AppState::Playing)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_builds() {
        let mut app = App::new();
        // MinimalPlugins + StatesPlugin are required for GameCorePlugin
        // (StatesPlugin is needed for init_state, included in DefaultPlugins but not MinimalPlugins)
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin);
        app.add_plugins(GameCorePlugin);
        // Plugin should build without panicking
    }

    #[test]
    fn test_prelude_imports() {
        // Verify that prelude imports work
        use crate::prelude::*;

        // Test that types are accessible
        let _state = AppState::default();
        let _fruit_type = FruitType::Cherry;
        let _game_state = GameState::default();
    }
}
