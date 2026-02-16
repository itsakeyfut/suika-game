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
//! - [`fruit`]: Fruit type definitions and parameters
//! - [`persistence`]: Highscore save/load functionality
//! - [`resources`]: Bevy resources for game state
//! - [`states`]: Application state definitions

use bevy::prelude::*;

// Module declarations
pub mod components;
pub mod constants;
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
        BottomWall, BoundaryLine, Container, Dropping, Fruit, FruitSpawnState, MergeCandidate,
        NextFruitPreview,
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
    fn build(&self, _app: &mut App) {
        info!("GameCorePlugin initialized");

        // TODO: Phase 3+ - Register systems and resources
        // app
        //     .init_state::<AppState>()
        //     .init_resource::<GameState>()
        //     .init_resource::<ComboTimer>()
        //     .init_resource::<GameOverTimer>()
        //     .init_resource::<NextFruitType>()
        //     .add_systems(Startup, setup_systems)
        //     .add_systems(Update, game_systems);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_builds() {
        let mut app = App::new();
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
