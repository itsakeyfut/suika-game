//! Fruit merge handling system
//!
//! This module processes `FruitMergeEvent` sent by the collision detection system.
//! It despawns both colliding fruits and spawns the next evolution stage at the
//! merge midpoint. For Watermelons (the final stage), both fruits are simply
//! removed without spawning a new one.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy_rapier2d::prelude::ActiveEvents;

use crate::components::FruitSpawnState;
use crate::config::{FruitsConfig, FruitsConfigHandle};
use crate::events::FruitMergeEvent;
use crate::systems::spawn::spawn_fruit;

/// Processes `FruitMergeEvent` and performs the actual fruit merge
///
/// For each merge event:
/// 1. Despawns both source fruit entities
/// 2. If the fruit type has a next evolution stage, spawns it at the midpoint
/// 3. If the fruit is Watermelon (final stage), both fruits disappear
///
/// # Duplicate despawn prevention
///
/// A local `HashSet` tracks entities already despawned within the current frame.
/// This prevents a panic if the same entity appears in multiple events (e.g., a
/// fruit that simultaneously satisfies two merge conditions).
///
/// # Config loading
///
/// If the fruits config asset is not yet loaded, all events are consumed but
/// no merges are executed. This avoids panics during startup.
pub fn handle_fruit_merge(
    mut commands: Commands,
    mut merge_events: MessageReader<FruitMergeEvent>,
    fruits_handle: Res<FruitsConfigHandle>,
    fruits_assets: Res<Assets<FruitsConfig>>,
) {
    let Some(fruits_config) = fruits_assets.get(&fruits_handle.0) else {
        // Drain events to prevent stale buffering
        for _ in merge_events.read() {}
        return;
    };

    let mut despawned: HashSet<Entity> = HashSet::new();

    for event in merge_events.read() {
        // Skip if either entity was already despawned this frame
        if despawned.contains(&event.entity1) || despawned.contains(&event.entity2) {
            warn!(
                "Skipping duplicate merge for already-despawned entity (fruit: {:?})",
                event.fruit_type
            );
            continue;
        }

        // Despawn both source fruits
        commands.entity(event.entity1).despawn();
        commands.entity(event.entity2).despawn();
        despawned.insert(event.entity1);
        despawned.insert(event.entity2);

        // Spawn next evolution, or just remove both if Watermelon (final stage)
        if let Some(next_type) = event.fruit_type.next() {
            let entity = spawn_fruit(&mut commands, next_type, event.position, fruits_config);

            // Add components required for the fruit to participate in collision detection
            commands.entity(entity).insert((
                next_type,
                FruitSpawnState::Falling,
                ActiveEvents::COLLISION_EVENTS,
            ));

            info!(
                "Merged {:?} + {:?} → {:?} at {:?}",
                event.fruit_type, event.fruit_type, next_type, event.position
            );
        } else {
            // Watermelon is the final stage: both fruits vanish
            info!(
                "Watermelon pair merged and disappeared at {:?}",
                event.position
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Fruit;
    use crate::config::{FruitConfigEntry, FruitsConfig};
    use crate::fruit::FruitType;
    use crate::systems::spawn::spawn_fruit;

    fn create_test_config() -> FruitsConfig {
        FruitsConfig {
            fruits: (0..11)
                .map(|i| FruitConfigEntry {
                    name: format!("Fruit{i}"),
                    radius: 20.0 + i as f32 * 10.0,
                    points: 10 * (1 << i),
                    restitution: 0.3,
                    friction: 0.5,
                    mass_multiplier: 0.01,
                })
                .collect(),
        }
    }

    #[test]
    fn test_watermelon_has_no_next() {
        // Watermelon is the final stage and should not produce a next fruit
        assert_eq!(FruitType::Watermelon.next(), None);
    }

    #[test]
    fn test_all_other_fruits_have_next() {
        let non_final = [
            FruitType::Cherry,
            FruitType::Strawberry,
            FruitType::Grape,
            FruitType::Dekopon,
            FruitType::Persimmon,
            FruitType::Apple,
            FruitType::Pear,
            FruitType::Peach,
            FruitType::Pineapple,
            FruitType::Melon,
        ];
        for ft in non_final {
            assert!(ft.next().is_some(), "{:?} should have a next evolution", ft);
        }
    }

    #[test]
    fn test_spawn_fruit_creates_valid_entity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let config = create_test_config();

        let mut commands = app.world_mut().commands();
        let entity = spawn_fruit(
            &mut commands,
            FruitType::Strawberry,
            Vec2::new(0.0, 0.0),
            &config,
        );
        app.update();

        assert!(
            app.world().get_entity(entity).is_ok(),
            "Spawned merge result entity should exist"
        );
        assert!(
            app.world().get::<Fruit>(entity).is_some(),
            "Merged entity should have Fruit component"
        );
    }

    #[test]
    fn test_duplicate_despawn_prevention() {
        // Verify that a HashSet correctly deduplicates entities
        let mut despawned: HashSet<Entity> = HashSet::new();
        let e = Entity::from_bits(42);

        despawned.insert(e);
        assert!(despawned.contains(&e));

        // Inserting again should be a no-op
        despawned.insert(e);
        assert_eq!(despawned.len(), 1);
    }
}
