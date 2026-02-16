//! Game configuration loaded from RON files
//!
//! This module handles loading and hot-reloading of game configuration
//! from RON (Rusty Object Notation) files in the assets directory.
//!
//! Supports hot-reloading: Edit config files while the game is running
//! and changes will be applied automatically.

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties, Friction, Restitution};
use serde::Deserialize;
use std::collections::HashMap;

/// Fruit configuration asset loaded from `assets/config/fruits.ron`
///
/// Contains parameters for all 11 fruit types including physics properties,
/// scoring values, and visual characteristics.
///
/// # Hot-reload Support
///
/// This asset supports hot-reloading. When the RON file is modified,
/// the changes are automatically detected and applied to the game.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct FruitsConfig {
    /// List of fruit configurations, indexed by FruitType enum order
    pub fruits: Vec<FruitConfigEntry>,
}

/// Configuration for a single fruit type
#[derive(Deserialize, Debug, Clone)]
pub struct FruitConfigEntry {
    /// Fruit type name (e.g., "Cherry", "Strawberry")
    pub name: String,
    /// Collision radius in pixels
    pub radius: f32,
    /// Points awarded when this fruit is created through merging
    pub points: u32,
    /// Restitution coefficient (bounciness, 0.0-1.0)
    pub restitution: f32,
    /// Friction coefficient (0.0-1.0)
    pub friction: f32,
    /// Mass multiplier (mass = radius² × mass_multiplier)
    pub mass_multiplier: f32,
}

/// Resource holding the handle to the loaded fruits configuration
#[derive(Resource)]
pub struct FruitsConfigHandle(pub Handle<FruitsConfig>);

/// Physics configuration asset loaded from `assets/config/physics.ron`
///
/// Contains parameters for physics simulation including gravity, container dimensions,
/// wall properties, and fruit physics behavior.
///
/// # Hot-reload Support
///
/// This asset supports hot-reloading. When the RON file is modified,
/// the changes are automatically detected and applied to the game.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct PhysicsConfig {
    /// Gravity acceleration in pixels per second squared (negative = downward)
    pub gravity: f32,
    /// Container (playable area) width in pixels
    pub container_width: f32,
    /// Container (playable area) height in pixels
    pub container_height: f32,
    /// Thickness of container walls in pixels
    pub wall_thickness: f32,
    /// Y position of boundary line (game over line) from container bottom
    pub boundary_line_y: f32,
    /// Wall restitution coefficient (bounciness, 0.0-1.0)
    pub wall_restitution: f32,
    /// Wall friction coefficient (0.0-1.0)
    pub wall_friction: f32,
    /// Distance from top of container to spawn held fruit
    pub fruit_spawn_y_offset: f32,
    /// Linear damping for fruit physics (reduces velocity over time)
    pub fruit_linear_damping: f32,
    /// Angular damping for fruit physics (reduces rotation over time)
    pub fruit_angular_damping: f32,
    /// Keyboard movement speed in pixels per second
    pub keyboard_move_speed: f32,
}

/// Resource holding the handle to the loaded physics configuration
#[derive(Resource)]
pub struct PhysicsConfigHandle(pub Handle<PhysicsConfig>);

/// Game rules configuration asset loaded from `assets/config/game_rules.ron`
///
/// Contains parameters for game mechanics, scoring, combo system, and UI behavior.
///
/// # Hot-reload Support
///
/// This asset supports hot-reloading. When the RON file is modified,
/// the changes are automatically detected and applied to the game.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct GameRulesConfig {
    /// Number of fruit types that can be spawned by player (1-11)
    pub spawnable_fruit_count: usize,
    /// Time window in seconds to maintain combo chain
    pub combo_window: f32,
    /// Maximum combo count (caps bonus multiplier)
    pub combo_max: u32,
    /// Seconds a fruit can stay above boundary line before game over
    pub game_over_timer: f32,
    /// Combo bonus multipliers (combo count -> multiplier)
    pub combo_bonuses: HashMap<u32, f32>,
    /// X offset from container edge for next fruit preview
    pub preview_x_offset: f32,
    /// Y offset from container top for next fruit preview
    pub preview_y_offset: f32,
    /// Size multiplier for preview display
    pub preview_scale: f32,
}

/// Resource holding the handle to the loaded game rules configuration
#[derive(Resource)]
pub struct GameRulesConfigHandle(pub Handle<GameRulesConfig>);

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
            .register_asset_loader(GameRulesConfigLoader);

        // Load all configs and insert handles immediately
        let asset_server = app.world_mut().resource::<AssetServer>();

        let fruits_handle: Handle<FruitsConfig> = asset_server.load("config/fruits.ron");
        let physics_handle: Handle<PhysicsConfig> = asset_server.load("config/physics.ron");
        let game_rules_handle: Handle<GameRulesConfig> = asset_server.load("config/game_rules.ron");

        app.insert_resource(FruitsConfigHandle(fruits_handle))
            .insert_resource(PhysicsConfigHandle(physics_handle))
            .insert_resource(GameRulesConfigHandle(game_rules_handle));

        // Add hot-reload systems
        app.add_systems(Update, (
            hot_reload_fruits_config,
            hot_reload_physics_config,
            hot_reload_game_rules_config,
        ));

        info!("✅ GameConfigPlugin initialized");
        info!("🔍 All configs load requested (fruits, physics, game_rules)");
    }
}

/// Handles hot-reloading of fruits configuration
///
/// Monitors for changes to the fruits.ron file and logs when updates are detected.
/// When the config is modified, this system updates all existing fruit entities
/// to reflect the new parameters.
fn hot_reload_fruits_config(
    mut events: MessageReader<AssetEvent<FruitsConfig>>,
    config_assets: Res<Assets<FruitsConfig>>,
    config_handle: Res<FruitsConfigHandle>,
    mut fruits: Query<(
        &crate::fruit::FruitType,
        &mut Sprite,
        &mut Collider,
        &mut Restitution,
        &mut Friction,
        &mut ColliderMassProperties,
    ), With<crate::components::Fruit>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Fruits config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                if let Some(config) = config_assets.get(&config_handle.0) {
                    info!("🔥 Hot-reloading fruits config! Loaded {} fruit types", config.fruits.len());

                    // Update all existing fruit entities with new parameters
                    let mut updated_count = 0;
                    for (fruit_type, mut sprite, mut collider, mut restitution, mut friction, mut mass_props) in fruits.iter_mut() {
                        let params = fruit_type.parameters_from_config(config);

                        // Update visual size
                        sprite.custom_size = Some(Vec2::splat(params.radius * 2.0));

                        // Update collision shape
                        *collider = Collider::ball(params.radius);

                        // Update physics properties
                        restitution.coefficient = params.restitution;
                        *friction = Friction::coefficient(params.friction);
                        *mass_props = ColliderMassProperties::Mass(params.mass);

                        updated_count += 1;
                    }

                    if updated_count > 0 {
                        info!("✨ Updated {} fruit entities with new config parameters", updated_count);
                    }
                }
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Fruits config removed");
            }
            _ => {}
        }
    }
}

/// Asset loader for FruitsConfig RON files
#[derive(Default)]
pub struct FruitsConfigLoader;

impl AssetLoader for FruitsConfigLoader {
    type Asset = FruitsConfig;
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
        let config: FruitsConfig = ron::de::from_bytes(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

/// Asset loader for PhysicsConfig RON files
#[derive(Default)]
pub struct PhysicsConfigLoader;

impl AssetLoader for PhysicsConfigLoader {
    type Asset = PhysicsConfig;
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
        let config: PhysicsConfig = ron::de::from_bytes(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

/// Asset loader for GameRulesConfig RON files
#[derive(Default)]
pub struct GameRulesConfigLoader;

impl AssetLoader for GameRulesConfigLoader {
    type Asset = GameRulesConfig;
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
        let config: GameRulesConfig = ron::de::from_bytes(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

/// Handles hot-reloading of physics configuration
///
/// Monitors for changes to the physics.ron file and logs when updates are detected.
/// When the config is modified, the changes are applied to game physics systems.
fn hot_reload_physics_config(
    mut events: MessageReader<AssetEvent<PhysicsConfig>>,
    config_assets: Res<Assets<PhysicsConfig>>,
    config_handle: Res<PhysicsConfigHandle>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Physics config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                if let Some(config) = config_assets.get(&config_handle.0) {
                    info!("🔥 Hot-reloading physics config!");
                    info!("   Gravity: {}, Container: {}x{}",
                        config.gravity, config.container_width, config.container_height);
                    // Note: Physics changes will be picked up by systems that read the config
                }
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Physics config removed");
            }
            _ => {}
        }
    }
}

/// Handles hot-reloading of game rules configuration
///
/// Monitors for changes to the game_rules.ron file and logs when updates are detected.
/// When the config is modified, the changes are applied to game mechanics.
fn hot_reload_game_rules_config(
    mut events: MessageReader<AssetEvent<GameRulesConfig>>,
    config_assets: Res<Assets<GameRulesConfig>>,
    config_handle: Res<GameRulesConfigHandle>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Game rules config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                if let Some(config) = config_assets.get(&config_handle.0) {
                    info!("🔥 Hot-reloading game rules config!");
                    info!("   Spawnable fruits: {}, Combo window: {}s, Game over timer: {}s",
                        config.spawnable_fruit_count, config.combo_window, config.game_over_timer);
                    // Note: Game rule changes will be picked up by systems that read the config
                }
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Game rules config removed");
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fruits_config_deserialization() {
        let ron_data = r#"
FruitsConfig(
    fruits: [
        (
            name: "Cherry",
            radius: 20.0,
            points: 10,
            restitution: 0.3,
            friction: 0.5,
            mass_multiplier: 0.01,
        ),
    ],
)
"#;

        let config: FruitsConfig = ron::de::from_str(ron_data).unwrap();
        assert_eq!(config.fruits.len(), 1);
        assert_eq!(config.fruits[0].name, "Cherry");
        assert_eq!(config.fruits[0].radius, 20.0);
        assert_eq!(config.fruits[0].points, 10);
    }

    #[test]
    fn test_physics_config_deserialization() {
        let ron_data = r#"
PhysicsConfig(
    gravity: -980.0,
    container_width: 600.0,
    container_height: 800.0,
    wall_thickness: 20.0,
    boundary_line_y: 300.0,
    wall_restitution: 0.2,
    wall_friction: 0.5,
    fruit_spawn_y_offset: 50.0,
    fruit_linear_damping: 0.5,
    fruit_angular_damping: 1.0,
    keyboard_move_speed: 300.0,
)
"#;

        let config: PhysicsConfig = ron::de::from_str(ron_data).unwrap();
        assert_eq!(config.gravity, -980.0);
        assert_eq!(config.container_width, 600.0);
        assert_eq!(config.container_height, 800.0);
        assert_eq!(config.wall_thickness, 20.0);
        assert_eq!(config.boundary_line_y, 300.0);
    }

    #[test]
    fn test_game_rules_config_deserialization() {
        let ron_data = r#"
GameRulesConfig(
    spawnable_fruit_count: 5,
    combo_window: 2.0,
    combo_max: 10,
    game_over_timer: 3.0,
    combo_bonuses: {
        2: 1.1,
        3: 1.2,
        4: 1.3,
        5: 1.5,
    },
    preview_x_offset: 120.0,
    preview_y_offset: -100.0,
    preview_scale: 1.5,
)
"#;

        let config: GameRulesConfig = ron::de::from_str(ron_data).unwrap();
        assert_eq!(config.spawnable_fruit_count, 5);
        assert_eq!(config.combo_window, 2.0);
        assert_eq!(config.combo_max, 10);
        assert_eq!(config.game_over_timer, 3.0);
        assert_eq!(config.combo_bonuses.get(&2), Some(&1.1));
        assert_eq!(config.combo_bonuses.get(&5), Some(&1.5));
    }
}
