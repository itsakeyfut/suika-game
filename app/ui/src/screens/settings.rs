//! Settings screen — shown when the player taps the 設定 / Settings button.
//!
//! Displays four configurable rows:
//!
//! ```text
//!          設定 / Settings
//!
//!  BGM音量    ◀  ■■■■■■■■□□  ▶   80%
//!  SE音量     ◀  ■■■■■■■■□□  ▶   80%
//!  エフェクト  [         ON        ]
//!  言語        ◀  [ 日本語 ]   ▶
//!
//!           [ もどる ]
//! ```
//!
//! Volume rows use ◀ / ▶ arrow buttons to step the value up or down.
//! The effects row uses a single wide toggle button that cycles ON ↔ OFF.
//! The language row uses ◀ / ▶ to cycle through available languages.
//!
//! Every button press immediately mutates [`SettingsResource`] and persists the
//! change to `save/settings.json`.  [`update_settings_display`] runs every
//! frame while in this state and updates the value text nodes whenever the
//! resource is marked changed.
//!
//! All entities are tagged with [`DespawnOnExit`]`(`[`AppState::Settings`]`)`
//! so Bevy cleans them up automatically on state exit.

use bevy::prelude::*;
use suika_game_core::prelude::AppState;
use suika_game_core::resources::settings::{Language, SettingsResource};

use crate::components::{ButtonAction, ButtonIndex, KeyboardFocusIndex, MenuButton, spawn_button};
use crate::i18n::t;
use crate::styles::{
    BG_COLOR, BUTTON_LARGE_HEIGHT, BUTTON_LARGE_WIDTH, BUTTON_NORMAL, FONT_JP, FONT_SIZE_LARGE,
    FONT_SIZE_MEDIUM, FONT_SIZE_SMALL, FONT_SYMBOL, PRIMARY_COLOR, TEXT_COLOR,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const LABEL_WIDTH: f32 = 160.0;
const GAUGE_WIDTH: f32 = 240.0;
const SMALL_BTN_SIZE: f32 = 48.0;
const ROW_GAP: f32 = 20.0;
/// Column gap between items within a row (label ↔ ◀ ↔ gauge ↔ ▶).
const COL_GAP: f32 = 12.0;
/// Button margin applied on all sides by [`spawn_button`] / [`spawn_arrow_button`].
const BTN_MARGIN: f32 = 10.0;
/// Total width of every settings row so the outer column can centre them consistently.
///
/// Computed as:
/// `LABEL_WIDTH + COL_GAP + (SMALL_BTN_SIZE + 2×BTN_MARGIN) + COL_GAP + GAUGE_WIDTH + COL_GAP + (SMALL_BTN_SIZE + 2×BTN_MARGIN)`
const ROW_WIDTH: f32 = LABEL_WIDTH
    + COL_GAP
    + (SMALL_BTN_SIZE + BTN_MARGIN * 2.0)
    + COL_GAP
    + GAUGE_WIDTH
    + COL_GAP
    + (SMALL_BTN_SIZE + BTN_MARGIN * 2.0);
/// Width of a boolean toggle button so it occupies the same horizontal space
/// as the ◀ + value-text + ▶ triplet in a setting row.
const TOGGLE_BTN_WIDTH: f32 = ROW_WIDTH - LABEL_WIDTH - COL_GAP - BTN_MARGIN * 2.0;

// ---------------------------------------------------------------------------
// Marker components (local — not exported)
// ---------------------------------------------------------------------------

/// Marks the text node that shows the BGM volume gauge.
#[derive(Component)]
pub struct BgmGaugeText;

/// Marks the text node that shows the SFX volume gauge.
#[derive(Component)]
pub struct SfxGaugeText;

/// Marks the text node that shows the current effects on/off value.
#[derive(Component)]
pub struct EffectsValueText;

/// Marks the text node that shows the current language selection.
#[derive(Component)]
pub struct LanguageValueText;

/// Marks any text node whose content should be refreshed via [`crate::i18n::t`]
/// whenever the language setting changes.
///
/// The stored key is the same key passed to [`crate::i18n::t`].
/// [`update_translatable_texts`] queries all entities with this component and
/// re-sets their text to the localised string for the current language.
#[derive(Component)]
pub struct TranslatableText(pub &'static str);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Builds the gauge string: filled blocks + empty blocks + percentage.
///
/// Example for `vol = 8`: `"■■■■■■■■□□  80%"`.
fn gauge_string(vol: u8) -> String {
    let filled = vol as usize;
    let empty = 10 - filled;
    format!(
        "{}{}  {:3}%",
        "■".repeat(filled),
        "□".repeat(empty),
        vol * 10
    )
}

/// Spawns a small ◀ or ▶ button as a child of `parent`.
fn spawn_arrow_button(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    label: &str,
    action: ButtonAction,
    index: usize,
    font: Handle<Font>,
) {
    spawn_button(
        parent,
        label,
        action,
        index,
        FONT_SIZE_MEDIUM,
        SMALL_BTN_SIZE,
        SMALL_BTN_SIZE,
        font,
    );
}

/// Spawns a single settings row (label + ◀ + value text + ▶).
///
/// `font` is used for the label and value text; `symbol_font` is used for the
/// ◀ / ▶ arrow buttons so that a font with broader Unicode coverage can be
/// used independently of the main pixel font.
#[allow(clippy::too_many_arguments)]
fn spawn_setting_row<M: Component>(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    label: &str,
    label_key: &'static str,
    value: &str,
    marker: M,
    left_action: ButtonAction,
    right_action: ButtonAction,
    left_index: usize,
    right_index: usize,
    font: Handle<Font>,
    symbol_font: Handle<Font>,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            width: Val::Px(ROW_WIDTH),
            margin: UiRect::vertical(Val::Px(ROW_GAP / 2.0)),
            column_gap: Val::Px(COL_GAP),
            ..default()
        })
        .with_children(|row| {
            // Label — tagged with TranslatableText so it updates on language change.
            row.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_SMALL,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    width: Val::Px(LABEL_WIDTH),
                    ..default()
                },
                TranslatableText(label_key),
            ));

            // ◀ button — uses symbol_font for the triangle glyph.
            spawn_arrow_button(row, "◀", left_action, left_index, symbol_font.clone());

            // Value / gauge text
            row.spawn((
                Text::new(value),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_SMALL,
                    ..default()
                },
                TextColor(PRIMARY_COLOR),
                Node {
                    width: Val::Px(GAUGE_WIDTH),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                marker,
            ));

            // ▶ button — uses symbol_font for the triangle glyph.
            spawn_arrow_button(row, "▶", right_action, right_index, symbol_font.clone());
        });
}

/// Spawns a boolean toggle row: label on the left, single wide toggle button on the right.
///
/// The toggle button spans the same horizontal area as the ◀ + value + ▶ triplet
/// used by [`spawn_setting_row`], so all rows stay visually aligned.
/// The button text entity receives `value_marker` so [`update_settings_display`]
/// can update it when the value changes.
#[allow(clippy::too_many_arguments)]
fn spawn_toggle_row<M: Component>(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    label: &str,
    label_key: &'static str,
    value: &str,
    value_marker: M,
    action: ButtonAction,
    index: usize,
    font: Handle<Font>,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            width: Val::Px(ROW_WIDTH),
            margin: UiRect::vertical(Val::Px(ROW_GAP / 2.0)),
            column_gap: Val::Px(COL_GAP),
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_SMALL,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    width: Val::Px(LABEL_WIDTH),
                    ..default()
                },
                TranslatableText(label_key),
            ));

            // Single toggle button — wide enough to fill ◀ + value + ▶ space.
            row.spawn((
                Button,
                Node {
                    width: Val::Px(TOGGLE_BTN_WIDTH),
                    height: Val::Px(SMALL_BTN_SIZE),
                    margin: UiRect::all(Val::Px(BTN_MARGIN)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(BUTTON_NORMAL),
                MenuButton { action },
                ButtonIndex(index),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(value),
                    TextFont {
                        font: font.clone(),
                        font_size: FONT_SIZE_SMALL,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    value_marker,
                ));
            });
        });
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the settings screen UI when entering [`AppState::Settings`].
pub fn setup_settings_screen(
    mut commands: Commands,
    settings: Res<SettingsResource>,
    asset_server: Res<AssetServer>,
    mut keyboard_focus: ResMut<KeyboardFocusIndex>,
) {
    keyboard_focus.0 = 0;

    let font: Handle<Font> = asset_server.load(FONT_JP);
    let symbol_font: Handle<Font> = asset_server.load(FONT_SYMBOL);
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
            DespawnOnExit(AppState::Settings),
        ))
        .with_children(|parent| {
            // Title — tagged so it updates when language changes.
            parent.spawn((
                Text::new(t("settings_title", lang)),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_LARGE,
                    ..default()
                },
                TextColor(PRIMARY_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
                TranslatableText("settings_title"),
            ));

            // BGM Volume row (arrow buttons: index 0 ◀, index 1 ▶)
            spawn_setting_row(
                parent,
                t("label_bgm", lang),
                "label_bgm",
                &gauge_string(settings.bgm_volume),
                BgmGaugeText,
                ButtonAction::BgmVolumeDown,
                ButtonAction::BgmVolumeUp,
                0,
                1,
                font.clone(),
                symbol_font.clone(),
            );

            // SFX Volume row (arrow buttons: index 2 ◀, index 3 ▶)
            spawn_setting_row(
                parent,
                t("label_sfx", lang),
                "label_sfx",
                &gauge_string(settings.sfx_volume),
                SfxGaugeText,
                ButtonAction::SfxVolumeDown,
                ButtonAction::SfxVolumeUp,
                2,
                3,
                font.clone(),
                symbol_font.clone(),
            );

            // Effects row — single toggle button (index 4); bool needs no arrows.
            let effects_val = if settings.effects_enabled {
                t("value_on", lang)
            } else {
                t("value_off", lang)
            };
            spawn_toggle_row(
                parent,
                t("label_effects", lang),
                "label_effects",
                effects_val,
                EffectsValueText,
                ButtonAction::ToggleEffects,
                4,
                font.clone(),
            );

            // Language row (arrow buttons: index 5 ◀, index 6 ▶)
            let lang_val = match settings.language {
                Language::Japanese => t("lang_japanese", lang),
                Language::English => t("lang_english", lang),
            };
            spawn_setting_row(
                parent,
                t("label_language", lang),
                "label_language",
                lang_val,
                LanguageValueText,
                ButtonAction::ToggleLanguage,
                ButtonAction::ToggleLanguage,
                5,
                6,
                font.clone(),
                symbol_font.clone(),
            );

            // Back button (index 7) — inlined to tag the text with TranslatableText.
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(BUTTON_LARGE_WIDTH),
                        height: Val::Px(BUTTON_LARGE_HEIGHT),
                        margin: UiRect::all(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(BUTTON_NORMAL),
                    MenuButton {
                        action: ButtonAction::BackToTitle,
                    },
                    ButtonIndex(7),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(t("btn_back", lang)),
                        TextFont {
                            font: font.clone(),
                            font_size: FONT_SIZE_MEDIUM,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        TranslatableText("btn_back"),
                    ));
                });
        });
}

/// Updates the value text nodes whenever [`SettingsResource`] changes.
///
/// Runs every frame while in [`AppState::Settings`], but only performs work
/// on frames where the resource was actually modified (via `is_changed()`).
#[allow(clippy::type_complexity)]
pub fn update_settings_display(
    settings: Res<SettingsResource>,
    mut bgm_q: Query<&mut Text, (With<BgmGaugeText>, Without<SfxGaugeText>)>,
    mut sfx_q: Query<&mut Text, (With<SfxGaugeText>, Without<BgmGaugeText>)>,
    mut effects_q: Query<
        &mut Text,
        (
            With<EffectsValueText>,
            Without<BgmGaugeText>,
            Without<SfxGaugeText>,
            Without<LanguageValueText>,
        ),
    >,
    mut lang_q: Query<
        &mut Text,
        (
            With<LanguageValueText>,
            Without<BgmGaugeText>,
            Without<SfxGaugeText>,
            Without<EffectsValueText>,
        ),
    >,
) {
    if !settings.is_changed() {
        return;
    }

    let lang = settings.language;

    for mut text in bgm_q.iter_mut() {
        text.0 = gauge_string(settings.bgm_volume);
    }
    for mut text in sfx_q.iter_mut() {
        text.0 = gauge_string(settings.sfx_volume);
    }
    for mut text in effects_q.iter_mut() {
        text.0 = if settings.effects_enabled {
            t("value_on", lang).to_string()
        } else {
            t("value_off", lang).to_string()
        };
    }
    for mut text in lang_q.iter_mut() {
        text.0 = match settings.language {
            Language::Japanese => t("lang_japanese", lang).to_string(),
            Language::English => t("lang_english", lang).to_string(),
        };
    }
}

/// Updates all [`TranslatableText`] nodes whenever [`SettingsResource`] changes.
///
/// Queries every text entity tagged with [`TranslatableText`] (the settings
/// title, row labels, and the Back button) and re-sets the text to the
/// localised string for the current language.
///
/// This system runs alongside [`update_settings_display`] while in
/// [`AppState::Settings`], so all static text refreshes on the same frame
/// that the user toggles the language.
pub fn update_translatable_texts(
    settings: Res<SettingsResource>,
    mut query: Query<(&mut Text, &TranslatableText)>,
) {
    if !settings.is_changed() {
        return;
    }
    let lang = settings.language;
    for (mut text, key) in query.iter_mut() {
        text.0 = t(key.0, lang).to_string();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gauge_string_vol_10() {
        let s = gauge_string(10);
        assert!(s.starts_with("■■■■■■■■■■"));
        assert!(s.contains("100%"));
    }

    #[test]
    fn test_gauge_string_vol_0() {
        let s = gauge_string(0);
        assert!(s.starts_with("□□□□□□□□□□"));
        assert!(s.contains("  0%"));
    }

    #[test]
    fn test_gauge_string_vol_8() {
        let s = gauge_string(8);
        // 8 filled, 2 empty
        let filled: String = "■".repeat(8);
        let empty: String = "□".repeat(2);
        assert!(s.starts_with(&filled));
        assert!(s.contains(&empty));
        assert!(s.contains("80%"));
    }

    #[test]
    fn test_gauge_string_length_consistent() {
        // All volumes should produce strings of the same "block" length
        for v in 0u8..=10 {
            let s = gauge_string(v);
            let filled = "■".repeat(v as usize);
            let empty = "□".repeat(10 - v as usize);
            assert!(
                s.contains(&filled) || v == 0,
                "vol {v}: filled blocks missing"
            );
            assert!(
                s.contains(&empty) || v == 10,
                "vol {v}: empty blocks missing"
            );
        }
    }
}
