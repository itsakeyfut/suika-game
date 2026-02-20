//! Title screen — the first screen the player sees when the game starts.
//!
//! Spawns a full-screen layout containing:
//! - The game title at the top center
//! - **Start**, **Settings**, **How to Play**, and **Quit** buttons
//! - The all-time highscore at the bottom
//!
//! All entities are tagged with [`DespawnOnExit`]`(AppState::Title)` so Bevy
//! automatically despawns them when the state transitions away from `Title`.

use bevy::prelude::*;
use suika_game_core::prelude::{AppState, GameState, SettingsResource};

use crate::components::{ButtonAction, KeyboardFocusIndex, spawn_button};
use crate::i18n::t;
use crate::styles::{
    BG_COLOR, BUTTON_LARGE_HEIGHT, BUTTON_LARGE_WIDTH, FONT_JP, FONT_SIZE_HUGE, FONT_SIZE_LARGE,
    FONT_SIZE_SMALL, PRIMARY_COLOR, TEXT_COLOR,
};

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the title screen UI when entering [`AppState::Title`].
///
/// Resets [`KeyboardFocusIndex`] to `0` so the Start button always has focus
/// when (re-)entering this screen.
pub fn setup_title_screen(
    mut commands: Commands,
    game_state: Res<GameState>,
    settings: Res<SettingsResource>,
    asset_server: Res<AssetServer>,
    mut keyboard_focus: ResMut<KeyboardFocusIndex>,
) {
    keyboard_focus.0 = 0;

    let font: Handle<Font> = asset_server.load(FONT_JP);
    let lang = settings.language;

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
            DespawnOnExit(AppState::Title),
        ))
        .with_children(|parent| {
            // Game title
            parent.spawn((
                Text::new(t("game_title", lang)),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_HUGE,
                    ..default()
                },
                TextColor(PRIMARY_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(100.0)),
                    ..default()
                },
            ));

            // Start button (index 0 — receives initial BUTTON_HOVER color)
            spawn_button(
                parent,
                t("btn_start", lang),
                ButtonAction::StartGame,
                0,
                FONT_SIZE_LARGE,
                BUTTON_LARGE_WIDTH,
                BUTTON_LARGE_HEIGHT,
                font.clone(),
            );

            // Settings button (index 1)
            spawn_button(
                parent,
                t("btn_settings", lang),
                ButtonAction::OpenSettings,
                1,
                FONT_SIZE_LARGE,
                BUTTON_LARGE_WIDTH,
                BUTTON_LARGE_HEIGHT,
                font.clone(),
            );

            // How-to-play button (index 2)
            spawn_button(
                parent,
                t("btn_how_to_play", lang),
                ButtonAction::OpenHowToPlay,
                2,
                FONT_SIZE_LARGE,
                BUTTON_LARGE_WIDTH,
                BUTTON_LARGE_HEIGHT,
                font.clone(),
            );

            // Quit button (index 3)
            spawn_button(
                parent,
                t("btn_quit", lang),
                ButtonAction::QuitGame,
                3,
                FONT_SIZE_LARGE,
                BUTTON_LARGE_WIDTH,
                BUTTON_LARGE_HEIGHT,
                font.clone(),
            );

            // Highscore display
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
                    margin: UiRect::top(Val::Px(150.0)),
                    ..default()
                },
            ));
        });
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Formats an integer with comma separators every three digits.
///
/// # Examples
///
/// ```
/// # use suika_game_ui::screens::title::format_score;
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
