//! UI layout configuration loaded from RON files.
//!
//! Each HUD widget has its own config file so it can be tuned independently:
//!
//! | File                          | Config type         | Controls                    |
//! |-------------------------------|---------------------|-----------------------------|
//! | `config/ui/hud/layout.ron`    | [`HudLayoutConfig`] | Widget anchor positions     |
//! | `config/ui/hud/score.ron`     | [`ScoreHudConfig`]  | Score panel padding/gap     |
//! | `config/ui/hud/best_score.ron`| [`BestScoreHudConfig`]| Best-score panel padding  |
//! | `config/ui/hud/next.ron`      | [`NextHudConfig`]   | Next-fruit preview size     |
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

/// Score panel configuration loaded from `config/ui/hud/score.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct ScoreHudConfig {
    /// Inner padding of the panel node (pixels).
    pub panel_padding: f32,
    /// Vertical gap between the label and the value text (pixels).
    pub label_value_gap: f32,
}

impl Default for ScoreHudConfig {
    fn default() -> Self {
        Self {
            panel_padding: 10.0,
            label_value_gap: 4.0,
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

/// Best-score panel configuration loaded from `config/ui/hud/best_score.ron`.
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct BestScoreHudConfig {
    /// Inner padding of the panel node (pixels).
    pub panel_padding: f32,
    /// Vertical gap between the label and the value text (pixels).
    pub label_value_gap: f32,
}

impl Default for BestScoreHudConfig {
    fn default() -> Self {
        Self {
            panel_padding: 10.0,
            label_value_gap: 4.0,
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
            .register_asset_loader(NextHudConfigLoader);

        // Load all config files and store handles as resources
        let asset_server = app.world_mut().resource::<AssetServer>();

        let layout_handle: Handle<HudLayoutConfig> = asset_server.load("config/ui/hud/layout.ron");
        let score_handle: Handle<ScoreHudConfig> = asset_server.load("config/ui/hud/score.ron");
        let best_score_handle: Handle<BestScoreHudConfig> =
            asset_server.load("config/ui/hud/best_score.ron");
        let next_handle: Handle<NextHudConfig> = asset_server.load("config/ui/hud/next.ron");

        app.insert_resource(HudLayoutConfigHandle(layout_handle))
            .insert_resource(ScoreHudConfigHandle(score_handle))
            .insert_resource(BestScoreHudConfigHandle(best_score_handle))
            .insert_resource(NextHudConfigHandle(next_handle));

        // Add hot-reload systems
        app.add_systems(
            Update,
            (
                hot_reload_hud_layout,
                hot_reload_score_hud,
                hot_reload_best_score_hud,
                hot_reload_next_hud,
            ),
        );

        info!("✅ UiConfigPlugin initialized");
    }
}
