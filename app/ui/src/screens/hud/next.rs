//! Next-fruit widget.
//!
//! Renders a "ネクスト" label with a coloured circle beneath it that mirrors
//! the [`NextFruitType`] resource.  Both the label and the preview live inside
//! a single UI column, so they always stay together regardless of layout
//! changes in [`super::setup_hud`].
//!
//! The preview circle is hidden while no active (held or falling) fruit exists,
//! matching the original game's behaviour.
//!
//! # Usage
//!
//! ```ignore
//! parent_anchor.with_children(|p| next::spawn_next_widget(p, &font, &cfg));
//! app.add_systems(Update, next::update_next.run_if(in_state(AppState::Playing)));
//! ```

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use suika_game_core::prelude::{Fruit, FruitSpawnState, NextFruitType};
use suika_game_core::resources::settings::Language;

use crate::config::NextHudConfig;
use crate::i18n::t;
use crate::styles::{FONT_SIZE_SMALL, TEXT_COLOR};

// ---------------------------------------------------------------------------
// Marker component
// ---------------------------------------------------------------------------

/// Marks the UI node used as the next-fruit preview circle.
#[derive(Component, Debug)]
pub struct HudNextPreview;

// ---------------------------------------------------------------------------
// Spawn helper
// ---------------------------------------------------------------------------

/// Spawns the next-fruit widget as children of `parent`.
///
/// The preview circle diameter comes from `cfg.preview_size`.
///
/// Layout (column, center-aligned):
///
/// ```text
/// ネクスト              ← FONT_SIZE_SMALL, TEXT_COLOR
/// ┌──────────┐
/// │  [color] │         ← preview_size × preview_size circle, HudNextPreview
/// └──────────┘
/// ```
pub fn spawn_next_widget(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    cfg: &NextHudConfig,
    lang: Language,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|col| {
            // Label
            col.spawn((
                Text::new(t("hud_next", lang)),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_SMALL,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            // Preview circle
            col.spawn((
                Node {
                    width: Val::Px(cfg.preview_size),
                    height: Val::Px(cfg.preview_size),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                BorderRadius::all(Val::Percent(50.0)),
                Visibility::Hidden,
                HudNextPreview,
            ));
        });
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Updates the next-fruit preview circle every frame.
///
/// - **Colour**: refreshed whenever [`NextFruitType`] changes.
/// - **Visibility**: shown while a held or falling fruit exists; hidden
///   otherwise (same rule as the old world-space preview).
pub fn update_next(
    next_fruit: Res<NextFruitType>,
    fruit_states: Query<&FruitSpawnState, With<Fruit>>,
    mut preview_q: Query<(&mut BackgroundColor, &mut Visibility), With<HudNextPreview>>,
) {
    let has_active = fruit_states
        .iter()
        .any(|s| *s == FruitSpawnState::Held || *s == FruitSpawnState::Falling);

    for (mut bg, mut vis) in preview_q.iter_mut() {
        let desired = if has_active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *vis != desired {
            *vis = desired;
        }

        // Always write the correct colour so that newly-spawned preview
        // widgets (e.g. after HUD rebuild on pause/resume) get the right
        // colour even when NextFruitType itself has not changed this frame.
        *bg = BackgroundColor(next_fruit.get().placeholder_color());
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_next_preview_marker_exists() {
        let _n = HudNextPreview;
    }

    #[test]
    fn test_default_preview_size_is_positive() {
        assert!(crate::config::NextHudConfig::default().preview_size > 0.0);
    }
}
