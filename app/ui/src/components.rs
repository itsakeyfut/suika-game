//! Reusable UI components and spawn helpers.
//!
//! Provides the [`MenuButton`] component, the [`ButtonAction`] enum, and
//! helper functions for spawning styled buttons and text nodes so that every
//! screen can build its layout from the same building blocks.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;

use crate::styles::{BUTTON_NORMAL, FONT_SIZE_MEDIUM, TEXT_COLOR};

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker component attached to every interactive menu button.
///
/// Stores the [`ButtonAction`] that should be executed when the button is
/// clicked so that a single interaction system can handle all button presses.
#[derive(Component, Debug, Clone)]
pub struct MenuButton {
    /// The action triggered when this button is activated.
    pub action: ButtonAction,
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Identifies the intended behavior of a [`MenuButton`].
///
/// A single button-interaction system matches on this enum to perform the
/// correct state transition or game action without coupling individual screens
/// to the input handling logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonAction {
    /// Transition from Title to Playing — starts a fresh game.
    StartGame,
    /// Transition from GameOver back to Playing — restarts the game.
    RetryGame,
    /// Transition from GameOver or Paused back to Title.
    GoToTitle,
    /// Transition from Paused back to Playing — resumes the current game.
    ResumeGame,
}

// ---------------------------------------------------------------------------
// Spawn helpers
// ---------------------------------------------------------------------------

/// Spawns a styled [`MenuButton`] as a child of `parent`.
///
/// The button is a Bevy [`Button`] node with a centered text label.  Colors
/// are taken from [`crate::styles`]; interaction state changes (hover /
/// pressed) are handled by the button-interaction system in the playing or
/// menu screen modules.
///
/// # Arguments
///
/// * `parent`    – the [`ChildBuilder`] to attach the new node to
/// * `text`      – label displayed inside the button
/// * `action`    – [`ButtonAction`] fired when the button is clicked
/// * `font_size` – text size in logical pixels (use `FONT_SIZE_*` constants)
/// * `width`     – button width in logical pixels (use `BUTTON_*_WIDTH` constants)
/// * `height`    – button height in logical pixels (use `BUTTON_*_HEIGHT` constants)
pub fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: ButtonAction,
    font_size: f32,
    width: f32,
    height: f32,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(width),
                height: Val::Px(height),
                margin: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BUTTON_NORMAL),
            MenuButton { action },
        ))
        .with_children(|btn: &mut ChildSpawnerCommands| {
            btn.spawn((
                Text::new(text),
                TextFont {
                    font_size,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        });
}

/// Spawns a plain text node as a child of `parent`.
///
/// Use this for labels, score readouts, and any other non-interactive text.
///
/// # Arguments
///
/// * `parent`    – the [`ChildBuilder`] to attach the new node to
/// * `text`      – string to display
/// * `font_size` – text size in logical pixels (use `FONT_SIZE_*` constants)
/// * `color`     – [`Color`] applied to the text (use palette constants from [`crate::styles`])
pub fn spawn_text(parent: &mut ChildSpawnerCommands, text: &str, font_size: f32, color: Color) {
    parent.spawn((
        Text::new(text),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
    ));
}

/// Spawns a default-sized [`MenuButton`] using [`FONT_SIZE_MEDIUM`] and the
/// large button dimensions from [`crate::styles`].
///
/// Convenience wrapper around [`spawn_button`] for the most common case.
pub fn spawn_menu_button(parent: &mut ChildSpawnerCommands, text: &str, action: ButtonAction) {
    use crate::styles::{BUTTON_LARGE_HEIGHT, BUTTON_LARGE_WIDTH};
    spawn_button(
        parent,
        text,
        action,
        FONT_SIZE_MEDIUM,
        BUTTON_LARGE_WIDTH,
        BUTTON_LARGE_HEIGHT,
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_action_equality() {
        assert_eq!(ButtonAction::StartGame, ButtonAction::StartGame);
        assert_ne!(ButtonAction::StartGame, ButtonAction::RetryGame);
        assert_ne!(ButtonAction::GoToTitle, ButtonAction::ResumeGame);
    }

    #[test]
    fn test_button_action_all_variants_copy() {
        // ButtonAction is Copy — verify cloning produces an equal value.
        let a = ButtonAction::RetryGame;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_menu_button_stores_action() {
        let btn = MenuButton {
            action: ButtonAction::StartGame,
        };
        assert_eq!(btn.action, ButtonAction::StartGame);
    }

    #[test]
    fn test_menu_button_clone() {
        let original = MenuButton {
            action: ButtonAction::GoToTitle,
        };
        let cloned = original.clone();
        assert_eq!(original.action, cloned.action);
    }
}
