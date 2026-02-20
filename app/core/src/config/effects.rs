//! Effect configuration: bounce, droplet, flash, shake, watermelon
//!
//! Loaded from `assets/config/effects/*.ron`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// BounceConfig
// ---------------------------------------------------------------------------

/// Bounce (Squash & Stretch) animation configuration
///
/// Loaded from `assets/config/effects/bounce.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct BounceConfig {
    pub merge_amplitude: f32,
    pub merge_frequency: f32,
    pub merge_damping: f32,
    pub landing_amplitude: f32,
    pub landing_frequency: f32,
    pub landing_damping: f32,
    pub settle_threshold: f32,
    pub settle_min_elapsed: f32,
}

/// Resource holding the handle to the loaded bounce configuration
#[derive(Resource)]
pub struct BounceConfigHandle(pub Handle<BounceConfig>);

/// SystemParam bundle for accessing [`BounceConfig`].
#[derive(SystemParam)]
pub struct BounceParams<'w> {
    handle: Option<Res<'w, BounceConfigHandle>>,
    assets: Option<Res<'w, Assets<BounceConfig>>>,
}

impl<'w> BounceParams<'w> {
    pub fn get(&self) -> Option<&BounceConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }
}

// ---------------------------------------------------------------------------
// DropletConfig
// ---------------------------------------------------------------------------

/// Water droplet particle effect configuration
///
/// Loaded from `assets/config/effects/droplet.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct DropletConfig {
    pub count_merge: u32,
    pub count_landing: u32,
    pub radius: f32,
    /// Base RGBA color used when `color_mode` is `Water`
    pub color: crate::config::gameplay::RonColor,
    pub color_mode: crate::systems::effects::droplet::DropletColorMode,
    pub min_speed: f32,
    pub max_speed: f32,
    pub lifetime_min: f32,
    pub lifetime_max: f32,
    pub gravity: f32,
    pub bounce_damping: f32,
}

/// Resource holding the handle to the loaded droplet configuration
#[derive(Resource)]
pub struct DropletConfigHandle(pub Handle<DropletConfig>);

/// SystemParam bundle for accessing [`DropletConfig`].
#[derive(SystemParam)]
pub struct DropletParams<'w> {
    handle: Option<Res<'w, DropletConfigHandle>>,
    assets: Option<Res<'w, Assets<DropletConfig>>>,
}

impl<'w> DropletParams<'w> {
    pub fn get(&self) -> Option<&DropletConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }
}

// ---------------------------------------------------------------------------
// FlashConfig
// ---------------------------------------------------------------------------

/// Flash visual effect configuration
///
/// Loaded from `assets/config/effects/flash.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct FlashConfig {
    pub local_duration: f32,
    pub local_initial_alpha: f32,
    pub local_size_multiplier: f32,
    pub screen_duration: f32,
    pub screen_initial_alpha: f32,
    pub screen_flash_min_index: usize,
}

/// Resource holding the handle to the loaded flash configuration
#[derive(Resource)]
pub struct FlashConfigHandle(pub Handle<FlashConfig>);

/// SystemParam bundle for accessing [`FlashConfig`].
#[derive(SystemParam)]
pub struct FlashParams<'w> {
    handle: Option<Res<'w, FlashConfigHandle>>,
    assets: Option<Res<'w, Assets<FlashConfig>>>,
}

impl<'w> FlashParams<'w> {
    pub fn get(&self) -> Option<&FlashConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }
}

// ---------------------------------------------------------------------------
// ShakeConfig
// ---------------------------------------------------------------------------

/// Camera shake effect configuration
///
/// Loaded from `assets/config/effects/shake.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct ShakeConfig {
    pub decay: f32,
    pub max_offset: f32,
    pub min_fruit_index: usize,
    pub intensity_step: f32,
}

/// Resource holding the handle to the loaded shake configuration
#[derive(Resource)]
pub struct ShakeConfigHandle(pub Handle<ShakeConfig>);

/// SystemParam bundle for accessing [`ShakeConfig`].
#[derive(SystemParam)]
pub struct ShakeParams<'w> {
    handle: Option<Res<'w, ShakeConfigHandle>>,
    assets: Option<Res<'w, Assets<ShakeConfig>>>,
}

impl<'w> ShakeParams<'w> {
    pub fn get(&self) -> Option<&ShakeConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }
}

// ---------------------------------------------------------------------------
// WatermelonConfig
// ---------------------------------------------------------------------------

/// Watermelon special-effect configuration
///
/// Loaded from `assets/config/effects/watermelon.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct WatermelonConfig {
    pub ring_duration: f32,
    pub ring_initial_diameter: f32,
    pub ring_expand_multiplier: f32,
    pub ring_initial_alpha: f32,
    pub burst_count: u32,
    pub burst_min_speed: f32,
    pub burst_max_speed: f32,
    pub burst_particle_size: f32,
    pub burst_lifetime: f32,
}

/// Resource holding the handle to the loaded watermelon effect configuration
#[derive(Resource)]
pub struct WatermelonConfigHandle(pub Handle<WatermelonConfig>);

/// SystemParam bundle for accessing [`WatermelonConfig`].
#[derive(SystemParam)]
pub struct WatermelonParams<'w> {
    handle: Option<Res<'w, WatermelonConfigHandle>>,
    assets: Option<Res<'w, Assets<WatermelonConfig>>>,
}

impl<'w> WatermelonParams<'w> {
    pub fn get(&self) -> Option<&WatermelonConfig> {
        self.handle
            .as_ref()
            .and_then(|h| self.assets.as_ref().and_then(|a| a.get(&h.0)))
    }
}

// ---------------------------------------------------------------------------
// Hot-reload systems
// ---------------------------------------------------------------------------

/// Handles hot-reloading of bounce effect configuration
pub fn hot_reload_bounce_config(
    mut events: MessageReader<AssetEvent<BounceConfig>>,
    config_assets: Res<Assets<BounceConfig>>,
    config_handle: Res<BounceConfigHandle>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Bounce effect config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                if let Some(config) = config_assets.get(&config_handle.0) {
                    info!(
                        "🔥 Hot-reloading bounce config! merge_amp={}, landing_amp={}",
                        config.merge_amplitude, config.landing_amplitude
                    );
                }
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Bounce effect config removed");
            }
            _ => {}
        }
    }
}

/// Handles hot-reloading of droplet effect configuration
pub fn hot_reload_droplet_config(
    mut events: MessageReader<AssetEvent<DropletConfig>>,
    config_assets: Res<Assets<DropletConfig>>,
    config_handle: Res<DropletConfigHandle>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Droplet effect config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                if let Some(config) = config_assets.get(&config_handle.0) {
                    info!(
                        "🔥 Hot-reloading droplet config! count_merge={}, count_landing={}",
                        config.count_merge, config.count_landing
                    );
                }
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Droplet effect config removed");
            }
            _ => {}
        }
    }
}

/// Handles hot-reloading of flash effect configuration
pub fn hot_reload_flash_config(
    mut events: MessageReader<AssetEvent<FlashConfig>>,
    config_assets: Res<Assets<FlashConfig>>,
    config_handle: Res<FlashConfigHandle>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Flash effect config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                if let Some(config) = config_assets.get(&config_handle.0) {
                    info!(
                        "🔥 Hot-reloading flash config! local_duration={}, screen_flash_min_index={}",
                        config.local_duration, config.screen_flash_min_index
                    );
                }
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Flash effect config removed");
            }
            _ => {}
        }
    }
}

/// Handles hot-reloading of shake effect configuration
pub fn hot_reload_shake_config(
    mut events: MessageReader<AssetEvent<ShakeConfig>>,
    config_assets: Res<Assets<ShakeConfig>>,
    config_handle: Res<ShakeConfigHandle>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Shake effect config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                if let Some(config) = config_assets.get(&config_handle.0) {
                    info!(
                        "🔥 Hot-reloading shake config! decay={}, max_offset={}, min_fruit_index={}",
                        config.decay, config.max_offset, config.min_fruit_index
                    );
                }
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Shake effect config removed");
            }
            _ => {}
        }
    }
}

/// Handles hot-reloading of watermelon effect configuration
pub fn hot_reload_watermelon_config(
    mut events: MessageReader<AssetEvent<WatermelonConfig>>,
    config_assets: Res<Assets<WatermelonConfig>>,
    config_handle: Res<WatermelonConfigHandle>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id: _ } => {
                info!("✅ Watermelon effect config loaded");
            }
            AssetEvent::Modified { id: _ } => {
                if let Some(config) = config_assets.get(&config_handle.0) {
                    info!(
                        "🔥 Hot-reloading watermelon config! burst_count={}, ring_duration={}",
                        config.burst_count, config.ring_duration
                    );
                }
            }
            AssetEvent::Removed { id: _ } => {
                warn!("⚠️ Watermelon effect config removed");
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
    fn test_bounce_config_deserialization() {
        let ron_data = r#"
BounceConfig(
    merge_amplitude: 0.3,
    merge_frequency: 18.0,
    merge_damping: 6.0,
    landing_amplitude: 0.18,
    landing_frequency: 22.0,
    landing_damping: 9.0,
    settle_threshold: 0.01,
    settle_min_elapsed: 0.3,
)
"#;
        let config: BounceConfig = ron::de::from_str(ron_data).unwrap();
        assert_eq!(config.merge_amplitude, 0.3);
        assert_eq!(config.merge_frequency, 18.0);
        assert_eq!(config.landing_amplitude, 0.18);
        assert_eq!(config.settle_threshold, 0.01);
    }

    #[test]
    fn test_droplet_config_deserialization() {
        let ron_data = r#"
DropletConfig(
    count_merge: 12,
    count_landing: 5,
    radius: 2.5,
    color: (r: 0.5, g: 0.78, b: 0.95, a: 0.85),
    color_mode: Juice,
    min_speed: 80.0,
    max_speed: 350.0,
    lifetime_min: 0.4,
    lifetime_max: 0.9,
    gravity: -600.0,
    bounce_damping: 0.55,
)
"#;
        let config: DropletConfig = ron::de::from_str(ron_data).unwrap();
        assert_eq!(config.count_merge, 12);
        assert_eq!(config.count_landing, 5);
        assert_eq!(config.radius, 2.5);
        assert_eq!(
            config.color_mode,
            crate::systems::effects::droplet::DropletColorMode::Juice
        );
        assert_eq!(config.min_speed, 80.0);
        assert_eq!(config.max_speed, 350.0);
        assert_eq!(config.gravity, -600.0);
        assert!((config.color.r - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_flash_config_deserialization() {
        let ron_data = r#"
FlashConfig(
    local_duration: 0.3,
    local_initial_alpha: 0.6,
    local_size_multiplier: 2.5,
    screen_duration: 0.25,
    screen_initial_alpha: 0.35,
    screen_flash_min_index: 8,
)
"#;
        let config: FlashConfig = ron::de::from_str(ron_data).unwrap();
        assert_eq!(config.local_duration, 0.3);
        assert_eq!(config.local_initial_alpha, 0.6);
        assert_eq!(config.screen_flash_min_index, 8);
    }

    #[test]
    fn test_watermelon_config_deserialization() {
        let ron_data = r#"
WatermelonConfig(
    ring_duration: 0.7,
    ring_initial_diameter: 160.0,
    ring_expand_multiplier: 5.0,
    ring_initial_alpha: 0.75,
    burst_count: 48,
    burst_min_speed: 150.0,
    burst_max_speed: 500.0,
    burst_particle_size: 5.0,
    burst_lifetime: 0.9,
)
"#;
        let config: WatermelonConfig = ron::de::from_str(ron_data).unwrap();
        assert!((config.ring_duration - 0.7).abs() < f32::EPSILON);
        assert_eq!(config.ring_initial_diameter, 160.0);
        assert_eq!(config.ring_expand_multiplier, 5.0);
        assert_eq!(config.burst_count, 48);
        assert_eq!(config.burst_min_speed, 150.0);
        assert_eq!(config.burst_max_speed, 500.0);
        assert_eq!(config.burst_particle_size, 5.0);
        assert!((config.burst_lifetime - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_shake_config_deserialization() {
        let ron_data = r#"
ShakeConfig(
    decay: 5.0,
    max_offset: 15.0,
    min_fruit_index: 5,
    intensity_step: 0.15,
)
"#;
        let config: ShakeConfig = ron::de::from_str(ron_data).unwrap();
        assert_eq!(config.decay, 5.0);
        assert_eq!(config.max_offset, 15.0);
        assert_eq!(config.min_fruit_index, 5);
        assert!((config.intensity_step - 0.15).abs() < f32::EPSILON);
    }
}
