//! UI SFX: button hover and click sounds.

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use suika_game_ui::components::{KeyboardFocusIndex, MenuButton};

use crate::channels::SfxChannel;
use crate::config::{AudioConfig, AudioConfigHandle};
use crate::handles::SfxHandles;

/// Plays sound effects in response to button hover and click interactions.
///
/// Queries every [`MenuButton`] entity whose [`Interaction`] component changed
/// this frame (Bevy change-detection) and plays the appropriate clip:
///
/// - [`Interaction::Hovered`] → `button_hover.wav`
/// - [`Interaction::Pressed`] → `button_click.wav`
pub fn play_ui_sfx(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<MenuButton>)>,
    sfx_channel: Res<AudioChannel<SfxChannel>>,
    sfx_handles: Option<Res<SfxHandles>>,
    audio_config_handle: Option<Res<AudioConfigHandle>>,
    audio_config_assets: Res<Assets<AudioConfig>>,
) {
    let Some(sfx_handles) = sfx_handles else {
        return;
    };

    let default_cfg = AudioConfig::default();
    let cfg = audio_config_handle
        .as_ref()
        .and_then(|h| audio_config_assets.get(&h.0))
        .unwrap_or(&default_cfg);

    for interaction in interaction_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                sfx_channel
                    .play(sfx_handles.button_click.clone())
                    .with_volume(cfg.sfx_button_click_volume);
            }
            Interaction::Hovered => {
                sfx_channel
                    .play(sfx_handles.button_hover.clone())
                    .with_volume(cfg.sfx_button_hover_volume);
            }
            Interaction::None => {}
        }
    }
}

/// Plays sound effects in response to keyboard menu navigation.
///
/// This system is the keyboard counterpart to [`play_ui_sfx`].  Bevy's
/// [`Interaction`] component is only updated by pointer devices, so
/// W / S / Arrow keys and Enter must be handled independently.
///
/// - W / S / Up / Down → `button_hover.wav`, **only when the focused button
///   index actually changes**.
/// - Enter → `button_click.wav` (confirms the currently focused button).
#[allow(clippy::too_many_arguments)]
pub fn play_keyboard_ui_sfx(
    keyboard: Res<ButtonInput<KeyCode>>,
    button_query: Query<(), With<MenuButton>>,
    focus: Option<Res<KeyboardFocusIndex>>,
    mut prev_focus: Local<Option<usize>>,
    sfx_channel: Res<AudioChannel<SfxChannel>>,
    sfx_handles: Option<Res<SfxHandles>>,
    audio_config_handle: Option<Res<AudioConfigHandle>>,
    audio_config_assets: Res<Assets<AudioConfig>>,
) {
    // No menu buttons on screen — reset tracking and bail.
    if button_query.is_empty() {
        *prev_focus = None;
        return;
    }

    let current = focus.as_ref().map(|r| r.0).unwrap_or(0);

    let Some(sfx_handles) = sfx_handles else {
        *prev_focus = Some(current);
        return;
    };

    let default_cfg = AudioConfig::default();
    let cfg = audio_config_handle
        .as_ref()
        .and_then(|h| audio_config_assets.get(&h.0))
        .unwrap_or(&default_cfg);

    // Hover sound: only when the focus index actually moved.
    let old = prev_focus.replace(current);
    if old.is_some_and(|p| p != current) {
        sfx_channel
            .play(sfx_handles.button_hover.clone())
            .with_volume(cfg.sfx_button_hover_volume);
    }

    // Confirm key → click sound.
    if keyboard.just_pressed(KeyCode::Enter) {
        sfx_channel
            .play(sfx_handles.button_click.clone())
            .with_volume(cfg.sfx_button_click_volume);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_sfx_volumes_are_audible_and_quiet() {
        let cfg = AudioConfig::default();
        assert!(
            cfg.sfx_button_click_volume <= 0.0,
            "button click volume should be ≤ 0 dB (quiet)"
        );
        assert!(
            cfg.sfx_button_click_volume > -30.0,
            "button click volume should be > -30 dB (audible)"
        );
        assert!(
            cfg.sfx_button_hover_volume <= 0.0,
            "button hover volume should be ≤ 0 dB (quiet)"
        );
        assert!(
            cfg.sfx_button_hover_volume > -30.0,
            "button hover volume should be > -30 dB (audible)"
        );
    }
}
