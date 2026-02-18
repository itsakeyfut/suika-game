//! Current score widget.
//!
//! Renders a single labelled panel showing the score for the current game
//! session.  Positioning is left to the caller — typically [`super::setup_hud`]
//! wraps this widget in an absolute-positioned anchor node.
//!
//! # Usage
//!
//! ```ignore
//! parent_anchor.with_children(|p| score::spawn_score_widget(p, &font, &cfg));
//! app.add_systems(Update, score::update_score.run_if(in_state(AppState::Playing)));
//! ```

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use suika_game_core::prelude::GameState;

use crate::config::ScoreHudConfig;
use crate::screens::title::format_score;
use crate::styles::{BG_COLOR, FONT_SIZE_MEDIUM, FONT_SIZE_SMALL, PRIMARY_COLOR, TEXT_COLOR};

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the [`Text`] node that displays the current score value.
#[derive(Component, Debug)]
pub struct HudScore;

/// Marks the container [`Node`] of the score panel.
///
/// Used by the hot-reload system in [`crate::config`] to update padding and gap
/// values at runtime without re-spawning the HUD.
#[derive(Component, Debug)]
pub struct HudScorePanel;

// ---------------------------------------------------------------------------
// Spawn helper
// ---------------------------------------------------------------------------

/// Spawns the current-score panel as a child of `parent`.
///
/// Layout values (`panel_padding`, `label_value_gap`) come from `cfg`.
///
/// ```text
/// ┌──────────┐
/// │  スコア   │  ← FONT_SIZE_SMALL, TEXT_COLOR
/// │    0      │  ← FONT_SIZE_MEDIUM, PRIMARY_COLOR, HudScore
/// └──────────┘
/// ```
pub fn spawn_score_widget(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    cfg: &ScoreHudConfig,
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
            HudScorePanel,
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("スコア"),
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
                HudScore,
            ));
        });
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Updates the score text node from [`GameState`].
pub fn update_score(game_state: Res<GameState>, mut score_q: Query<&mut Text, With<HudScore>>) {
    if let Ok(mut text) = score_q.single_mut() {
        text.0 = format_score(game_state.score);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_score_marker_exists() {
        let _s = HudScore;
    }

    #[test]
    fn test_hud_score_panel_marker_exists() {
        let _p = HudScorePanel;
    }
}
