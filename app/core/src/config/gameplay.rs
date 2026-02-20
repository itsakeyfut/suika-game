//! Gameplay configuration: fruits, physics, game rules
//!
//! Loaded from `assets/config/fruits.ron`, `assets/config/physics.ron`,
//! and `assets/config/game_rules.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_rapier2d::prelude::{
    Collider, ColliderMassProperties, DefaultRapierContext, Friction, RapierConfiguration,
    Restitution,
};
use serde::Deserialize;
use std::collections::HashMap;

use crate::components::{BottomWall, BoundaryLine, Container, Fruit, LeftWall, NextFruitPreview};

// ---------------------------------------------------------------------------
// Shared color type
// ---------------------------------------------------------------------------

/// RGBA color value for use in RON configuration files
///
/// `bevy::prelude::Color` does not implement `Deserialize`, so effect configs
/// use this lightweight struct and convert via `Into<Color>`.
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct RonColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<RonColor> for bevy::prelude::Color {
    fn from(c: RonColor) -> Self {
        bevy::prelude::Color::srgba(c.r, c.g, c.b, c.a)
    }
}

// ---------------------------------------------------------------------------
// FruitsConfig
// ---------------------------------------------------------------------------

/// Fruit configuration asset loaded from `assets/config/fruits.ron`
///
/// Contains parameters for all 11 fruit types including physics properties,
/// scoring values, and visual characteristics.
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

// ---------------------------------------------------------------------------
// PhysicsConfig
// ---------------------------------------------------------------------------

/// Physics configuration asset loaded from `assets/config/physics.ron`
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

// ---------------------------------------------------------------------------
// GameRulesConfig
// ---------------------------------------------------------------------------

/// Game rules configuration asset loaded from `assets/config/game_rules.ron`
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

// ---------------------------------------------------------------------------
// SystemParam bundles
// ---------------------------------------------------------------------------

/// SystemParam bundle for accessing [`FruitsConfig`].
///
/// Reduces every fruits-config read site from two `Option<Res<>>` parameters
/// down to one.  Call `.get()` to obtain `Option<&FruitsConfig>`.
#[derive(SystemParam)]
pub struct FruitsParams<'w> {
    handle: Option<Res<'w, FruitsConfigHandle>>,
    assets: Option<Res<'w, Assets<FruitsConfig>>>,
}

impl<'w> FruitsParams<'w> {
    /// Returns the currently loaded [`FruitsConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&FruitsConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }
}

/// SystemParam bundle for accessing [`PhysicsConfig`].
#[derive(SystemParam)]
pub struct PhysicsParams<'w> {
    handle: Option<Res<'w, PhysicsConfigHandle>>,
    assets: Option<Res<'w, Assets<PhysicsConfig>>>,
}

impl<'w> PhysicsParams<'w> {
    /// Returns the currently loaded [`PhysicsConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&PhysicsConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }
}

/// SystemParam bundle for accessing [`GameRulesConfig`].
#[derive(SystemParam)]
pub struct GameRulesParams<'w> {
    handle: Option<Res<'w, GameRulesConfigHandle>>,
    assets: Option<Res<'w, Assets<GameRulesConfig>>>,
}

impl<'w> GameRulesParams<'w> {
    /// Returns the currently loaded [`GameRulesConfig`], or `None` while loading.
    pub fn get(&self) -> Option<&GameRulesConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }
}

// ---------------------------------------------------------------------------
// Helper functions (used by hot-reload systems)
// ---------------------------------------------------------------------------

/// Updates Rapier's gravity setting when physics config changes
pub fn update_rapier_gravity(rapier_config: &mut RapierConfiguration, new_gravity: f32) {
    rapier_config.gravity = Vec2::new(0.0, new_gravity);
    info!("🎯 Gravity updated to: {}", new_gravity);
}

/// Updates a single container wall's position and collider when dimensions change
pub fn update_wall(
    transform: &mut Transform,
    collider: &mut Collider,
    sprite: &mut Sprite,
    is_bottom: bool,
    is_left: bool,
    config: &PhysicsConfig,
) {
    let half_width = config.container_width / 2.0;
    let half_height = config.container_height / 2.0;
    let thickness = config.wall_thickness;

    if is_bottom {
        let new_y = -half_height - thickness / 2.0;
        transform.translation.y = new_y;
        *collider = Collider::cuboid(half_width, thickness / 2.0);
        sprite.custom_size = Some(Vec2::new(config.container_width, thickness));
        info!(
            "🔧 Updated bottom wall: y={}, width={}",
            new_y, config.container_width
        );
    } else {
        let new_x = if is_left {
            -half_width - thickness / 2.0
        } else {
            half_width + thickness / 2.0
        };
        transform.translation.x = new_x;
        *collider = Collider::cuboid(thickness / 2.0, half_height);
        sprite.custom_size = Some(Vec2::new(thickness, config.container_height));
        info!(
            "🔧 Updated {} wall: x={}, height={}",
            if is_left { "left" } else { "right" },
            new_x,
            config.container_height
        );
    }
}

/// Checks if a fruit position is outside container bounds
pub fn is_out_of_bounds(position: Vec3, radius: f32, config: &PhysicsConfig) -> bool {
    let max_x = config.container_width / 2.0;
    let max_y = config.container_height / 2.0;
    position.x.abs() + radius > max_x || position.y.abs() + radius > max_y
}

/// Updates the preview display position and size when config changes
pub fn update_preview(
    transform: &mut Transform,
    sprite: &mut Sprite,
    physics_config: &PhysicsConfig,
    rules_config: &GameRulesConfig,
    fruits_config: &FruitsConfig,
    next_fruit_type: crate::fruit::FruitType,
) {
    let new_x = physics_config.container_width / 2.0 + rules_config.preview_x_offset;
    let new_y = physics_config.container_height / 2.0 + rules_config.preview_y_offset;
    transform.translation.x = new_x;
    transform.translation.y = new_y;

    if let Some(params) = next_fruit_type.try_parameters_from_config(fruits_config) {
        let preview_size = params.radius * 2.0 * rules_config.preview_scale;
        sprite.custom_size = Some(Vec2::splat(preview_size));
        info!(
            "🎨 Preview display updated to position ({:.1}, {:.1}), size={:.1}",
            new_x, new_y, preview_size
        );
    } else {
        warn!(
            "⚠️ No config entry for preview fruit {:?}, keeping previous size",
            next_fruit_type
        );
    }
}

/// Updates game timer resources when game rules config changes
pub fn update_game_timers(
    combo_timer: &mut crate::resources::ComboTimer,
    game_over_timer: &mut crate::resources::GameOverTimer,
    config: &GameRulesConfig,
) {
    combo_timer.combo_window = config.combo_window;
    combo_timer.combo_max = config.combo_max;
    game_over_timer.warning_threshold = config.game_over_timer;
    info!(
        "⏱️ Game timers updated: combo_window={:.1}s, combo_max={}, game_over={:.1}s",
        config.combo_window, config.combo_max, config.game_over_timer
    );
}

// ---------------------------------------------------------------------------
// Hot-reload systems
// ---------------------------------------------------------------------------

/// Handles hot-reloading of fruits configuration
#[allow(clippy::type_complexity)]
pub fn hot_reload_fruits_config(
    mut events: MessageReader<AssetEvent<FruitsConfig>>,
    config_assets: Res<Assets<FruitsConfig>>,
    config_handle: Res<FruitsConfigHandle>,
    mut fruits: Query<
        (
            &crate::fruit::FruitType,
            &mut Sprite,
            &mut Collider,
            &mut Restitution,
            &mut Friction,
            &mut ColliderMassProperties,
        ),
        With<crate::components::Fruit>,
    >,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Fruits config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                if let Some(config) = config_assets.get(&config_handle.0) {
                    info!(
                        "🔥 Hot-reloading fruits config! Loaded {} fruit types",
                        config.fruits.len()
                    );

                    let mut updated_count = 0;
                    let mut skipped_count = 0;
                    for (
                        fruit_type,
                        mut sprite,
                        mut collider,
                        mut restitution,
                        mut friction,
                        mut mass_props,
                    ) in fruits.iter_mut()
                    {
                        let Some(params) = fruit_type.try_parameters_from_config(config) else {
                            warn!(
                                "⚠️ No config entry for {:?} (index {}), skipping update",
                                fruit_type, *fruit_type as usize
                            );
                            skipped_count += 1;
                            continue;
                        };

                        sprite.custom_size = Some(Vec2::splat(params.radius * 2.0));
                        *collider = Collider::ball(params.radius);
                        restitution.coefficient = params.restitution;
                        *friction = Friction::coefficient(params.friction);
                        *mass_props = ColliderMassProperties::Mass(params.mass);
                        updated_count += 1;
                    }

                    if updated_count > 0 {
                        info!(
                            "✨ Updated {} fruit entities with new config parameters",
                            updated_count
                        );
                    }
                    if skipped_count > 0 {
                        warn!(
                            "⚠️ Skipped {} fruit entities due to missing config entries",
                            skipped_count
                        );
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

/// Handles hot-reloading of physics configuration
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn hot_reload_physics_config(
    mut events: MessageReader<AssetEvent<PhysicsConfig>>,
    config_assets: Res<Assets<PhysicsConfig>>,
    config_handle: Res<PhysicsConfigHandle>,
    mut rapier_query: Query<&mut RapierConfiguration, With<DefaultRapierContext>>,
    mut commands: Commands,
    mut walls_query: Query<
        (
            &mut Transform,
            &mut Collider,
            &mut Sprite,
            Option<&BottomWall>,
            Option<&LeftWall>,
        ),
        (With<Container>, Without<Fruit>, Without<BoundaryLine>),
    >,
    mut boundary_query: Query<
        &mut Transform,
        (With<BoundaryLine>, Without<Container>, Without<Fruit>),
    >,
    fruits_query: Query<
        (Entity, &Transform, &crate::fruit::FruitType),
        (With<Fruit>, Without<Container>, Without<BoundaryLine>),
    >,
    fruits: FruitsParams,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Physics config loaded");
                if let Some(config) = config_assets.get(&config_handle.0) {
                    if let Ok(mut rapier_config) = rapier_query.single_mut() {
                        update_rapier_gravity(&mut rapier_config, config.gravity);
                    }

                    for (mut transform, mut collider, mut sprite, bottom_wall, left_wall) in
                        walls_query.iter_mut()
                    {
                        update_wall(
                            &mut transform,
                            &mut collider,
                            &mut sprite,
                            bottom_wall.is_some(),
                            left_wall.is_some(),
                            config,
                        );
                    }
                    info!(
                        "✨ Container walls initialized from physics.ron ({}x{})",
                        config.container_width, config.container_height
                    );

                    if let Ok(mut transform) = boundary_query.single_mut() {
                        transform.translation.y = config.boundary_line_y;
                        info!(
                            "📐 Boundary line positioned at y={} (initial load)",
                            config.boundary_line_y
                        );
                    }
                }
            }
            AssetEvent::Modified { id: _ } => {
                if let Some(config) = config_assets.get(&config_handle.0) {
                    info!("🔥 Hot-reloading physics config!");
                    info!(
                        "   Gravity: {}, Container: {}x{}",
                        config.gravity, config.container_width, config.container_height
                    );

                    // CRITICAL: Delete out-of-bounds fruits BEFORE updating walls
                    let mut deleted_count = 0;
                    for (entity, transform, fruit_type) in fruits_query.iter() {
                        let radius = if let Some(fruits_config) = fruits.get() {
                            fruit_type
                                .try_parameters_from_config(fruits_config)
                                .map(|p| p.radius)
                                .unwrap_or(20.0)
                        } else {
                            20.0
                        };

                        if is_out_of_bounds(transform.translation, radius, config) {
                            commands.entity(entity).despawn();
                            deleted_count += 1;
                            info!(
                                "🗑️ Deleted out-of-bounds fruit {:?} at ({:.1}, {:.1}), radius={}",
                                fruit_type,
                                transform.translation.x,
                                transform.translation.y,
                                radius
                            );
                        }
                    }
                    if deleted_count > 0 {
                        info!("✨ Deleted {} out-of-bounds fruits", deleted_count);
                    }

                    if let Ok(mut rapier_config) = rapier_query.single_mut() {
                        update_rapier_gravity(&mut rapier_config, config.gravity);
                    } else {
                        warn!("⚠️ Failed to find RapierConfiguration component");
                    }

                    for (mut transform, mut collider, mut sprite, bottom_wall, left_wall) in
                        walls_query.iter_mut()
                    {
                        update_wall(
                            &mut transform,
                            &mut collider,
                            &mut sprite,
                            bottom_wall.is_some(),
                            left_wall.is_some(),
                            config,
                        );
                    }
                    info!(
                        "✨ Container walls updated to width={}, height={}",
                        config.container_width, config.container_height
                    );

                    if let Ok(mut transform) = boundary_query.single_mut() {
                        transform.translation.y = config.boundary_line_y;
                        info!("📐 Boundary line updated to y={}", config.boundary_line_y);
                    }
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
#[allow(clippy::too_many_arguments)]
pub fn hot_reload_game_rules_config(
    mut events: MessageReader<AssetEvent<GameRulesConfig>>,
    config_assets: Res<Assets<GameRulesConfig>>,
    config_handle: Res<GameRulesConfigHandle>,
    mut preview_query: Query<(&mut Transform, &mut Sprite), With<NextFruitPreview>>,
    physics: PhysicsParams,
    fruits: FruitsParams,
    next_fruit: Res<crate::resources::NextFruitType>,
    mut combo_timer: ResMut<crate::resources::ComboTimer>,
    mut game_over_timer: ResMut<crate::resources::GameOverTimer>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                if let Some(config) = config_assets.get(&config_handle.0) {
                    info!("✅ Game rules config loaded");
                    update_game_timers(&mut combo_timer, &mut game_over_timer, config);
                }
            }
            AssetEvent::Modified { id: _ } => {
                if let Some(config) = config_assets.get(&config_handle.0) {
                    info!("🔥 Hot-reloading game rules config!");
                    info!(
                        "   Spawnable fruits: {}, Combo window: {}s, Game over timer: {}s",
                        config.spawnable_fruit_count, config.combo_window, config.game_over_timer
                    );

                    update_game_timers(&mut combo_timer, &mut game_over_timer, config);

                    if let Some(physics_config) = physics.get()
                        && let Some(fruits_config) = fruits.get()
                        && let Ok((mut transform, mut sprite)) = preview_query.single_mut()
                    {
                        update_preview(
                            &mut transform,
                            &mut sprite,
                            physics_config,
                            config,
                            fruits_config,
                            next_fruit.get(),
                        );
                    }
                }
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Game rules config removed");
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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

    #[test]
    fn test_update_rapier_gravity() {
        use bevy_rapier2d::prelude::RapierConfiguration;

        let mut rapier_config = RapierConfiguration::new(100.0);
        rapier_config.gravity = Vec2::new(0.0, -980.0);

        update_rapier_gravity(&mut rapier_config, -1980.0);

        assert_eq!(rapier_config.gravity.x, 0.0);
        assert_eq!(rapier_config.gravity.y, -1980.0);
    }

    #[test]
    fn test_is_out_of_bounds() {
        let config = PhysicsConfig {
            gravity: -980.0,
            container_width: 400.0,
            container_height: 600.0,
            wall_thickness: 20.0,
            boundary_line_y: 300.0,
            wall_restitution: 0.2,
            wall_friction: 0.5,
            fruit_spawn_y_offset: 50.0,
            fruit_linear_damping: 0.5,
            fruit_angular_damping: 1.0,
            keyboard_move_speed: 300.0,
        };

        let radius = 20.0;

        assert!(!is_out_of_bounds(Vec3::new(0.0, 0.0, 0.0), radius, &config));
        assert!(!is_out_of_bounds(
            Vec3::new(100.0, 200.0, 0.0),
            radius,
            &config
        ));

        assert!(is_out_of_bounds(
            Vec3::new(300.0, 0.0, 0.0),
            radius,
            &config
        ));
        assert!(is_out_of_bounds(
            Vec3::new(0.0, 400.0, 0.0),
            radius,
            &config
        ));
        assert!(is_out_of_bounds(
            Vec3::new(-250.0, 0.0, 0.0),
            radius,
            &config
        ));
        assert!(is_out_of_bounds(
            Vec3::new(0.0, -350.0, 0.0),
            radius,
            &config
        ));

        assert!(is_out_of_bounds(
            Vec3::new(185.0, 0.0, 0.0),
            radius,
            &config
        ));
        assert!(!is_out_of_bounds(
            Vec3::new(175.0, 0.0, 0.0),
            radius,
            &config
        ));
    }
}
