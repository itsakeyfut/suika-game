//! How-to-play screen — explains the game rules in a two-column layout.
//!
//! ```text
//!          遊び方 / How to Play
//!
//!  ┌──────┐  フルーツを落とす
//!  │ 色枠 │  左右キー / マウスで移動
//!  └──────┘  クリック / スペースで落下
//!
//!  ┌──────┐  同じフルーツが合体
//!  │ 色枠 │  隣接する同種フルーツが自動合体
//!  └──────┘  スコア獲得！
//!
//!  ┌──────┐  スイカを目指せ
//!  │ 色枠 │  合体するたびに大きなフルーツに進化
//!  └──────┘  スイカが最大！
//!
//!  ┌──────┐  ゲームオーバー
//!  │ 色枠 │  フルーツが境界ラインを超えたら終了
//!  └──────┘
//!
//!           [ もどる ]
//! ```
//!
//! The coloured placeholder rectangles on the left are ready to be replaced
//! with real images in a future iteration — just swap the [`BackgroundColor`]
//! node for an [`ImageNode`].
//!
//! All entities are tagged with [`DespawnOnExit`]`(`[`AppState::HowToPlay`]`)`
//! so Bevy cleans them up automatically on state exit.

use bevy::prelude::*;
use suika_game_core::prelude::AppState;
use suika_game_core::resources::settings::SettingsResource;

use crate::components::{ButtonAction, KeyboardFocusIndex, spawn_button};
use crate::i18n::t;
use crate::styles::{
    BG_COLOR, BUTTON_LARGE_HEIGHT, BUTTON_LARGE_WIDTH, FONT_JP, FONT_SIZE_LARGE, FONT_SIZE_MEDIUM,
    FONT_SIZE_SMALL, PRIMARY_COLOR, SECONDARY_COLOR, TEXT_COLOR,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const PLACEHOLDER_SIZE: f32 = 100.0;
const ROW_GAP: f32 = 24.0;

/// Colour cycling for the placeholder image boxes.
const PLACEHOLDER_COLORS: [Color; 4] = [
    Color::srgb(0.9, 0.4, 0.4), // red-ish — "drop"
    Color::srgb(0.4, 0.7, 0.4), // green-ish — "merge"
    Color::srgb(0.4, 0.6, 0.9), // blue-ish — "evolve"
    Color::srgb(0.8, 0.6, 0.2), // orange-ish — "game over"
];

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the how-to-play screen UI when entering [`AppState::HowToPlay`].
pub fn setup_how_to_play_screen(
    mut commands: Commands,
    settings: Res<SettingsResource>,
    asset_server: Res<AssetServer>,
    mut keyboard_focus: ResMut<KeyboardFocusIndex>,
) {
    keyboard_focus.0 = 0;

    let font: Handle<Font> = asset_server.load(FONT_JP);
    let lang = settings.language;

    // ── step definitions ──────────────────────────────────────────────────
    let steps: [(usize, &str, &str); 4] = [
        (0, "htp_drop_title", "htp_drop_body"),
        (1, "htp_merge_title", "htp_merge_body"),
        (2, "htp_evolve_title", "htp_evolve_body"),
        (3, "htp_gameover_title", "htp_gameover_body"),
    ];

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(BG_COLOR),
            DespawnOnExit(AppState::HowToPlay),
        ))
        .with_children(|root| {
            // Title
            root.spawn((
                Text::new(t("how_to_play_title", lang)),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_LARGE,
                    ..default()
                },
                TextColor(PRIMARY_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(32.0)),
                    ..default()
                },
            ));

            // One row per step
            for (i, title_key, body_key) in steps {
                root.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::vertical(Val::Px(ROW_GAP / 2.0)),
                    column_gap: Val::Px(24.0),
                    ..default()
                })
                .with_children(|row| {
                    // Left: coloured placeholder rectangle
                    row.spawn((
                        Node {
                            width: Val::Px(PLACEHOLDER_SIZE),
                            height: Val::Px(PLACEHOLDER_SIZE),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(PLACEHOLDER_COLORS[i]),
                        BorderColor::all(SECONDARY_COLOR),
                        BorderRadius::all(Val::Px(8.0)),
                    ));

                    // Right: title + body text column
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(360.0),
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|col| {
                        col.spawn((
                            Text::new(t(title_key, lang)),
                            TextFont {
                                font: font.clone(),
                                font_size: FONT_SIZE_SMALL,
                                ..default()
                            },
                            TextColor(PRIMARY_COLOR),
                        ));
                        col.spawn((
                            Text::new(t(body_key, lang)),
                            TextFont {
                                font: font.clone(),
                                font_size: FONT_SIZE_SMALL,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ));
                    });
                });
            }

            // Back button (index 0 — initial keyboard focus)
            spawn_button(
                root,
                t("btn_back", lang),
                ButtonAction::BackToTitle,
                0,
                FONT_SIZE_MEDIUM,
                BUTTON_LARGE_WIDTH,
                BUTTON_LARGE_HEIGHT,
                font.clone(),
            );
        });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_colors_count() {
        // Must match the number of steps defined in setup_how_to_play_screen.
        assert_eq!(PLACEHOLDER_COLORS.len(), 4);
    }

    #[test]
    fn test_placeholder_size_positive() {
        assert!(PLACEHOLDER_SIZE > 0.0);
    }
}
