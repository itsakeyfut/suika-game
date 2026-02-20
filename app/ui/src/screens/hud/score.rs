//! Current score widget.
//!
//! Renders a single labelled panel showing the score for the current game
//! session.  Positioning is left to the caller — typically [`super::setup_hud`]
//! wraps this widget in an absolute-positioned anchor node.
//!
//! # Usage
//!
//! ```ignore
//! parent_anchor.with_children(|p| score::spawn_score_widget(p, &font, &cfg));
//! app.add_systems(Update, score::update_score.run_if(in_state(AppState::Playing)));
//! app.add_systems(Update, score::animate_score_pulse.after(score::update_score));
//! ```

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use suika_game_core::prelude::GameState;
use suika_game_core::resources::settings::Language;

use crate::config::{ScoreHudConfig, ScoreHudConfigHandle};
use crate::i18n::t;
use crate::screens::title::format_score;
use crate::styles::{BG_COLOR, FONT_SIZE_MEDIUM, FONT_SIZE_SMALL, PRIMARY_COLOR, TEXT_COLOR};

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marks the [`Text`] node that displays the current score value.
#[derive(Component, Debug)]
pub struct HudScore;

/// Drives a font-size pulse animation on the score text node.
///
/// Inserted onto the [`HudScore`] entity the first time the current score
/// exceeds [`GameState::highscore`] in a session.
/// Removed automatically by [`animate_score_pulse`] when the animation
/// completes and the font size is snapped back to `base_font_size`.
///
/// The size multiplier follows `1.0 + (peak_scale − 1.0) × sin(π × t)` where
/// `t = elapsed / duration`, giving a smooth rise-and-fall envelope.
#[derive(Component, Debug, Clone)]
pub struct ScorePulse {
    /// Elapsed time since the pulse started, in seconds
    pub elapsed: f32,
    /// Total duration of the pulse in seconds (loaded from `score.ron`)
    pub duration: f32,
    /// The resting font size to return to when the pulse ends
    pub base_font_size: f32,
    /// Peak scale factor at the midpoint of the pulse (loaded from `score.ron`)
    pub peak_scale: f32,
}

/// Marks the container [`Node`] of the score panel.
///
/// Used by the hot-reload system in [`crate::config`] to update padding and gap
/// values at runtime without re-spawning the HUD.
#[derive(Component, Debug)]
pub struct HudScorePanel;

// ---------------------------------------------------------------------------
// Spawn helper
// ---------------------------------------------------------------------------

/// Spawns the current-score panel as a child of `parent`.
///
/// Layout values (`panel_padding`, `label_value_gap`) come from `cfg`.
///
/// ```text
/// ┌──────────┐
/// │  スコア   │  ← FONT_SIZE_SMALL, TEXT_COLOR
/// │    0      │  ← FONT_SIZE_MEDIUM, PRIMARY_COLOR, HudScore
/// └──────────┘
/// ```
pub fn spawn_score_widget(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    cfg: &ScoreHudConfig,
    lang: Language,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(cfg.panel_padding)),
                row_gap: Val::Px(cfg.label_value_gap),
                ..default()
            },
            BackgroundColor(BG_COLOR),
            BorderRadius::all(Val::Px(8.0)),
            HudScorePanel,
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(t("hud_score", lang)),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_SMALL,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
            panel.spawn((
                Text::new("0"),
                TextFont {
                    font: font.clone(),
                    font_size: FONT_SIZE_MEDIUM,
                    ..default()
                },
                TextColor(PRIMARY_COLOR),
                HudScore,
            ));
        });
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Updates the score text node from [`GameState`].
///
/// Triggers a [`ScorePulse`] on the [`HudScore`] entity the first time the
/// current score exceeds the all-time highscore in a session.
/// Uses a frame-local flag (`was_beating`) to ensure the animation fires only
/// on the transition frame rather than every frame while leading.
pub fn update_score(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut score_q: Query<(Entity, &mut Text), With<HudScore>>,
    mut was_beating: Local<bool>,
    cfg_handle: Option<Res<ScoreHudConfigHandle>>,
    cfg_assets: Res<Assets<ScoreHudConfig>>,
) {
    let Ok((entity, mut text)) = score_q.single_mut() else {
        return;
    };
    text.0 = format_score(game_state.score);

    let default_cfg = ScoreHudConfig::default();
    let cfg = cfg_handle
        .as_ref()
        .and_then(|h| cfg_assets.get(&h.0))
        .unwrap_or(&default_cfg);

    let now_beating = game_state.score > game_state.highscore;
    if now_beating && !*was_beating {
        commands.entity(entity).insert(ScorePulse {
            elapsed: 0.0,
            duration: cfg.pulse_duration,
            base_font_size: FONT_SIZE_MEDIUM,
            peak_scale: cfg.pulse_peak_scale,
        });
    }
    *was_beating = now_beating;
}

/// Advances the [`ScorePulse`] animation on the score text node.
///
/// Each frame the font size is set to `base × (1.0 + (peak − 1.0) × sin(π × t))`,
/// producing a smooth rise-and-fall envelope.  When `elapsed ≥ duration`
/// the component is removed and the font size is snapped back to `base_font_size`.
///
/// `TextFont::font_size` is used instead of `Transform::scale` because Bevy's
/// UI layout pipeline ignores world-space transforms for text rendering.
pub fn animate_score_pulse(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ScorePulse, &mut TextFont)>,
    time: Res<Time>,
) {
    for (entity, mut pulse, mut text_font) in query.iter_mut() {
        pulse.elapsed += time.delta_secs();

        if pulse.elapsed >= pulse.duration {
            text_font.font_size = pulse.base_font_size;
            commands.entity(entity).remove::<ScorePulse>();
            continue;
        }

        let t = pulse.elapsed / pulse.duration;
        // Sine envelope: peaks at t=0.5, returns to 1.0 at t=0 and t=1.
        // Clamped to avoid floating-point noise pushing the multiplier below 1.0.
        let multiplier =
            (1.0 + (pulse.peak_scale - 1.0) * (std::f32::consts::PI * t).sin()).max(1.0);
        text_font.font_size = pulse.base_font_size * multiplier;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_score_marker_exists() {
        let _s = HudScore;
    }

    #[test]
    fn test_hud_score_panel_marker_exists() {
        let _p = HudScorePanel;
    }

    #[test]
    fn test_score_pulse_scale_at_midpoint_is_peak() {
        // At t=0.5 the sine envelope should reach peak_scale
        let peak_scale = ScoreHudConfig::default().pulse_peak_scale;
        let t = 0.5_f32;
        let scale = 1.0 + (peak_scale - 1.0) * (std::f32::consts::PI * t).sin();
        assert!(
            (scale - peak_scale).abs() < 1e-5,
            "Scale at t=0.5 should equal peak_scale ({peak_scale}), got {scale}"
        );
    }

    #[test]
    fn test_score_pulse_scale_at_start_and_end_is_one() {
        // At t=0 and t=1 the sine envelope returns to 1.0
        let peak_scale = ScoreHudConfig::default().pulse_peak_scale;
        for t in [0.0_f32, 1.0_f32] {
            let scale = 1.0 + (peak_scale - 1.0) * (std::f32::consts::PI * t).sin();
            assert!(
                (scale - 1.0).abs() < 1e-5,
                "Scale at t={t} should be 1.0, got {scale}"
            );
        }
    }

    #[test]
    fn test_score_pulse_scale_always_above_one() {
        // In the system, t is always in [0, 1) — at t=1.0 the despawn path
        // runs first and scale is never computed.  So we test t in [0, 1).
        let peak_scale = ScoreHudConfig::default().pulse_peak_scale;
        let steps = 100;
        for i in 0..steps {
            let t = i as f32 / steps as f32; // t in [0.0, 1.0)
            let scale = 1.0 + (peak_scale - 1.0) * (std::f32::consts::PI * t).sin();
            assert!(
                scale >= 1.0,
                "Scale should never go below 1.0 during pulse, got {scale} at t={t}"
            );
        }
    }

    #[test]
    fn test_animate_score_pulse_despawns_component_when_done() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, animate_score_pulse);

        let cfg = ScoreHudConfig::default();

        // Spawn an entity with a pulse that has already expired
        let entity = app
            .world_mut()
            .spawn((
                ScorePulse {
                    elapsed: cfg.pulse_duration,
                    duration: cfg.pulse_duration,
                    base_font_size: FONT_SIZE_MEDIUM,
                    peak_scale: cfg.pulse_peak_scale,
                },
                TextFont {
                    font_size: FONT_SIZE_MEDIUM,
                    ..default()
                },
            ))
            .id();

        app.update();

        assert!(
            app.world().get::<ScorePulse>(entity).is_none(),
            "ScorePulse component should be removed when duration is reached"
        );
        let text_font = app.world().get::<TextFont>(entity).unwrap();
        assert_eq!(
            text_font.font_size, FONT_SIZE_MEDIUM,
            "Font size should snap back to FONT_SIZE_MEDIUM when pulse ends"
        );
    }

    #[test]
    fn test_animate_score_pulse_advances_elapsed() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, animate_score_pulse);

        let entity = app
            .world_mut()
            .spawn((
                ScorePulse {
                    elapsed: 0.0,
                    duration: 10.0, // long duration so it doesn't finish
                    base_font_size: FONT_SIZE_MEDIUM,
                    peak_scale: ScoreHudConfig::default().pulse_peak_scale,
                },
                TextFont {
                    font_size: FONT_SIZE_MEDIUM,
                    ..default()
                },
            ))
            .id();

        // Two updates: first frame has delta=0, second has non-zero delta
        app.update();
        app.update();

        let pulse = app.world().get::<ScorePulse>(entity).unwrap();
        assert!(
            pulse.elapsed > 0.0,
            "elapsed should advance after at least one frame with non-zero delta"
        );
    }
}
