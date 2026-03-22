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
    pub use crate::resources::settings::{Language, SettingsResource};
    pub use crate::resources::{
        CircleTexture, ComboTimer, FruitSprites, GameOverTimer, GameState, NextFruitType,
    };
    pub use crate::systems::input::{InputMode, LastCursorPosition, SpawnPosition};

    // States
    pub use crate::states::AppState;

    // Constants (re-export module for namespaced access)
    pub use crate::constants;

    // Persistence
    pub use crate::persistence::{
        HighscoreData, load_highscore, load_settings, save_highscore, save_settings,
        update_highscore,
    };

    // Systems
    pub use crate::systems;

    // Config
    pub use crate::config::{
        BounceConfig, BounceConfigHandle, BounceParams, DropletColorMode, DropletConfig,
        DropletConfigHandle, DropletParams, FlashConfig, FlashConfigHandle, FlashParams,
        FruitConfigEntry, FruitsConfig, FruitsConfigHandle, FruitsParams, GameConfigPlugin,
        GameRulesConfig, GameRulesConfigHandle, GameRulesParams, PhysicsConfig,
        PhysicsConfigHandle, PhysicsParams, RonColor, ShakeConfig, ShakeConfigHandle, ShakeParams,
        WatermelonConfig, WatermelonConfigHandle, WatermelonParams,
    };

    // Events
    pub use crate::events::{FruitMergeEvent, ScoreEarnedEvent};

    // Collision
    pub use crate::systems::collision::ProcessedCollisions;

    // Score
    pub use crate::systems::score::combo_multiplier;

    // System sets
    pub use crate::systems::game_over::GameOverSet;

    // Effects
    pub use crate::systems::effects::MergeAnimation;
    pub use crate::systems::effects::bounce::SquashStretchAnimation;
    pub use crate::systems::effects::droplet::WaterDroplet;
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
            .init_resource::<resources::SettingsResource>();

        // Register CircleTexture immediately (default = invalid handle) so any
        // Startup system can safely declare Res<CircleTexture> without ordering
        // constraints.  setup_circle_texture then fills in the real texture.
        app.init_resource::<resources::CircleTexture>();

        // FruitSprites is populated at Startup by the assets crate's load_fruit_sprites.
        // Initialise the empty resource here so core systems can always use
        // Option<Res<FruitSprites>> or Res<FruitSprites> safely.
        app.init_resource::<resources::FruitSprites>();

        // Load persisted data into resources at startup
        app.add_systems(
            Startup,
            (
                persistence::load_highscore_startup,
                persistence::load_settings_startup,
            ),
        );

        // Register events
        app.add_message::<events::FruitMergeEvent>();
        app.add_message::<events::ScoreEarnedEvent>();

        // Spawn the physics container walls once all configs are loaded
        app.add_systems(
            OnExit(states::AppState::Loading),
            systems::container::setup_container,
        );

        // Sub-plugins (each owns its system registrations)
        app.add_plugins((
            systems::spawn::SpawnPlugin,
            systems::collision::CollisionPlugin,
            systems::merge::MergePlugin,
            systems::score::ScorePlugin,
            systems::effects::EffectsPlugin,
            systems::input::InputPlugin,
            systems::preview::PreviewPlugin,
            systems::boundary::BoundaryPlugin,
            systems::game_over::GameOverPlugin,
            systems::pause::PausePlugin,
        ));
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
            .add_plugins(bevy::state::app::StatesPlugin)
            // AssetPlugin required by setup_circle_texture (ResMut<Assets<Image>>)
            .add_plugins(bevy::asset::AssetPlugin::default());
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
