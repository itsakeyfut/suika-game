//! Floating score popup widget.
//!
//! Spawns a `Text2d` entity at the merge position when a fruit merge occurs.
//! The text rises upward and fades out over a configurable duration.
//!
//! # Text format
//!
//! | Combo | Text        |
//! |-------|-------------|
//! | 1     | `+10`       |
//! | 2+    | `+10 ×2`    |
//!
//! # Colors
//!
//! | Combo | Color                        |
//! |-------|------------------------------|
//! | 1     | White                        |
//! | 2     | Silver `srgb(0.75, 0.75, 0.82)` |
//! | 3     | Gold `srgb(1.0, 0.84, 0.0)`  |
//! | 4+    | Rainbow (hue rotation)       |

use bevy::prelude::*;
use suika_game_core::prelude::{FruitsConfig, FruitsConfigHandle, ScoreEarnedEvent};

use crate::config::{ScorePopupConfig, ScorePopupConfigHandle};
use crate::styles::FONT_JP;

// ---------------------------------------------------------------------------
// Default values for RON-loaded parameters
// ---------------------------------------------------------------------------

/// Default fruit radius fallback (px) — mirrors the Cherry entry radius in `fruits.ron`.
const DEFAULT_FRUIT_RADIUS: f32 = 20.0;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Drives the rise-and-fade animation of a floating score popup.
#[derive(Component, Debug)]
pub struct ScorePopup {
    /// Elapsed time since this popup was spawned (seconds).
    pub elapsed: f32,
    /// Total display duration before despawn (seconds).
    pub duration: f32,
    /// Total vertical distance traveled over `duration` (pixels).
    pub rise_distance: f32,
    /// World-space Y position at spawn time.
    pub start_y: f32,
    /// Time at which the alpha fade-out begins (seconds).
    pub fade_start: f32,
    /// Combo count at the time of the merge — determines color behavior.
    pub combo: u32,
    /// Angular speed of the hue rotation for rainbow mode (degrees/second).
    pub rainbow_hue_speed: f32,
    /// Base color used for non-rainbow combos (alpha is overridden each frame).
    pub initial_color: Color,
}

// ---------------------------------------------------------------------------
// Color helper
// ---------------------------------------------------------------------------

/// Returns the initial color for a given combo count.
///
/// ```
/// # use suika_game_ui::screens::hud::score_popup::color_for_combo;
/// // combo=1 → white
/// let c = color_for_combo(1);
/// let s = c.to_srgba();
/// assert!((s.red - 1.0).abs() < 1e-5);
/// assert!((s.green - 1.0).abs() < 1e-5);
/// assert!((s.blue - 1.0).abs() < 1e-5);
/// ```
pub fn color_for_combo(combo: u32) -> Color {
    match combo {
        0 | 1 => Color::WHITE,
        2 => Color::srgb(0.75, 0.75, 0.82), // silver
        3 => Color::srgb(1.0, 0.84, 0.0),   // gold
        _ => Color::hsl(0.0, 1.0, 0.65),    // rainbow start hue
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns floating score popups when fruit merges are scored.
///
/// Reads [`ScoreEarnedEvent`] each frame. For each event, calculates the
/// font size from the resulting fruit's radius and spawns a [`Text2d`]
/// entity with the [`ScorePopup`] component.
///
/// Each event carries the authoritative `earned_points` (after multiplier)
/// and `combo_count` for that specific merge, so all popups in a frame
/// correctly reflect their individual combo state.
///
/// Ordering: must run **after** `update_score_on_merge` which emits the events.
pub fn spawn_score_popups(
    mut commands: Commands,
    mut score_events: MessageReader<ScoreEarnedEvent>,
    fruits_handle: Res<FruitsConfigHandle>,
    fruits_assets: Res<Assets<FruitsConfig>>,
    popup_handle: Option<Res<ScorePopupConfigHandle>>,
    popup_assets: Res<Assets<ScorePopupConfig>>,
    asset_server: Res<AssetServer>,
) {
    let Some(fruits_cfg) = fruits_assets.get(&fruits_handle.0) else {
        for _ in score_events.read() {}
        return;
    };

    let default_popup = ScorePopupConfig::default();
    let popup_cfg = popup_handle
        .as_ref()
        .and_then(|h| popup_assets.get(&h.0))
        .unwrap_or(&default_popup);

    let font: Handle<Font> = asset_server.load(FONT_JP);
    let fade_start = popup_cfg.duration * popup_cfg.fade_start_fraction;

    for event in score_events.read() {
        // Font size scales with the resulting fruit's radius
        let result_type = event.fruit_type.next().unwrap_or(event.fruit_type);
        let radius = result_type
            .try_parameters_from_config(fruits_cfg)
            .map(|p| p.radius)
            .unwrap_or(DEFAULT_FRUIT_RADIUS);
        let font_size = (radius * popup_cfg.font_size_per_radius).max(8.0);

        let combo = event.combo_count;
        let text = if combo <= 1 {
            format!("+{}", event.earned_points)
        } else {
            format!("+{} ×{}", event.earned_points, combo)
        };

        let initial_color = color_for_combo(combo);

        commands.spawn((
            Text2d::new(text),
            TextFont {
                font: font.clone(),
                font_size,
                ..default()
            },
            TextColor(initial_color),
            Transform::from_translation(event.position.extend(popup_cfg.z_layer)),
            ScorePopup {
                elapsed: 0.0,
                duration: popup_cfg.duration,
                rise_distance: popup_cfg.rise_distance,
                start_y: event.position.y,
                fade_start,
                combo,
                rainbow_hue_speed: popup_cfg.rainbow_hue_speed,
                initial_color,
            },
        ));
    }
}

/// Advances all active [`ScorePopup`] animations each frame.
///
/// - Moves the entity upward proportionally to elapsed time.
/// - Fades alpha out linearly after `fade_start` seconds.
/// - For combo ≥ 4: rotates the hue to create a rainbow effect.
/// - Despawns the entity once `elapsed ≥ duration`.
pub fn update_score_popups(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ScorePopup, &mut Transform, &mut TextColor)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut popup, mut transform, mut text_color) in query.iter_mut() {
        popup.elapsed += dt;

        if popup.elapsed >= popup.duration {
            commands.entity(entity).despawn();
            continue;
        }

        // Rise: move linearly upward
        let progress = popup.elapsed / popup.duration;
        transform.translation.y = popup.start_y + progress * popup.rise_distance;

        // Fade: linear from 1.0 to 0.0 after fade_start
        let alpha = if popup.elapsed < popup.fade_start {
            1.0_f32
        } else {
            let fade_progress =
                (popup.elapsed - popup.fade_start) / (popup.duration - popup.fade_start);
            (1.0 - fade_progress).max(0.0)
        };

        // Color: rainbow for combo ≥ 4, otherwise tint initial_color with alpha
        text_color.0 = if popup.combo >= 4 {
            let hue = (popup.elapsed * popup.rainbow_hue_speed).rem_euclid(360.0);
            Color::hsla(hue, 1.0, 0.65, alpha)
        } else {
            popup.initial_color.with_alpha(alpha)
        };
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- color_for_combo ---

    #[test]
    fn test_color_for_combo_1_is_white() {
        let c = color_for_combo(1).to_srgba();
        assert!((c.red - 1.0).abs() < 1e-5, "red should be 1.0");
        assert!((c.green - 1.0).abs() < 1e-5, "green should be 1.0");
        assert!((c.blue - 1.0).abs() < 1e-5, "blue should be 1.0");
    }

    #[test]
    fn test_color_for_combo_0_is_white() {
        // combo=0 treated the same as combo=1
        let c0 = color_for_combo(0).to_srgba();
        let c1 = color_for_combo(1).to_srgba();
        assert!((c0.red - c1.red).abs() < 1e-5);
        assert!((c0.green - c1.green).abs() < 1e-5);
        assert!((c0.blue - c1.blue).abs() < 1e-5);
    }

    #[test]
    fn test_color_for_combo_2_is_silver() {
        let c = color_for_combo(2).to_srgba();
        assert!((c.red - 0.75).abs() < 1e-4, "red ~0.75, got {}", c.red);
        assert!(
            (c.green - 0.75).abs() < 1e-4,
            "green ~0.75, got {}",
            c.green
        );
        assert!((c.blue - 0.82).abs() < 1e-4, "blue ~0.82, got {}", c.blue);
    }

    #[test]
    fn test_color_for_combo_3_is_gold() {
        let c = color_for_combo(3).to_srgba();
        assert!((c.red - 1.0).abs() < 1e-4, "red ~1.0, got {}", c.red);
        assert!(
            (c.green - 0.84).abs() < 1e-4,
            "green ~0.84, got {}",
            c.green
        );
        assert!(c.blue < 0.01, "blue ~0.0, got {}", c.blue);
    }

    #[test]
    fn test_color_for_combo_4_plus_is_hsl() {
        // combo ≥ 4 returns a non-white, non-silver, non-gold color
        let c1 = color_for_combo(1).to_srgba();
        let c2 = color_for_combo(2).to_srgba();
        let c3 = color_for_combo(3).to_srgba();
        let c4 = color_for_combo(4).to_srgba();
        // They should all be distinct
        assert_ne!(
            (c4.red, c4.green, c4.blue),
            (c1.red, c1.green, c1.blue),
            "combo=4 should differ from combo=1"
        );
        assert_ne!(
            (c4.red, c4.green, c4.blue),
            (c2.red, c2.green, c2.blue),
            "combo=4 should differ from combo=2"
        );
        assert_ne!(
            (c4.red, c4.green, c4.blue),
            (c3.red, c3.green, c3.blue),
            "combo=4 should differ from combo=3"
        );
    }

    // --- fade calculation ---

    #[test]
    fn test_fade_before_fade_start_is_one() {
        // elapsed < fade_start → alpha = 1.0
        let fade_start = 0.5_f32;
        let duration = 1.0_f32;
        let elapsed = 0.3_f32;
        let alpha: f32 = if elapsed < fade_start {
            1.0
        } else {
            let p = (elapsed - fade_start) / (duration - fade_start);
            (1.0 - p).max(0.0)
        };
        assert!(
            (alpha - 1.0).abs() < 1e-5,
            "alpha should be 1.0 before fade_start"
        );
    }

    #[test]
    fn test_fade_at_duration_is_zero() {
        // elapsed = duration → alpha ≈ 0.0 (the system despawns before this,
        // but the formula should produce 0.0 at the boundary)
        let fade_start = 0.5_f32;
        let duration = 1.0_f32;
        let elapsed = duration;
        let alpha: f32 = if elapsed < fade_start {
            1.0
        } else {
            let p = (elapsed - fade_start) / (duration - fade_start);
            (1.0 - p).max(0.0)
        };
        assert!(
            alpha < 1e-5,
            "alpha should be ~0.0 at elapsed=duration, got {alpha}"
        );
    }

    // --- despawn ---

    #[test]
    fn test_update_score_popups_despawns_when_done() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, update_score_popups);

        let entity = app
            .world_mut()
            .spawn((
                ScorePopup {
                    elapsed: 1.0,
                    duration: 1.0,
                    rise_distance: 80.0,
                    start_y: 0.0,
                    fade_start: 0.5,
                    combo: 1,
                    rainbow_hue_speed: 180.0,
                    initial_color: Color::WHITE,
                },
                Transform::from_xyz(0.0, 0.0, 8.0),
                TextColor(Color::WHITE),
            ))
            .id();

        app.update();

        assert!(
            app.world().get_entity(entity).is_err(),
            "ScorePopup entity should be despawned when elapsed >= duration"
        );
    }

    #[test]
    fn test_update_score_popups_advances_elapsed() {
        use bevy::time::TimeUpdateStrategy;
        use std::time::Duration;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Fix delta to 16 ms per frame so the test is deterministic regardless
        // of wall-clock speed. The first update initializes Time with delta=0;
        // subsequent updates each advance by exactly 16 ms.
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )));
        app.add_systems(Update, update_score_popups);

        let entity = app
            .world_mut()
            .spawn((
                ScorePopup {
                    elapsed: 0.0,
                    duration: 10.0,
                    rise_distance: 80.0,
                    start_y: 0.0,
                    fade_start: 5.0,
                    combo: 1,
                    rainbow_hue_speed: 180.0,
                    initial_color: Color::WHITE,
                },
                Transform::from_xyz(0.0, 0.0, 8.0),
                TextColor(Color::WHITE),
            ))
            .id();

        app.update(); // frame 0: Time initializes with delta = 0
        app.update(); // frame 1: delta = 16 ms → elapsed should advance

        let popup = app.world().get::<ScorePopup>(entity).unwrap();
        assert!(
            popup.elapsed >= 0.016 - f32::EPSILON,
            "elapsed should be at least 16 ms after second frame, got {}",
            popup.elapsed
        );
    }
}
