//! Score and combo system
//!
//! This module updates `GameState.score` and manages the `ComboTimer`
//! in response to `FruitMergeEvent` messages sent by the collision system.
//!
//! # Scoring rules
//!
//! Each fruit type has a base point value stored in `FruitsConfig`.
//! These points are awarded when a fruit of that type participates in a merge
//! (i.e., the **merging** fruit's points, not the resulting fruit's points).
//!
//! # Combo multipliers
//!
//! Multipliers are loaded from `game_rules.ron` via `combo_bonuses`.
//! The built-in fallback (used when the config is unavailable) matches the
//! default RON values:
//!
//! | Combo count | Multiplier |
//! |-------------|-----------|
//! | 1 (no combo)| 1.0×      |
//! | 2           | 1.1× (+10%)|
//! | 3           | 1.2× (+20%)|
//! | 4           | 1.3× (+30%)|
//! | 5+          | 1.5× (+50%)|

use bevy::prelude::*;

use crate::config::{FruitsConfig, FruitsConfigHandle, GameRulesConfig, GameRulesConfigHandle};
use crate::events::FruitMergeEvent;
use crate::resources::{ComboTimer, GameState};

// ---------------------------------------------------------------------------
// Default combo bonus fallbacks — mirror `game_rules.ron` `combo_bonuses`
// ---------------------------------------------------------------------------

/// Fallback multiplier for a 2× combo — mirrors `game_rules.ron` `combo_bonuses` key 2.
const DEFAULT_COMBO_BONUS_2: f32 = 1.1;
/// Fallback multiplier for a 3× combo — mirrors `game_rules.ron` `combo_bonuses` key 3.
const DEFAULT_COMBO_BONUS_3: f32 = 1.2;
/// Fallback multiplier for a 4× combo — mirrors `game_rules.ron` `combo_bonuses` key 4.
const DEFAULT_COMBO_BONUS_4: f32 = 1.3;
/// Fallback multiplier for a 5×+ combo — mirrors `game_rules.ron` `combo_bonuses` key 5.
const DEFAULT_COMBO_BONUS_5_PLUS: f32 = 1.5;

/// Returns the combo score multiplier for the given combo count.
///
/// When `rules` is `Some`, the multiplier is looked up from
/// `GameRulesConfig::combo_bonuses`.  The map is treated as a step function:
/// the entry with the largest key ≤ `combo` is used, so a map with keys
/// `{2, 3, 4, 5}` naturally covers all combos ≥ 5 through the `5` entry.
///
/// When `rules` is `None` the function falls back to the hardcoded defaults
/// that match the shipped `game_rules.ron`.
///
/// # Examples
///
/// ```
/// # use suika_game_core::systems::score::combo_multiplier;
/// assert_eq!(combo_multiplier(0, None), 1.0);
/// assert_eq!(combo_multiplier(1, None), 1.0);
/// assert_eq!(combo_multiplier(2, None), 1.1);
/// assert_eq!(combo_multiplier(3, None), 1.2);
/// assert_eq!(combo_multiplier(4, None), 1.3);
/// assert_eq!(combo_multiplier(5, None), 1.5);
/// assert_eq!(combo_multiplier(10, None), 1.5);
/// ```
pub fn combo_multiplier(combo: u32, rules: Option<&GameRulesConfig>) -> f32 {
    if let Some(rules) = rules {
        // Step-function lookup: find the largest key ≤ combo.
        rules
            .combo_bonuses
            .iter()
            .filter(|&(&k, _)| k <= combo)
            .max_by_key(|&(&k, _)| k)
            .map(|(_, &v)| v)
            .unwrap_or(1.0)
    } else {
        // Hardcoded fallback — mirrors the default game_rules.ron values.
        match combo {
            0 | 1 => 1.0,
            2 => DEFAULT_COMBO_BONUS_2,
            3 => DEFAULT_COMBO_BONUS_3,
            4 => DEFAULT_COMBO_BONUS_4,
            _ => DEFAULT_COMBO_BONUS_5_PLUS,
        }
    }
}

/// Updates score and combo state in response to `FruitMergeEvent`
///
/// For each merge event:
/// 1. Registers the merge with `ComboTimer` (updates combo count and window)
/// 2. Calculates base points from the merged fruit's config entry
/// 3. Applies the combo multiplier from `GameRulesConfig::combo_bonuses`
/// 4. Adds the result to `GameState.score`
///
/// If the fruits config is not yet loaded, events are drained silently.
pub fn update_score_on_merge(
    mut merge_events: MessageReader<FruitMergeEvent>,
    mut game_state: ResMut<GameState>,
    mut combo_timer: ResMut<ComboTimer>,
    fruits_handle: Res<FruitsConfigHandle>,
    fruits_assets: Res<Assets<FruitsConfig>>,
    rules_handle: Option<Res<GameRulesConfigHandle>>,
    rules_assets: Option<Res<Assets<GameRulesConfig>>>,
) {
    let Some(config) = fruits_assets.get(&fruits_handle.0) else {
        for _ in merge_events.read() {}
        return;
    };

    let rules = rules_handle
        .as_ref()
        .zip(rules_assets.as_ref())
        .and_then(|(h, a)| a.get(&h.0));

    for event in merge_events.read() {
        // Update the combo timer first so the multiplier reflects this merge
        combo_timer.register_merge();
        let multiplier = combo_multiplier(combo_timer.current_combo, rules);

        // Base points from the merged fruit type (not the resulting fruit)
        let base_points = event
            .fruit_type
            .try_parameters_from_config(config)
            .map(|p| p.points)
            .unwrap_or(0);

        let earned = (base_points as f32 * multiplier).round() as u32;
        game_state.score = game_state.score.saturating_add(earned);

        if combo_timer.is_combo() {
            info!(
                "Merge scored {} pts ({}× combo {}): {:?} → total {}",
                earned, multiplier, combo_timer.current_combo, event.fruit_type, game_state.score
            );
        } else {
            info!(
                "Merge scored {} pts: {:?} → total {}",
                earned, event.fruit_type, game_state.score
            );
        }
    }
}

/// Ticks `ComboTimer` every frame and resets it when the combo window expires
///
/// Must run every frame to keep `time_since_last_merge` up to date.
pub fn tick_combo_timer(mut combo_timer: ResMut<ComboTimer>, time: Res<Time>) {
    combo_timer.tick(time.delta_secs());
    combo_timer.check_and_reset();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FruitConfigEntry, FruitsConfig, FruitsConfigHandle};
    use crate::events::FruitMergeEvent;
    use crate::fruit::FruitType;
    use crate::resources::{ComboTimer, GameState};
    use std::collections::HashMap;

    fn create_test_config() -> FruitsConfig {
        FruitsConfig {
            fruits: (0..11)
                .map(|i| FruitConfigEntry {
                    name: format!("Fruit{i}"),
                    radius: 20.0 + i as f32 * 10.0,
                    points: 10 * (1u32 << i), // 10, 20, 40, ...
                    restitution: 0.3,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                })
                .collect(),
        }
    }

    fn setup_score_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<FruitMergeEvent>();
        app.add_systems(Update, update_score_on_merge);
        app.init_resource::<GameState>();
        app.init_resource::<ComboTimer>();

        let mut fruits_assets = Assets::<FruitsConfig>::default();
        let handle = fruits_assets.add(create_test_config());
        app.insert_resource(fruits_assets);
        app.insert_resource(FruitsConfigHandle(handle));

        app
        // Note: GameRulesConfig is intentionally omitted here.
        // update_score_on_merge falls back to hardcoded multipliers when absent.
    }

    // --- combo_multiplier unit tests (fallback / None path) ---

    #[test]
    fn test_combo_multiplier_no_combo() {
        assert_eq!(combo_multiplier(1, None), 1.0);
    }

    #[test]
    fn test_combo_multiplier_levels() {
        assert!((combo_multiplier(2, None) - DEFAULT_COMBO_BONUS_2).abs() < f32::EPSILON);
        assert!((combo_multiplier(3, None) - DEFAULT_COMBO_BONUS_3).abs() < f32::EPSILON);
        assert!((combo_multiplier(4, None) - DEFAULT_COMBO_BONUS_4).abs() < f32::EPSILON);
        assert!((combo_multiplier(5, None) - DEFAULT_COMBO_BONUS_5_PLUS).abs() < f32::EPSILON);
        assert!((combo_multiplier(10, None) - DEFAULT_COMBO_BONUS_5_PLUS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_combo_multiplier_from_config() {
        let rules = GameRulesConfig {
            spawnable_fruit_count: 5,
            combo_window: 2.0,
            combo_max: 10,
            game_over_timer: 3.0,
            combo_bonuses: HashMap::from([(2, 2.0), (3, 3.0), (5, 5.0)]),
            preview_x_offset: 0.0,
            preview_y_offset: 0.0,
            preview_scale: 1.0,
        };
        // combo=1 → no key ≤ 1 in map → 1.0
        assert!((combo_multiplier(1, Some(&rules)) - 1.0).abs() < f32::EPSILON);
        // combo=2 → key 2 → 2.0
        assert!((combo_multiplier(2, Some(&rules)) - 2.0).abs() < f32::EPSILON);
        // combo=4 → largest key ≤ 4 is 3 → 3.0
        assert!((combo_multiplier(4, Some(&rules)) - 3.0).abs() < f32::EPSILON);
        // combo=10 → largest key ≤ 10 is 5 → 5.0
        assert!((combo_multiplier(10, Some(&rules)) - 5.0).abs() < f32::EPSILON);
    }

    // --- update_score_on_merge system tests ---

    #[test]
    fn test_single_merge_adds_score() {
        let mut app = setup_score_app();

        app.world_mut().write_message(FruitMergeEvent {
            entity1: Entity::from_bits(1),
            entity2: Entity::from_bits(2),
            fruit_type: FruitType::Cherry, // Cherry.points = 10
            position: Vec2::ZERO,
        });

        app.update();

        let score = app.world().resource::<GameState>().score;
        // No combo on first merge (was f32::MAX), multiplier = 1.0
        assert_eq!(score, 10, "Cherry merge should award 10 pts with no combo");
    }

    #[test]
    fn test_combo_bonus_applied() {
        let mut app = setup_score_app();

        // First merge
        app.world_mut().write_message(FruitMergeEvent {
            entity1: Entity::from_bits(1),
            entity2: Entity::from_bits(2),
            fruit_type: FruitType::Cherry, // 10 pts
            position: Vec2::ZERO,
        });
        app.update();

        // Second merge immediately (within combo window, time_since_last_merge = 0)
        app.world_mut().write_message(FruitMergeEvent {
            entity1: Entity::from_bits(3),
            entity2: Entity::from_bits(4),
            fruit_type: FruitType::Cherry, // 10 pts × 1.1 = 11 pts
            position: Vec2::ZERO,
        });
        app.update();

        let score = app.world().resource::<GameState>().score;
        // 10 (first) + 11 (second with +10% combo) = 21
        assert_eq!(
            score, 21,
            "Second merge within window should apply +10% combo"
        );
    }

    #[test]
    fn test_score_uses_fruit_type_points() {
        let mut app = setup_score_app();

        // Grape is index 2 → points = 10 * 4 = 40
        app.world_mut().write_message(FruitMergeEvent {
            entity1: Entity::from_bits(1),
            entity2: Entity::from_bits(2),
            fruit_type: FruitType::Grape,
            position: Vec2::ZERO,
        });

        app.update();

        let score = app.world().resource::<GameState>().score;
        assert_eq!(score, 40, "Grape merge should award 40 pts");
    }

    #[test]
    fn test_score_saturates_on_overflow() {
        let mut app = setup_score_app();

        // Pre-set score near max
        app.world_mut().resource_mut::<GameState>().score = u32::MAX - 5;

        app.world_mut().write_message(FruitMergeEvent {
            entity1: Entity::from_bits(1),
            entity2: Entity::from_bits(2),
            fruit_type: FruitType::Watermelon, // index 10 → 10240 pts
            position: Vec2::ZERO,
        });

        app.update();

        let score = app.world().resource::<GameState>().score;
        assert_eq!(score, u32::MAX, "Score should saturate at u32::MAX");
    }

    #[test]
    fn test_combo_timer_updated_on_merge() {
        let mut app = setup_score_app();

        // Start with the default (time = f32::MAX, combo = 1)
        {
            let timer = app.world().resource::<ComboTimer>();
            assert_eq!(timer.current_combo, 1);
        }

        app.world_mut().write_message(FruitMergeEvent {
            entity1: Entity::from_bits(1),
            entity2: Entity::from_bits(2),
            fruit_type: FruitType::Cherry,
            position: Vec2::ZERO,
        });
        app.update();

        // After first merge: combo stays 1 (was f32::MAX → reset window)
        // but time_since_last_merge is now 0
        {
            let timer = app.world().resource::<ComboTimer>();
            assert_eq!(timer.current_combo, 1);
            assert_eq!(timer.time_since_last_merge, 0.0);
        }

        // Immediate second merge (still at 0 seconds) → combo should increment
        app.world_mut().write_message(FruitMergeEvent {
            entity1: Entity::from_bits(3),
            entity2: Entity::from_bits(4),
            fruit_type: FruitType::Cherry,
            position: Vec2::ZERO,
        });
        app.update();

        {
            let timer = app.world().resource::<ComboTimer>();
            assert_eq!(timer.current_combo, 2);
        }
    }
}
