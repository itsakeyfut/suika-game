//! Game-over screen — shown when the player's fruit stack overflows the boundary.
//!
//! Spawns a full-screen layout containing:
//! - A **GAME OVER** heading
//! - The **final score** in large text
//! - A **NEW RECORD!** banner when a new highscore was achieved
//! - The **all-time highscore**
//! - The **elapsed time** for this run in `M:SS` format
//! - A **Retry** button (→ [`AppState::Playing`])
//! - A **Title** button (→ [`AppState::Title`])
//!
//! All entities are tagged with [`DespawnOnExit`]`(`[`AppState::GameOver`]`)` so
//! Bevy automatically despawns them when the state transitions away from
//! `GameOver`.

use bevy::prelude::*;
use suika_game_core::prelude::{AppState, GameState, SettingsResource};

use crate::components::{ButtonAction, KeyboardFocusIndex, spawn_button};
use crate::i18n::t;
use crate::screens::hud::format_elapsed;
use crate::styles::{
    BG_COLOR, BUTTON_LARGE_HEIGHT, BUTTON_LARGE_WIDTH, BUTTON_MEDIUM_HEIGHT, BUTTON_MEDIUM_WIDTH,
    FONT_JP, FONT_SIZE_HUGE, FONT_SIZE_LARGE, FONT_SIZE_MEDIUM, FONT_SIZE_SMALL, HIGHLIGHT_COLOR,
    PRIMARY_COLOR, TEXT_COLOR,
};

// ---------------------------------------------------------------------------
// Color constants (local to this screen)
// ---------------------------------------------------------------------------

/// Red tone used for the "GAME OVER" heading.
const GAME_OVER_COLOR: Color = Color::srgb(0.8, 0.2, 0.2);

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the game-over screen UI when entering [`AppState::GameOver`].
///
/// Reads [`GameState`] to display the final score, the all-time highscore, and
/// whether this run set a new record.
///
/// This system is registered with `.after(`[`GameOverSet::SaveHighscore`]`)` in
/// [`GameUIPlugin`] so it is guaranteed to run after `save_highscore_on_game_over`
/// has written [`GameState::is_new_record`] and updated [`GameState::highscore`].
///
/// Resets [`KeyboardFocusIndex`] to `0` so the Retry button always has focus.
pub fn setup_game_over_screen(
    mut commands: Commands,
    game_state: Res<GameState>,
    settings: Res<SettingsResource>,
    asset_server: Res<AssetServer>,
    mut keyboard_focus: ResMut<KeyboardFocusIndex>,
) {
    keyboard_focus.0 = 0;

    let font: Handle<Font> = asset_server.load(FONT_JP);
    let lang = settings.language;
    let is_new_record = game_state.is_new_record;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BG_COLOR),
            DespawnOnExit(AppState::GameOver),
        ))
        .with_children(|parent| {
            // Game-over heading
            parent.spawn((
                Text::new(t("game_over_title", lang)),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_HUGE,
                    ..default()
                },
                TextColor(GAME_OVER_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // Final score
            parent.spawn((
                Text::new(format!(
                    "{}: {}",
                    t("score", lang),
                    format_score(game_state.score)
                )),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_LARGE,
                    ..default()
                },
                TextColor(PRIMARY_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // NEW RECORD banner (only when the highscore was beaten)
            if is_new_record {
                parent.spawn((
                    Text::new(t("new_record", lang)),
                    TextFont {
                        font: font.clone(),
                        font_size: FONT_SIZE_MEDIUM,
                        ..default()
                    },
                    TextColor(HIGHLIGHT_COLOR),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));
            }

            // All-time highscore
            parent.spawn((
                Text::new(format!(
                    "{}: {}",
                    t("highscore", lang),
                    format_score(game_state.highscore)
                )),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_SMALL,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Elapsed time for this run
            parent.spawn((
                Text::new(format!(
                    "{}: {}",
                    t("elapsed_time", lang),
                    format_elapsed(game_state.elapsed_time as u32)
                )),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_SMALL,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Retry button (index 0 — initial keyboard focus)
            spawn_button(
                parent,
                t("btn_retry", lang),
                ButtonAction::RetryGame,
                0,
                FONT_SIZE_LARGE,
                BUTTON_LARGE_WIDTH,
                BUTTON_LARGE_HEIGHT,
                font.clone(),
            );

            // Go-to-title button (index 1)
            spawn_button(
                parent,
                t("btn_title", lang),
                ButtonAction::GoToTitle,
                1,
                FONT_SIZE_MEDIUM,
                BUTTON_MEDIUM_WIDTH,
                BUTTON_MEDIUM_HEIGHT,
                font.clone(),
            );
        });
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Formats an integer score with comma separators every three digits.
///
/// # Examples
///
/// ```
/// # use suika_game_ui::screens::game_over::format_score;
/// assert_eq!(format_score(0),       "0");
/// assert_eq!(format_score(1000),    "1,000");
/// assert_eq!(format_score(123456),  "123,456");
/// assert_eq!(format_score(1234567), "1,234,567");
/// ```
pub fn format_score(n: u32) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_score_zero() {
        assert_eq!(format_score(0), "0");
    }

    #[test]
    fn test_format_score_below_thousand() {
        assert_eq!(format_score(999), "999");
    }

    #[test]
    fn test_format_score_thousands() {
        assert_eq!(format_score(1_000), "1,000");
        assert_eq!(format_score(10_000), "10,000");
        assert_eq!(format_score(100_000), "100,000");
    }

    #[test]
    fn test_format_score_millions() {
        assert_eq!(format_score(1_000_000), "1,000,000");
        assert_eq!(format_score(1_234_567), "1,234,567");
    }

    #[test]
    fn test_format_score_u32_max() {
        // u32::MAX = 4,294,967,295
        assert_eq!(format_score(u32::MAX), "4,294,967,295");
    }
}
