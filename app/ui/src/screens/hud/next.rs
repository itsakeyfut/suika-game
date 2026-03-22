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
use suika_game_core::prelude::{Fruit, FruitSpawnState, FruitSprites, NextFruitType};
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

            // Preview circle / sprite
            col.spawn((
                Node {
                    width: Val::Px(cfg.preview_size),
                    height: Val::Px(cfg.preview_size),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                BorderRadius::all(Val::Percent(50.0)),
                ImageNode::default(),
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
/// - **Sprite / colour**: refreshed whenever [`NextFruitType`] or [`FruitSprites`] changes.
///   Uses the real sprite image when available; falls back to a tinted placeholder circle.
/// - **Visibility**: shown while a held or falling fruit exists; hidden otherwise.
pub fn update_next(
    next_fruit: Res<NextFruitType>,
    fruit_states: Query<&FruitSpawnState, With<Fruit>>,
    mut preview_q: Query<
        (
            &mut BackgroundColor,
            &mut Visibility,
            &mut ImageNode,
            &mut BorderRadius,
        ),
        With<HudNextPreview>,
    >,
    fruit_sprites: Option<Res<FruitSprites>>,
) {
    let has_active = fruit_states
        .iter()
        .any(|s| *s == FruitSpawnState::Held || *s == FruitSpawnState::Falling);

    let sprites_changed = fruit_sprites
        .as_ref()
        .map(|s| s.is_changed())
        .unwrap_or(false);
    let should_update_sprite = next_fruit.is_changed() || sprites_changed;

    for (mut bg, mut vis, mut image_node, mut border_radius) in preview_q.iter_mut() {
        let desired = if has_active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *vis != desired {
            *vis = desired;
        }

        // Always refresh so newly-spawned HUD widgets get the correct state.
        if should_update_sprite || image_node.image == Handle::default() {
            if let Some(handle) = fruit_sprites
                .as_deref()
                .and_then(|s| s.get(next_fruit.get()))
            {
                // Real sprite available — show it directly, no circle clipping.
                image_node.image = handle.clone();
                image_node.color = Color::WHITE;
                *bg = BackgroundColor(Color::NONE);
                *border_radius = BorderRadius::ZERO;
            } else {
                // Fallback: tinted placeholder circle.
                image_node.image = Handle::default();
                image_node.color = Color::WHITE;
                *bg = BackgroundColor(next_fruit.get().placeholder_color());
                *border_radius = BorderRadius::all(Val::Percent(50.0));
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
    fn test_hud_next_preview_marker_exists() {
        let _n = HudNextPreview;
    }

    #[test]
    fn test_default_preview_size_is_positive() {
        assert!(crate::config::NextHudConfig::default().preview_size > 0.0);
    }
}
