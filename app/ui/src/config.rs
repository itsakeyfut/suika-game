//! UI layout configuration loaded from RON files.
//!
//! Each HUD widget has its own config file so it can be tuned independently:
//!
//! | File                          | Config type         | Controls                    |
//! |-------------------------------|---------------------|-----------------------------|
//! | `config/ui/hud/layout.ron`     | [`HudLayoutConfig`]     | Widget anchor positions         |
//! | `config/ui/hud/score.ron`      | [`ScoreHudConfig`]      | Score panel padding/gap         |
//! | `config/ui/hud/best_score.ron` | [`BestScoreHudConfig`]  | Best-score panel padding        |
//! | `config/ui/hud/next.ron`       | [`NextHudConfig`]       | Next-fruit preview size         |
//! | `config/ui/hud/score_popup.ron`| [`ScorePopupConfig`]    | Floating score popup visuals    |
//!
//! All files are watched by Bevy's asset server, so edits take effect while
//! the game is running (hot-reload).

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Macro — reusable RON loader (mirrors the pattern in app/core/src/config.rs)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// HudLayoutConfig — widget anchor positions on screen
// ---------------------------------------------------------------------------

/// HUD anchor position configuration loaded from `config/ui/hud/layout.ron`.
///
/// Controls where each widget group is placed on screen.  Adjust these values
/// to reposition widgets without touching widget-level code.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct HudLayoutConfig {
    /// Margin from the screen edge for the score panels (pixels).
    pub edge_margin: f32,
    /// Horizontal distance between the best-score and score panel anchors (pixels).
    pub score_panel_offset: f32,
    /// Distance from the top of the screen for the next-fruit anchor (pixels).
    pub next_top: f32,
    /// Distance from the right edge of the screen for the next-fruit anchor (pixels).
    pub next_right: f32,
}

impl Default for HudLayoutConfig {
    fn default() -> Self {
        Self {
            edge_margin: 16.0,
            score_panel_offset: 160.0,
            next_top: 40.0,
            next_right: 300.0,
        }
    }
}

/// Resource holding the handle to the loaded [`HudLayoutConfig`].
#[derive(Resource)]
pub struct HudLayoutConfigHandle(pub Handle<HudLayoutConfig>);

ron_asset_loader!(HudLayoutConfigLoader, HudLayoutConfig);

// ---------------------------------------------------------------------------
// ScoreHudConfig — current-score panel appearance
// ---------------------------------------------------------------------------

// Default values — mirror `config/ui/hud/score.ron`
const DEFAULT_SCORE_PANEL_PADDING: f32 = 10.0;
const DEFAULT_SCORE_LABEL_VALUE_GAP: f32 = 4.0;
const DEFAULT_SCORE_PULSE_DURATION: f32 = 0.35;
const DEFAULT_SCORE_PULSE_PEAK_SCALE: f32 = 1.4;

/// Score panel configuration loaded from `config/ui/hud/score.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ScoreHudConfig {
    /// Inner padding of the panel node (pixels).
    pub panel_padding: f32,
    /// Vertical gap between the label and the value text (pixels).
    pub label_value_gap: f32,
    /// Duration of the score-beats-highscore pulse animation (seconds).
    pub pulse_duration: f32,
    /// Peak scale factor at the midpoint of the pulse (1.0 = no change).
    pub pulse_peak_scale: f32,
}

impl Default for ScoreHudConfig {
    fn default() -> Self {
        Self {
            panel_padding: DEFAULT_SCORE_PANEL_PADDING,
            label_value_gap: DEFAULT_SCORE_LABEL_VALUE_GAP,
            pulse_duration: DEFAULT_SCORE_PULSE_DURATION,
            pulse_peak_scale: DEFAULT_SCORE_PULSE_PEAK_SCALE,
        }
    }
}

/// Resource holding the handle to the loaded [`ScoreHudConfig`].
#[derive(Resource)]
pub struct ScoreHudConfigHandle(pub Handle<ScoreHudConfig>);

ron_asset_loader!(ScoreHudConfigLoader, ScoreHudConfig);

// ---------------------------------------------------------------------------
// BestScoreHudConfig — best-score panel appearance
// ---------------------------------------------------------------------------

// Default values — mirror `config/ui/hud/best_score.ron`
const DEFAULT_BEST_SCORE_PANEL_PADDING: f32 = 10.0;
const DEFAULT_BEST_SCORE_LABEL_VALUE_GAP: f32 = 4.0;

/// Best-score panel configuration loaded from `config/ui/hud/best_score.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct BestScoreHudConfig {
    /// Inner padding of the panel node (pixels).
    pub panel_padding: f32,
    /// Vertical gap between the label and the value text (pixels).
    pub label_value_gap: f32,
}

impl Default for BestScoreHudConfig {
    fn default() -> Self {
        Self {
            panel_padding: DEFAULT_BEST_SCORE_PANEL_PADDING,
            label_value_gap: DEFAULT_BEST_SCORE_LABEL_VALUE_GAP,
        }
    }
}

/// Resource holding the handle to the loaded [`BestScoreHudConfig`].
#[derive(Resource)]
pub struct BestScoreHudConfigHandle(pub Handle<BestScoreHudConfig>);

ron_asset_loader!(BestScoreHudConfigLoader, BestScoreHudConfig);

// ---------------------------------------------------------------------------
// NextHudConfig — next-fruit preview appearance
// ---------------------------------------------------------------------------

/// Next-fruit widget configuration loaded from `config/ui/hud/next.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct NextHudConfig {
    /// Diameter of the next-fruit preview circle (pixels).
    pub preview_size: f32,
}

impl Default for NextHudConfig {
    fn default() -> Self {
        Self { preview_size: 80.0 }
    }
}

/// Resource holding the handle to the loaded [`NextHudConfig`].
#[derive(Resource)]
pub struct NextHudConfigHandle(pub Handle<NextHudConfig>);

ron_asset_loader!(NextHudConfigLoader, NextHudConfig);

// ---------------------------------------------------------------------------
// ScorePopupConfig — floating score popup appearance
// ---------------------------------------------------------------------------

// Default values — mirror `config/ui/hud/score_popup.ron`
const DEFAULT_POPUP_DURATION: f32 = 1.0;
const DEFAULT_POPUP_RISE_DISTANCE: f32 = 80.0;
const DEFAULT_POPUP_FONT_SIZE_PER_RADIUS: f32 = 0.8;
const DEFAULT_POPUP_FADE_START_FRACTION: f32 = 0.5;
const DEFAULT_POPUP_RAINBOW_HUE_SPEED: f32 = 180.0;
const DEFAULT_POPUP_Z_LAYER: f32 = 8.0;

/// Floating score popup configuration loaded from `config/ui/hud/score_popup.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ScorePopupConfig {
    /// Total display duration before the popup despawns (seconds).
    pub duration: f32,
    /// Total vertical distance traveled over `duration` (pixels).
    pub rise_distance: f32,
    /// Font size = resulting fruit radius × this multiplier.
    pub font_size_per_radius: f32,
    /// Fraction of `duration` at which the alpha fade-out begins (0.0–1.0).
    pub fade_start_fraction: f32,
    /// Angular speed of the hue rotation in rainbow mode (degrees/second).
    pub rainbow_hue_speed: f32,
    /// Z depth for the popup text entity — renders above game objects.
    pub z_layer: f32,
}

impl Default for ScorePopupConfig {
    fn default() -> Self {
        Self {
            duration: DEFAULT_POPUP_DURATION,
            rise_distance: DEFAULT_POPUP_RISE_DISTANCE,
            font_size_per_radius: DEFAULT_POPUP_FONT_SIZE_PER_RADIUS,
            fade_start_fraction: DEFAULT_POPUP_FADE_START_FRACTION,
            rainbow_hue_speed: DEFAULT_POPUP_RAINBOW_HUE_SPEED,
            z_layer: DEFAULT_POPUP_Z_LAYER,
        }
    }
}

/// Resource holding the handle to the loaded [`ScorePopupConfig`].
#[derive(Resource)]
pub struct ScorePopupConfigHandle(pub Handle<ScorePopupConfig>);

ron_asset_loader!(ScorePopupConfigLoader, ScorePopupConfig);

// ---------------------------------------------------------------------------
// Hot-reload systems
// ---------------------------------------------------------------------------

/// Updates HUD anchor node positions when `config/ui/hud/layout.ron` changes.
#[allow(clippy::type_complexity)]
pub fn hot_reload_hud_layout(
    mut events: MessageReader<AssetEvent<HudLayoutConfig>>,
    config_assets: Res<Assets<HudLayoutConfig>>,
    config_handle: Res<HudLayoutConfigHandle>,
    mut best_score_q: Query<
        &mut Node,
        (
            With<crate::screens::hud::HudBestScoreAnchor>,
            Without<crate::screens::hud::HudScoreAnchor>,
            Without<crate::screens::hud::HudNextAnchor>,
        ),
    >,
    mut score_q: Query<
        &mut Node,
        (
            With<crate::screens::hud::HudScoreAnchor>,
            Without<crate::screens::hud::HudBestScoreAnchor>,
            Without<crate::screens::hud::HudNextAnchor>,
        ),
    >,
    mut next_q: Query<
        &mut Node,
        (
            With<crate::screens::hud::HudNextAnchor>,
            Without<crate::screens::hud::HudBestScoreAnchor>,
            Without<crate::screens::hud::HudScoreAnchor>,
        ),
    >,
) {
    for event in events.read() {
        if let AssetEvent::Modified { .. } = event
            && let Some(cfg) = config_assets.get(&config_handle.0)
        {
            if let Ok(mut node) = best_score_q.single_mut() {
                node.top = Val::Px(cfg.edge_margin);
                node.left = Val::Px(cfg.edge_margin);
            }
            if let Ok(mut node) = score_q.single_mut() {
                node.top = Val::Px(cfg.edge_margin);
                node.left = Val::Px(cfg.edge_margin + cfg.score_panel_offset);
            }
            if let Ok(mut node) = next_q.single_mut() {
                node.top = Val::Px(cfg.next_top);
                node.right = Val::Px(cfg.next_right);
            }
            info!("🔥 HUD layout config hot-reloaded");
        }
    }
}

/// Updates the score panel [`Node`] when `config/ui/hud/score.ron` changes.
pub fn hot_reload_score_hud(
    mut events: MessageReader<AssetEvent<ScoreHudConfig>>,
    config_assets: Res<Assets<ScoreHudConfig>>,
    config_handle: Res<ScoreHudConfigHandle>,
    mut panel_q: Query<&mut Node, With<crate::screens::hud::score::HudScorePanel>>,
) {
    for event in events.read() {
        if let AssetEvent::Modified { .. } = event
            && let Some(cfg) = config_assets.get(&config_handle.0)
        {
            if let Ok(mut node) = panel_q.single_mut() {
                node.padding = UiRect::all(Val::Px(cfg.panel_padding));
                node.row_gap = Val::Px(cfg.label_value_gap);
            }
            info!("🔥 Score HUD config hot-reloaded");
        }
    }
}

/// Updates the best-score panel [`Node`] when `config/ui/hud/best_score.ron` changes.
pub fn hot_reload_best_score_hud(
    mut events: MessageReader<AssetEvent<BestScoreHudConfig>>,
    config_assets: Res<Assets<BestScoreHudConfig>>,
    config_handle: Res<BestScoreHudConfigHandle>,
    mut panel_q: Query<&mut Node, With<crate::screens::hud::best_score::HudBestScorePanel>>,
) {
    for event in events.read() {
        if let AssetEvent::Modified { .. } = event
            && let Some(cfg) = config_assets.get(&config_handle.0)
        {
            if let Ok(mut node) = panel_q.single_mut() {
                node.padding = UiRect::all(Val::Px(cfg.panel_padding));
                node.row_gap = Val::Px(cfg.label_value_gap);
            }
            info!("🔥 Best-score HUD config hot-reloaded");
        }
    }
}

/// Updates the next-fruit preview circle size when `config/ui/hud/next.ron` changes.
pub fn hot_reload_next_hud(
    mut events: MessageReader<AssetEvent<NextHudConfig>>,
    config_assets: Res<Assets<NextHudConfig>>,
    config_handle: Res<NextHudConfigHandle>,
    mut preview_q: Query<&mut Node, With<crate::screens::hud::next::HudNextPreview>>,
) {
    for event in events.read() {
        if let AssetEvent::Modified { .. } = event
            && let Some(cfg) = config_assets.get(&config_handle.0)
        {
            if let Ok(mut node) = preview_q.single_mut() {
                node.width = Val::Px(cfg.preview_size);
                node.height = Val::Px(cfg.preview_size);
            }
            info!("🔥 Next-fruit HUD config hot-reloaded");
        }
    }
}

/// No-op hot-reload handler for `config/ui/hud/score_popup.ron`.
///
/// The popup systems read the config handle directly each frame, so there is
/// nothing to update reactively here.  This system exists solely to log a
/// message when the file changes, making it easy to verify that hot-reload is
/// working during development.
pub fn hot_reload_score_popup(mut events: MessageReader<AssetEvent<ScorePopupConfig>>) {
    for event in events.read() {
        if let AssetEvent::Modified { .. } = event {
            info!("🔥 Score popup config hot-reloaded");
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin that registers all UI config asset types, starts loading the files,
/// and adds hot-reload systems.
///
/// Mirrors the structure of `GameConfigPlugin` in `suika-game-core`.
/// Added automatically by [`crate::GameUIPlugin`].
pub struct UiConfigPlugin;

impl Plugin for UiConfigPlugin {
    fn build(&self, app: &mut App) {
        // Register asset types and their loaders
        app.init_asset::<HudLayoutConfig>()
            .register_asset_loader(HudLayoutConfigLoader)
            .init_asset::<ScoreHudConfig>()
            .register_asset_loader(ScoreHudConfigLoader)
            .init_asset::<BestScoreHudConfig>()
            .register_asset_loader(BestScoreHudConfigLoader)
            .init_asset::<NextHudConfig>()
            .register_asset_loader(NextHudConfigLoader)
            .init_asset::<ScorePopupConfig>()
            .register_asset_loader(ScorePopupConfigLoader);

        // Load all config files and store handles as resources
        let asset_server = app.world_mut().resource::<AssetServer>();

        let layout_handle: Handle<HudLayoutConfig> = asset_server.load("config/ui/hud/layout.ron");
        let score_handle: Handle<ScoreHudConfig> = asset_server.load("config/ui/hud/score.ron");
        let best_score_handle: Handle<BestScoreHudConfig> =
            asset_server.load("config/ui/hud/best_score.ron");
        let next_handle: Handle<NextHudConfig> = asset_server.load("config/ui/hud/next.ron");
        let score_popup_handle: Handle<ScorePopupConfig> =
            asset_server.load("config/ui/hud/score_popup.ron");

        app.insert_resource(HudLayoutConfigHandle(layout_handle))
            .insert_resource(ScoreHudConfigHandle(score_handle))
            .insert_resource(BestScoreHudConfigHandle(best_score_handle))
            .insert_resource(NextHudConfigHandle(next_handle))
            .insert_resource(ScorePopupConfigHandle(score_popup_handle));

        // Add hot-reload systems
        app.add_systems(
            Update,
            (
                hot_reload_hud_layout,
                hot_reload_score_hud,
                hot_reload_best_score_hud,
                hot_reload_next_hud,
                hot_reload_score_popup,
            ),
        );

        info!("✅ UiConfigPlugin initialized");
    }
}
