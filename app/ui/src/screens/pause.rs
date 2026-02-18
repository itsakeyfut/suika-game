//! Pause menu — shown as a semi-transparent overlay when the player presses
//! ESC during active gameplay.
//!
//! Spawns a full-screen overlay containing:
//! - A **PAUSED** heading
//! - A **Resume** button (→ [`AppState::Playing`])
//! - A **Back to Title** button (→ [`AppState::Title`])
//!
//! All entities are tagged with [`DespawnOnExit`]`(`[`AppState::Paused`]`)` so
//! Bevy automatically despawns the menu when the state transitions away from
//! `Paused`.
//!
//! ## Pause toggle
//!
//! [`toggle_pause`] listens for the ESC key in both [`AppState::Playing`] and
//! [`AppState::Paused`] and toggles between them.  It is registered uncondi-
//! tionally in [`GameUIPlugin`] so the same system handles both directions.

use bevy::prelude::*;
use suika_game_core::prelude::AppState;

use crate::components::{ButtonAction, KeyboardFocusIndex, spawn_button};
use crate::styles::{
    BUTTON_LARGE_HEIGHT, BUTTON_LARGE_WIDTH, BUTTON_MEDIUM_HEIGHT, BUTTON_MEDIUM_WIDTH,
    FONT_JP, FONT_SIZE_LARGE, FONT_SIZE_MEDIUM,
};

// ---------------------------------------------------------------------------
// Color constants (local to this screen)
// ---------------------------------------------------------------------------

/// Semi-transparent dark overlay — dims the game scene while paused.
const OVERLAY_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.70);

/// White text used for the "PAUSED" heading.
const PAUSED_TEXT_COLOR: Color = Color::WHITE;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the pause menu overlay when entering [`AppState::Paused`].
///
/// Creates an absolute-positioned, full-screen semi-transparent panel with
/// a "PAUSED" heading and two buttons.  Resets [`KeyboardFocusIndex`] to `0`
/// so the Resume button always receives initial keyboard focus.
pub fn setup_pause_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut keyboard_focus: ResMut<KeyboardFocusIndex>,
) {
    keyboard_focus.0 = 0;

    let font: Handle<Font> = asset_server.load(FONT_JP);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(OVERLAY_COLOR),
            DespawnOnExit(AppState::Paused),
        ))
        .with_children(|parent| {
            // "PAUSED" heading
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_LARGE,
                    ..default()
                },
                TextColor(PAUSED_TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // Resume button (index 0 — initial keyboard focus)
            spawn_button(
                parent,
                "再開",
                ButtonAction::ResumeGame,
                0,
                FONT_SIZE_LARGE,
                BUTTON_LARGE_WIDTH,
                BUTTON_LARGE_HEIGHT,
                font.clone(),
            );

            // Back-to-title button (index 1)
            spawn_button(
                parent,
                "タイトルへ",
                ButtonAction::GoToTitle,
                1,
                FONT_SIZE_MEDIUM,
                BUTTON_MEDIUM_WIDTH,
                BUTTON_MEDIUM_HEIGHT,
                font.clone(),
            );
        });
}

/// Toggles between [`AppState::Playing`] and [`AppState::Paused`] on ESC.
///
/// Runs every frame regardless of the current state (registered without a
/// `run_if` filter).  Only acts in the two states where the toggle makes
/// sense; all other states are silently ignored.
pub fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            AppState::Playing => {
                next_state.set(AppState::Paused);
            }
            AppState::Paused => {
                next_state.set(AppState::Playing);
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
    fn test_overlay_color_is_semi_transparent() {
        let a = OVERLAY_COLOR.to_srgba().alpha;
        assert!(a > 0.0 && a < 1.0, "overlay should be semi-transparent, got alpha {a}");
    }

    #[test]
    fn test_paused_text_color_is_opaque() {
        let a = PAUSED_TEXT_COLOR.to_srgba().alpha;
        assert!((a - 1.0).abs() < f32::EPSILON, "paused text should be fully opaque");
    }
}
