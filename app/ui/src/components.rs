//! Reusable UI components and spawn helpers.
//!
//! Provides the [`MenuButton`] component, the [`ButtonAction`] enum, and
//! helper functions for spawning styled buttons and text nodes so that every
//! screen can build its layout from the same building blocks.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use suika_game_core::prelude::AppState;

use crate::styles::{BUTTON_HOVER, BUTTON_NORMAL, BUTTON_PRESSED, FONT_SIZE_MEDIUM, TEXT_COLOR};

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

/// Sequential index used for keyboard navigation between [`MenuButton`]s.
///
/// Assign incrementing values (0, 1, 2, …) to buttons in top-to-bottom order.
/// [`handle_keyboard_menu_navigation`] uses this component to move focus up
/// and down with the W / S / Arrow keys.
#[derive(Component, Debug, Clone, Copy)]
pub struct ButtonIndex(pub usize);

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Tracks which [`MenuButton`] currently has keyboard focus.
///
/// The value corresponds to the [`ButtonIndex`] of the focused button.
/// Defaults to `0` (first button).
#[derive(Resource, Debug, Default)]
pub struct KeyboardFocusIndex(pub usize);

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
    /// Open the settings screen (not yet implemented).
    OpenSettings,
}

// ---------------------------------------------------------------------------
// Spawn helpers
// ---------------------------------------------------------------------------

/// Spawns a styled [`MenuButton`] as a child of `parent`.
///
/// The button is a Bevy [`Button`] node with a centered text label. The button
/// at `index == 0` starts with [`BUTTON_HOVER`] color to indicate initial
/// keyboard focus; all others start with [`BUTTON_NORMAL`].
///
/// # Arguments
///
/// * `parent`    – the [`ChildBuilder`] to attach the new node to
/// * `text`      – label displayed inside the button
/// * `action`    – [`ButtonAction`] fired when the button is clicked
/// * `index`     – keyboard-navigation order (0 = first / top button)
/// * `font_size` – text size in logical pixels (use `FONT_SIZE_*` constants)
/// * `width`     – button width in logical pixels (use `BUTTON_*_WIDTH` constants)
/// * `height`    – button height in logical pixels (use `BUTTON_*_HEIGHT` constants)
/// * `font`      – font asset handle; pass `Handle::default()` to use Bevy's built-in font
#[allow(clippy::too_many_arguments)]
pub fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: ButtonAction,
    index: usize,
    font_size: f32,
    width: f32,
    height: f32,
    font: Handle<Font>,
) {
    let initial_color = if index == 0 {
        BUTTON_HOVER
    } else {
        BUTTON_NORMAL
    };

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
            BackgroundColor(initial_color),
            MenuButton { action },
            ButtonIndex(index),
        ))
        .with_children(|btn: &mut ChildSpawnerCommands| {
            btn.spawn((
                Text::new(text),
                TextFont {
                    font: font.clone(),
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
/// * `font`      – font asset handle; pass `Handle::default()` to use Bevy's built-in font
pub fn spawn_text(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font_size: f32,
    color: Color,
    font: Handle<Font>,
) {
    parent.spawn((
        Text::new(text),
        TextFont {
            font,
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
pub fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: ButtonAction,
    index: usize,
    font: Handle<Font>,
) {
    use crate::styles::{BUTTON_LARGE_HEIGHT, BUTTON_LARGE_WIDTH};
    spawn_button(
        parent,
        text,
        action,
        index,
        FONT_SIZE_MEDIUM,
        BUTTON_LARGE_WIDTH,
        BUTTON_LARGE_HEIGHT,
        font,
    );
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Handles mouse/touch interaction with [`MenuButton`] entities.
///
/// Changes the button background color on hover/press and triggers the
/// appropriate [`AppState`] transition when a button is clicked.
pub fn handle_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, button, mut bg) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(BUTTON_PRESSED);
                match button.action {
                    ButtonAction::StartGame | ButtonAction::RetryGame => {
                        next_state.set(AppState::Playing);
                    }
                    ButtonAction::GoToTitle => {
                        next_state.set(AppState::Title);
                    }
                    ButtonAction::ResumeGame => {
                        next_state.set(AppState::Playing);
                    }
                    ButtonAction::OpenSettings => {
                        // Settings screen not yet implemented.
                    }
                }
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(BUTTON_HOVER);
            }
            Interaction::None => {
                *bg = BackgroundColor(BUTTON_NORMAL);
            }
        }
    }
}

/// Moves keyboard focus between [`MenuButton`]s using W / Up (up) and S / Down (down),
/// and confirms the focused button with Enter.
///
/// Updates [`KeyboardFocusIndex`] and reflects the change immediately by
/// recoloring all buttons: the focused one gets [`BUTTON_HOVER`], the rest
/// get [`BUTTON_NORMAL`].
pub fn handle_keyboard_menu_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut focus: ResMut<KeyboardFocusIndex>,
    mut button_query: Query<(&ButtonIndex, &MenuButton, &mut BackgroundColor)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let count = button_query.iter().count();
    if count == 0 {
        return;
    }

    let prev = focus.0;

    if keyboard.just_pressed(KeyCode::KeyW) || keyboard.just_pressed(KeyCode::ArrowUp) {
        focus.0 = focus.0.saturating_sub(1);
    }
    if keyboard.just_pressed(KeyCode::KeyS) || keyboard.just_pressed(KeyCode::ArrowDown) {
        focus.0 = (focus.0 + 1).min(count - 1);
    }

    if focus.0 != prev {
        for (idx, _, mut bg) in button_query.iter_mut() {
            *bg = BackgroundColor(if idx.0 == focus.0 {
                BUTTON_HOVER
            } else {
                BUTTON_NORMAL
            });
        }
    }

    if keyboard.just_pressed(KeyCode::Enter)
        && let Some((_, button, _)) = button_query.iter().find(|(idx, _, _)| idx.0 == focus.0)
    {
        match button.action {
            ButtonAction::StartGame | ButtonAction::RetryGame => {
                next_state.set(AppState::Playing);
            }
            ButtonAction::GoToTitle => {
                next_state.set(AppState::Title);
            }
            ButtonAction::ResumeGame => {
                next_state.set(AppState::Playing);
            }
            ButtonAction::OpenSettings => {
                // Settings screen not yet implemented.
            }
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
    fn test_button_action_equality() {
        assert_eq!(ButtonAction::StartGame, ButtonAction::StartGame);
        assert_ne!(ButtonAction::StartGame, ButtonAction::RetryGame);
        assert_ne!(ButtonAction::GoToTitle, ButtonAction::ResumeGame);
        assert_ne!(ButtonAction::OpenSettings, ButtonAction::StartGame);
    }

    #[test]
    fn test_button_action_all_variants_copy() {
        // ButtonAction is Copy — verify copying produces an equal value.
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

    #[test]
    fn test_button_index_copy() {
        let a = ButtonIndex(2);
        let b = a;
        assert_eq!(a.0, b.0);
    }

    #[test]
    fn test_keyboard_focus_index_default() {
        let focus = KeyboardFocusIndex::default();
        assert_eq!(focus.0, 0);
    }
}
