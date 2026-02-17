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
use crate::config::{BounceConfig, BounceConfigHandle, FruitsConfig, FruitsConfigHandle};
use crate::events::FruitMergeEvent;
use crate::systems::effects::bounce::SquashStretchAnimation;
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
    bounce_handle: Option<Res<BounceConfigHandle>>,
    bounce_assets: Option<Res<Assets<BounceConfig>>>,
) {
    let Some(fruits_config) = fruits_assets.get(&fruits_handle.0) else {
        // Drain events to prevent stale buffering
        for _ in merge_events.read() {}
        return;
    };

    let bounce_config = bounce_handle
        .as_ref()
        .zip(bounce_assets.as_ref())
        .and_then(|(h, a)| a.get(&h.0));

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

            // Add components required for collision detection and the pop-in animation
            commands.entity(entity).insert((
                next_type,
                FruitSpawnState::Falling,
                ActiveEvents::COLLISION_EVENTS,
                SquashStretchAnimation::for_merge(bounce_config),
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
    use crate::components::{Fruit, FruitSpawnState};
    use crate::config::{FruitConfigEntry, FruitsConfig, FruitsConfigHandle};
    use crate::events::FruitMergeEvent;
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

    /// Build a minimal app wired with the handle_fruit_merge system and a
    /// pre-loaded FruitsConfig asset so tests can drive it directly.
    fn setup_merge_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<FruitMergeEvent>();
        app.add_systems(Update, handle_fruit_merge);

        let mut fruits_assets = Assets::<FruitsConfig>::default();
        let handle = fruits_assets.add(create_test_config());
        app.insert_resource(fruits_assets);
        app.insert_resource(FruitsConfigHandle(handle));

        app
    }

    /// Spawn a minimal fruit entity suitable for merge tests
    fn spawn_test_fruit(app: &mut App, fruit_type: FruitType) -> Entity {
        let config = create_test_config();
        let mut commands = app.world_mut().commands();
        let entity = spawn_fruit(&mut commands, fruit_type, Vec2::ZERO, &config);
        // Flush commands so entity exists before further operations
        app.update();
        entity
    }

    #[test]
    fn test_merge_spawns_next_evolution_fruit() {
        let mut app = setup_merge_app();

        let e1 = spawn_test_fruit(&mut app, FruitType::Cherry);
        let e2 = spawn_test_fruit(&mut app, FruitType::Cherry);

        // Send a merge event for the two cherries
        app.world_mut().write_message(FruitMergeEvent {
            entity1: e1,
            entity2: e2,
            fruit_type: FruitType::Cherry,
            position: Vec2::ZERO,
        });

        app.update();

        // Source entities should be despawned
        assert!(
            app.world().get_entity(e1).is_err(),
            "entity1 should be despawned after merge"
        );
        assert!(
            app.world().get_entity(e2).is_err(),
            "entity2 should be despawned after merge"
        );

        // One new Strawberry fruit should have been spawned
        let strawberry_count = app
            .world_mut()
            .query_filtered::<&FruitType, With<Fruit>>()
            .iter(app.world())
            .filter(|&&ft| ft == FruitType::Strawberry)
            .count();

        assert_eq!(
            strawberry_count, 1,
            "Merging two cherries should spawn exactly one strawberry"
        );
    }

    #[test]
    fn test_watermelon_merge_despawns_both_without_new_fruit() {
        let mut app = setup_merge_app();

        let e1 = spawn_test_fruit(&mut app, FruitType::Watermelon);
        let e2 = spawn_test_fruit(&mut app, FruitType::Watermelon);

        let fruit_count_before = app
            .world_mut()
            .query_filtered::<Entity, With<Fruit>>()
            .iter(app.world())
            .count();

        app.world_mut().write_message(FruitMergeEvent {
            entity1: e1,
            entity2: e2,
            fruit_type: FruitType::Watermelon,
            position: Vec2::ZERO,
        });

        app.update();

        let fruit_count_after = app
            .world_mut()
            .query_filtered::<Entity, With<Fruit>>()
            .iter(app.world())
            .count();

        // Both watermelons removed, no new fruit spawned → count decreases by 2
        assert_eq!(
            fruit_count_after,
            fruit_count_before - 2,
            "Watermelon merge should remove 2 fruits and spawn none"
        );
    }

    #[test]
    fn test_duplicate_merge_event_only_despawns_once() {
        let mut app = setup_merge_app();

        let e1 = spawn_test_fruit(&mut app, FruitType::Cherry);
        let e2 = spawn_test_fruit(&mut app, FruitType::Cherry);
        let e3 = spawn_test_fruit(&mut app, FruitType::Cherry);

        // Two events share entity e1 — only the first should be processed
        app.world_mut().write_message(FruitMergeEvent {
            entity1: e1,
            entity2: e2,
            fruit_type: FruitType::Cherry,
            position: Vec2::ZERO,
        });
        app.world_mut().write_message(FruitMergeEvent {
            entity1: e1,
            entity2: e3,
            fruit_type: FruitType::Cherry,
            position: Vec2::ZERO,
        });

        // Should not panic even though e1 would be despawned twice without dedup
        app.update();

        // Only one strawberry should be spawned (from the first event only)
        let strawberry_count = app
            .world_mut()
            .query_filtered::<&FruitType, With<Fruit>>()
            .iter(app.world())
            .filter(|&&ft| ft == FruitType::Strawberry)
            .count();

        assert_eq!(
            strawberry_count, 1,
            "Duplicate events sharing an entity should produce only one merged fruit"
        );
    }

    #[test]
    fn test_merged_fruit_has_falling_state() {
        let mut app = setup_merge_app();

        let e1 = spawn_test_fruit(&mut app, FruitType::Cherry);
        let e2 = spawn_test_fruit(&mut app, FruitType::Cherry);

        app.world_mut().write_message(FruitMergeEvent {
            entity1: e1,
            entity2: e2,
            fruit_type: FruitType::Cherry,
            position: Vec2::ZERO,
        });

        app.update();

        // The merged Strawberry should start in Falling state
        let falling_count = app
            .world_mut()
            .query_filtered::<&FruitSpawnState, With<Fruit>>()
            .iter(app.world())
            .filter(|&&state| state == FruitSpawnState::Falling)
            .count();

        assert_eq!(falling_count, 1, "Merged fruit should be in Falling state");
    }
}
