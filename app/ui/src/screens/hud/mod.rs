//! In-game HUD — gameplay UI layout.
//!
//! This module is responsible solely for **where** each widget appears on
//! screen.  The visual appearance and data-update logic of every widget live
//! in their own sub-modules:
//!
//! | Module        | Widget              |
//! |---------------|---------------------|
//! | [`best_score`]| ベストスコアパネル   |
//! | [`score`]     | スコアパネル        |
//! | [`next`]      | ネクストラベル      |
//!
//! # Layout
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │  [ベストスコア]  [スコア]              [ネクスト]         │
//! │                                                          │
//! │                  [game container]                        │
//! └──────────────────────────────────────────────────────────┘
//! ```
//!
//! # Adding a new widget
//!
//! 1. Create `hud/<widget>.rs` with a `spawn_<widget>` function and optional
//!    `update_<widget>` system.
//! 2. Add `pub mod <widget>;` below.
//! 3. Call `spawn_<widget>` from an **independent** anchor node inside
//!    [`setup_hud`] so its position can be adjusted without touching other
//!    widgets.
//! 4. Register `update_<widget>` in [`crate::GameUIPlugin`].

pub mod best_score;
pub mod next;
pub mod score;
pub mod score_popup;

use bevy::prelude::*;
use suika_game_core::prelude::{AppState, SettingsResource};

use crate::config::{
    BestScoreHudConfig, BestScoreHudConfigHandle, HudLayoutConfig, HudLayoutConfigHandle,
    NextHudConfig, NextHudConfigHandle, ScoreHudConfig, ScoreHudConfigHandle,
};
use crate::styles::FONT_JP;

// ---------------------------------------------------------------------------
// Anchor marker components (used by hot-reload systems in config.rs)
// ---------------------------------------------------------------------------

/// Marks the absolute-positioned anchor node that holds the best-score widget.
#[derive(Component)]
pub struct HudBestScoreAnchor;

/// Marks the absolute-positioned anchor node that holds the current-score widget.
#[derive(Component)]
pub struct HudScoreAnchor;

/// Marks the absolute-positioned anchor node that holds the next-fruit widget.
#[derive(Component)]
pub struct HudNextAnchor;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the full HUD overlay when entering [`AppState::Playing`].
///
/// Reads layout values from the per-widget RON configs when available,
/// falling back to built-in defaults otherwise.
/// Creates a transparent full-screen root node and positions each widget
/// inside absolute-positioned anchor containers.  Add new widgets here.
#[allow(clippy::too_many_arguments)]
pub fn setup_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<SettingsResource>,
    layout_handle: Res<HudLayoutConfigHandle>,
    layout_assets: Res<Assets<HudLayoutConfig>>,
    score_handle: Res<ScoreHudConfigHandle>,
    score_assets: Res<Assets<ScoreHudConfig>>,
    best_score_handle: Res<BestScoreHudConfigHandle>,
    best_score_assets: Res<Assets<BestScoreHudConfig>>,
    next_handle: Res<NextHudConfigHandle>,
    next_assets: Res<Assets<NextHudConfig>>,
) {
    let font: Handle<Font> = asset_server.load(FONT_JP);
    let lang = settings.language;

    let default_layout = HudLayoutConfig::default();
    let default_score = ScoreHudConfig::default();
    let default_best_score = BestScoreHudConfig::default();
    let default_next = NextHudConfig::default();

    let layout = layout_assets
        .get(&layout_handle.0)
        .unwrap_or(&default_layout);
    let score_cfg = score_assets.get(&score_handle.0).unwrap_or(&default_score);
    let best_score_cfg = best_score_assets
        .get(&best_score_handle.0)
        .unwrap_or(&default_best_score);
    let next_cfg = next_assets.get(&next_handle.0).unwrap_or(&default_next);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::NONE),
            DespawnOnExit(AppState::Playing),
        ))
        .with_children(|root| {
            // ------------------------------------------------------------------
            // Top-left: best score widget
            // ------------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(layout.edge_margin),
                    left: Val::Px(layout.edge_margin),
                    ..default()
                },
                HudBestScoreAnchor,
            ))
            .with_children(|anchor| {
                best_score::spawn_best_score_widget(anchor, &font, best_score_cfg, lang);
            });

            // ------------------------------------------------------------------
            // Top-left (second): current score widget
            // ------------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(layout.edge_margin),
                    left: Val::Px(layout.edge_margin + layout.score_panel_offset),
                    ..default()
                },
                HudScoreAnchor,
            ))
            .with_children(|anchor| {
                score::spawn_score_widget(anchor, &font, score_cfg, lang);
            });

            // ------------------------------------------------------------------
            // Right side: next-fruit widget
            // ------------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(layout.next_top),
                    right: Val::Px(layout.next_right),
                    ..default()
                },
                HudNextAnchor,
            ))
            .with_children(|anchor| {
                next::spawn_next_widget(anchor, &font, next_cfg, lang);
            });
        });
}

// ---------------------------------------------------------------------------
// Helper — elapsed time formatter (available to timer widget when added)
// ---------------------------------------------------------------------------

/// Formats elapsed seconds as `M:SS`.
///
/// # Examples
///
/// ```
/// # use suika_game_ui::screens::hud::format_elapsed;
/// assert_eq!(format_elapsed(0),    "0:00");
/// assert_eq!(format_elapsed(65),   "1:05");
/// assert_eq!(format_elapsed(3661), "61:01");
/// ```
pub fn format_elapsed(total_secs: u32) -> String {
    format!("{}:{:02}", total_secs / 60, total_secs % 60)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_elapsed_zero() {
        assert_eq!(format_elapsed(0), "0:00");
    }

    #[test]
    fn test_format_elapsed_under_one_minute() {
        assert_eq!(format_elapsed(5), "0:05");
        assert_eq!(format_elapsed(59), "0:59");
    }

    #[test]
    fn test_format_elapsed_one_minute() {
        assert_eq!(format_elapsed(60), "1:00");
        assert_eq!(format_elapsed(65), "1:05");
    }

    #[test]
    fn test_format_elapsed_large() {
        assert_eq!(format_elapsed(3661), "61:01");
    }
}
