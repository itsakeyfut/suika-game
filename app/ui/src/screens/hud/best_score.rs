//! Best score widget.
//!
//! Renders a single labelled panel showing the all-time highscore.
//! Positioning is left to the caller — typically [`super::setup_hud`] wraps
//! this widget in an absolute-positioned anchor node.
//!
//! # Usage
//!
//! ```ignore
//! parent_anchor.with_children(|p| best_score::spawn_best_score_widget(p, &font, &cfg));
//! app.add_systems(Update, best_score::update_best_score.run_if(in_state(AppState::Playing)));
//! ```

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use suika_game_core::prelude::GameState;

use crate::config::BestScoreHudConfig;
use crate::screens::title::format_score;
use crate::styles::{BG_COLOR, FONT_SIZE_MEDIUM, FONT_SIZE_SMALL, PRIMARY_COLOR, TEXT_COLOR};

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the [`Text`] node that displays the all-time best score value.
#[derive(Component, Debug)]
pub struct HudBestScore;

/// Marks the container [`Node`] of the best-score panel.
///
/// Used by the hot-reload system in [`crate::config`] to update padding and gap
/// values at runtime without re-spawning the HUD.
#[derive(Component, Debug)]
pub struct HudBestScorePanel;

// ---------------------------------------------------------------------------
// Spawn helper
// ---------------------------------------------------------------------------

/// Spawns the best-score panel as a child of `parent`.
///
/// Layout values (`panel_padding`, `label_value_gap`) come from `cfg`.
///
/// ```text
/// ┌──────────────┐
/// │  ベストスコア │  ← FONT_SIZE_SMALL, TEXT_COLOR
/// │    16,777     │  ← FONT_SIZE_MEDIUM, PRIMARY_COLOR, HudBestScore
/// └──────────────┘
/// ```
pub fn spawn_best_score_widget(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    cfg: &BestScoreHudConfig,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(cfg.panel_padding)),
                row_gap: Val::Px(cfg.label_value_gap),
                ..default()
            },
            BackgroundColor(BG_COLOR),
            BorderRadius::all(Val::Px(8.0)),
            HudBestScorePanel,
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("ベストスコア"),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_SMALL,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
            panel.spawn((
                Text::new("0"),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_MEDIUM,
                    ..default()
                },
                TextColor(PRIMARY_COLOR),
                HudBestScore,
            ));
        });
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Updates the best-score text node from [`GameState`].
pub fn update_best_score(
    game_state: Res<GameState>,
    mut best_score_q: Query<&mut Text, With<HudBestScore>>,
) {
    if let Ok(mut text) = best_score_q.single_mut() {
        text.0 = format_score(game_state.highscore);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_best_score_marker_exists() {
        let _b = HudBestScore;
    }

    #[test]
    fn test_hud_best_score_panel_marker_exists() {
        let _p = HudBestScorePanel;
    }
}
