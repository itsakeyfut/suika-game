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
//! | Combo count | Multiplier |
//! |-------------|-----------|
//! | 1 (no combo)| 1.0×      |
//! | 2           | 1.1× (+10%)|
//! | 3           | 1.2× (+20%)|
//! | 4           | 1.3× (+30%)|
//! | 5+          | 1.5× (+50%)|

use bevy::prelude::*;

use crate::config::{FruitsConfig, FruitsConfigHandle};
use crate::events::FruitMergeEvent;
use crate::resources::{ComboTimer, GameState};

/// Returns the combo score multiplier for a given combo count
///
/// # Examples
///
/// ```
/// # use suika_game_core::systems::score::combo_multiplier;
/// assert_eq!(combo_multiplier(1), 1.0);
/// assert_eq!(combo_multiplier(2), 1.1);
/// assert_eq!(combo_multiplier(3), 1.2);
/// assert_eq!(combo_multiplier(4), 1.3);
/// assert_eq!(combo_multiplier(5), 1.5);
/// assert_eq!(combo_multiplier(10), 1.5);
/// ```
pub fn combo_multiplier(combo: u32) -> f32 {
    match combo {
        1 => 1.0,
        2 => 1.1,
        3 => 1.2,
        4 => 1.3,
        _ => 1.5, // 5+
    }
}

/// Updates score and combo state in response to `FruitMergeEvent`
///
/// For each merge event:
/// 1. Registers the merge with `ComboTimer` (updates combo count and window)
/// 2. Calculates base points from the merged fruit's config entry
/// 3. Applies the combo multiplier (`combo_multiplier`)
/// 4. Adds the result to `GameState.score`
///
/// If the fruits config is not yet loaded, events are drained silently.
pub fn update_score_on_merge(
    mut merge_events: MessageReader<FruitMergeEvent>,
    mut game_state: ResMut<GameState>,
    mut combo_timer: ResMut<ComboTimer>,
    fruits_handle: Res<FruitsConfigHandle>,
    fruits_assets: Res<Assets<FruitsConfig>>,
) {
    let Some(config) = fruits_assets.get(&fruits_handle.0) else {
        for _ in merge_events.read() {}
        return;
    };

    for event in merge_events.read() {
        // Update the combo timer first so the multiplier reflects this merge
        combo_timer.register_merge();
        let multiplier = combo_multiplier(combo_timer.current_combo);

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
    }

    // --- combo_multiplier unit tests ---

    #[test]
    fn test_combo_multiplier_no_combo() {
        assert_eq!(combo_multiplier(1), 1.0);
    }

    #[test]
    fn test_combo_multiplier_levels() {
        assert!((combo_multiplier(2) - 1.1).abs() < f32::EPSILON);
        assert!((combo_multiplier(3) - 1.2).abs() < f32::EPSILON);
        assert!((combo_multiplier(4) - 1.3).abs() < f32::EPSILON);
        assert!((combo_multiplier(5) - 1.5).abs() < f32::EPSILON);
        assert!((combo_multiplier(10) - 1.5).abs() < f32::EPSILON);
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
