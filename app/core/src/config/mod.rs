//! Game configuration loaded from RON files
//!
//! This module handles loading and hot-reloading of game configuration
//! from RON (Rusty Object Notation) files in the assets directory.
//!
//! Supports hot-reloading: Edit config files while the game is running
//! and changes will be applied automatically.
//!
//! # Sub-modules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`gameplay`] | `FruitsConfig`, `PhysicsConfig`, `GameRulesConfig` + SystemParam bundles |
//! | [`effects`]  | `BounceConfig`, `DropletConfig`, `FlashConfig`, `ShakeConfig`, `WatermelonConfig` + SystemParam bundles |

pub mod effects;
pub mod gameplay;

pub use effects::*;
pub use gameplay::*;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::states::AppState;

// ---------------------------------------------------------------------------
// RON asset loader macro
// ---------------------------------------------------------------------------

/// Generates a RON-based `AssetLoader` implementation for a config type.
///
/// All game config assets use identical loading logic (read bytes → `ron::de::from_bytes`),
/// so this macro eliminates the repetition while keeping each loader a distinct type.
///
/// # Usage
/// ```ignore
/// ron_asset_loader!(MyConfigLoader, MyConfig);
/// ```
macro_rules! ron_asset_loader {
    ($loader:ident, $asset:ty) => {
        #[derive(Default)]
        struct $loader;

        impl AssetLoader for $loader {
            type Asset = $asset;
            type Settings = ();
            type Error = std::io::Error;

            async fn load(
                &self,
                reader: &mut dyn Reader,
                _settings: &Self::Settings,
                _load_context: &mut LoadContext<'_>,
            ) -> Result<Self::Asset, Self::Error> {
                let mut bytes = Vec::new();
                reader.read_to_end(&mut bytes).await?;
                ron::de::from_bytes(&bytes)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            }

            fn extensions(&self) -> &[&str] {
                &["ron"]
            }
        }
    };
}

// Loader types generated from the macro (all in mod.rs so the macro is local here)
ron_asset_loader!(FruitsConfigLoader, FruitsConfig);
ron_asset_loader!(PhysicsConfigLoader, PhysicsConfig);
ron_asset_loader!(GameRulesConfigLoader, GameRulesConfig);
ron_asset_loader!(BounceConfigLoader, BounceConfig);
ron_asset_loader!(DropletConfigLoader, DropletConfig);
ron_asset_loader!(FlashConfigLoader, FlashConfig);
ron_asset_loader!(ShakeConfigLoader, ShakeConfig);
ron_asset_loader!(WatermelonConfigLoader, WatermelonConfig);

// ---------------------------------------------------------------------------
// AllConfigs — private SystemParam for wait_for_configs
// ---------------------------------------------------------------------------

/// Bundles all config handle/asset pairs into a single `SystemParam` to stay
/// within Bevy's 16-parameter system limit as more configs are added.
#[derive(SystemParam)]
struct AllConfigs<'w> {
    physics_handle: Res<'w, PhysicsConfigHandle>,
    physics_assets: Res<'w, Assets<PhysicsConfig>>,
    fruits_handle: Res<'w, FruitsConfigHandle>,
    fruits_assets: Res<'w, Assets<FruitsConfig>>,
    game_rules_handle: Res<'w, GameRulesConfigHandle>,
    game_rules_assets: Res<'w, Assets<GameRulesConfig>>,
    bounce_handle: Res<'w, BounceConfigHandle>,
    bounce_assets: Res<'w, Assets<BounceConfig>>,
    droplet_handle: Res<'w, DropletConfigHandle>,
    droplet_assets: Res<'w, Assets<DropletConfig>>,
    flash_handle: Res<'w, FlashConfigHandle>,
    flash_assets: Res<'w, Assets<FlashConfig>>,
    shake_handle: Res<'w, ShakeConfigHandle>,
    shake_assets: Res<'w, Assets<ShakeConfig>>,
    watermelon_handle: Res<'w, WatermelonConfigHandle>,
    watermelon_assets: Res<'w, Assets<WatermelonConfig>>,
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin for game configuration management
pub struct GameConfigPlugin;

impl Plugin for GameConfigPlugin {
    fn build(&self, app: &mut App) {
        info!("🔧 Initializing GameConfigPlugin...");

        // Initialize asset types and register loaders
        app.init_asset::<FruitsConfig>()
            .register_asset_loader(FruitsConfigLoader)
            .init_asset::<PhysicsConfig>()
            .register_asset_loader(PhysicsConfigLoader)
            .init_asset::<GameRulesConfig>()
            .register_asset_loader(GameRulesConfigLoader)
            .init_asset::<BounceConfig>()
            .register_asset_loader(BounceConfigLoader)
            .init_asset::<DropletConfig>()
            .register_asset_loader(DropletConfigLoader)
            .init_asset::<FlashConfig>()
            .register_asset_loader(FlashConfigLoader)
            .init_asset::<ShakeConfig>()
            .register_asset_loader(ShakeConfigLoader)
            .init_asset::<WatermelonConfig>()
            .register_asset_loader(WatermelonConfigLoader);

        // Load all configs and insert handles immediately
        let asset_server = app.world_mut().resource::<AssetServer>();

        let fruits_handle: Handle<FruitsConfig> = asset_server.load("config/fruits.ron");
        let physics_handle: Handle<PhysicsConfig> = asset_server.load("config/physics.ron");
        let game_rules_handle: Handle<GameRulesConfig> = asset_server.load("config/game_rules.ron");
        let bounce_handle: Handle<BounceConfig> = asset_server.load("config/effects/bounce.ron");
        let droplet_handle: Handle<DropletConfig> = asset_server.load("config/effects/droplet.ron");
        let flash_handle: Handle<FlashConfig> = asset_server.load("config/effects/flash.ron");
        let shake_handle: Handle<ShakeConfig> = asset_server.load("config/effects/shake.ron");
        let watermelon_handle: Handle<WatermelonConfig> =
            asset_server.load("config/effects/watermelon.ron");

        app.insert_resource(FruitsConfigHandle(fruits_handle))
            .insert_resource(PhysicsConfigHandle(physics_handle))
            .insert_resource(GameRulesConfigHandle(game_rules_handle))
            .insert_resource(BounceConfigHandle(bounce_handle))
            .insert_resource(DropletConfigHandle(droplet_handle))
            .insert_resource(FlashConfigHandle(flash_handle))
            .insert_resource(ShakeConfigHandle(shake_handle))
            .insert_resource(WatermelonConfigHandle(watermelon_handle));

        // Add hot-reload systems (run in all states so live-edit always works)
        app.add_systems(
            Update,
            (
                hot_reload_fruits_config,
                hot_reload_physics_config,
                hot_reload_game_rules_config,
                hot_reload_bounce_config,
                hot_reload_droplet_config,
                hot_reload_flash_config,
                hot_reload_shake_config,
                hot_reload_watermelon_config,
            ),
        );

        // Transition Loading → Title once all required configs are ready
        app.add_systems(Update, wait_for_configs.run_if(in_state(AppState::Loading)));

        info!("✅ GameConfigPlugin initialized");
        info!(
            "🔍 All configs load requested (fruits, physics, game_rules, bounce, droplet, flash, shake, watermelon)"
        );
    }
}

// ---------------------------------------------------------------------------
// wait_for_configs
// ---------------------------------------------------------------------------

/// Transitions from `Loading` → `Title` once all required RON configs are ready.
fn wait_for_configs(configs: AllConfigs, mut next_state: ResMut<NextState<AppState>>) {
    if configs
        .physics_assets
        .get(&configs.physics_handle.0)
        .is_some()
        && configs
            .fruits_assets
            .get(&configs.fruits_handle.0)
            .is_some()
        && configs
            .game_rules_assets
            .get(&configs.game_rules_handle.0)
            .is_some()
        && configs
            .bounce_assets
            .get(&configs.bounce_handle.0)
            .is_some()
        && configs
            .droplet_assets
            .get(&configs.droplet_handle.0)
            .is_some()
        && configs.flash_assets.get(&configs.flash_handle.0).is_some()
        && configs.shake_assets.get(&configs.shake_handle.0).is_some()
        && configs
            .watermelon_assets
            .get(&configs.watermelon_handle.0)
            .is_some()
    {
        info!(
            "✅ All configs loaded (physics, fruits, game_rules, bounce, droplet, flash, shake, watermelon), transitioning to Title"
        );
        next_state.set(AppState::Title);
    }
}
