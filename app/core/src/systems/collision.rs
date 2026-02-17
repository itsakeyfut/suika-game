//! Fruit collision detection system
//!
//! This module monitors Rapier2D collision events and detects when two fruits
//! of the same type collide, triggering the merge system via `FruitMergeEvent`.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy_rapier2d::prelude::CollisionEvent;

use crate::components::{Fruit, FruitSpawnState, MergeCandidate};
use crate::events::FruitMergeEvent;
use crate::fruit::FruitType;

/// Resource tracking entity pairs processed this frame to prevent duplicate merge events
///
/// Stores normalized (min, max) entity pairs to ensure each collision is only
/// processed once per frame, even if multiple collision events fire for the same pair.
#[derive(Resource, Default)]
pub struct ProcessedCollisions {
    pub pairs: HashSet<(Entity, Entity)>,
}

/// Detects collisions between fruits of the same type and fires `FruitMergeEvent`
///
/// This system reads Rapier2D `CollisionEvent::Started` events and checks whether
/// both entities are fruits with the same `FruitType`. When a valid merge is
/// detected, both fruits are marked with `MergeCandidate` and a `FruitMergeEvent`
/// is sent.
///
/// # Deduplication
///
/// Entity pairs are normalized (min entity, max entity) and tracked in
/// `ProcessedCollisions` to prevent duplicate events within a single frame.
/// Fruits already marked as `MergeCandidate` are skipped.
///
/// # Conditions for a merge
///
/// - Both entities must have the `Fruit` component
/// - Neither entity may already be a `MergeCandidate`
/// - Neither may be in `FruitSpawnState::Held` state (still aimed by the player)
/// - Both must have the same `FruitType`
#[allow(clippy::type_complexity)]
pub fn detect_fruit_collision(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    fruit_query: Query<(&FruitType, &FruitSpawnState), (With<Fruit>, Without<MergeCandidate>)>,
    transform_query: Query<&Transform>,
    mut merge_events: MessageWriter<FruitMergeEvent>,
    mut processed: ResMut<ProcessedCollisions>,
) {
    for event in collision_events.read() {
        let CollisionEvent::Started(entity1, entity2, _flags) = event else {
            continue;
        };

        let (entity1, entity2) = (*entity1, *entity2);

        // Normalize pair to prevent duplicate processing (smaller entity first)
        let pair = if entity1 < entity2 {
            (entity1, entity2)
        } else {
            (entity2, entity1)
        };

        if processed.pairs.contains(&pair) {
            continue;
        }

        // Check both entities are fruits not already merging
        let Ok((type1, state1)) = fruit_query.get(entity1) else {
            continue;
        };
        let Ok((type2, state2)) = fruit_query.get(entity2) else {
            continue;
        };

        // Skip fruits still held by the player (not yet dropped)
        if *state1 == FruitSpawnState::Held || *state2 == FruitSpawnState::Held {
            continue;
        }

        // Only merge fruits of the same type
        if type1 != type2 {
            continue;
        }

        let fruit_type = *type1;

        // Calculate merge position as midpoint between the two fruits
        let pos1 = transform_query
            .get(entity1)
            .map(|t| t.translation.truncate())
            .unwrap_or(Vec2::ZERO);
        let pos2 = transform_query
            .get(entity2)
            .map(|t| t.translation.truncate())
            .unwrap_or(Vec2::ZERO);
        let position = (pos1 + pos2) / 2.0;

        // Mark both fruits as merge candidates to prevent further collision processing
        commands.entity(entity1).insert(MergeCandidate);
        commands.entity(entity2).insert(MergeCandidate);

        processed.pairs.insert(pair);

        merge_events.write(FruitMergeEvent {
            entity1,
            entity2,
            fruit_type,
            position,
        });

        info!(
            "Merge detected: {:?} + {:?} = {:?} at {:?}",
            fruit_type,
            fruit_type,
            fruit_type.next(),
            position
        );
    }
}

/// Clears the processed collision pairs at the end of each frame
///
/// This allows the same entity pair to be processed again in subsequent frames
/// if a new collision starts after a previous merge completes.
pub fn clear_processed_collisions(mut processed: ResMut<ProcessedCollisions>) {
    processed.pairs.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processed_collisions_default_is_empty() {
        let processed = ProcessedCollisions::default();
        assert!(processed.pairs.is_empty());
    }

    #[test]
    fn test_processed_collisions_insert_and_contains() {
        let mut processed = ProcessedCollisions::default();
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        let pair = (e1, e2);

        processed.pairs.insert(pair);
        assert!(processed.pairs.contains(&pair));
    }

    #[test]
    fn test_processed_collisions_normalized_pair() {
        let mut processed = ProcessedCollisions::default();
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);

        // Insert (e1, e2)
        let pair = if e1 < e2 { (e1, e2) } else { (e2, e1) };
        processed.pairs.insert(pair);

        // Lookup with same normalization should find it
        let lookup = if e1 < e2 { (e1, e2) } else { (e2, e1) };
        assert!(processed.pairs.contains(&lookup));
    }

    #[test]
    fn test_clear_processed_collisions() {
        let mut processed = ProcessedCollisions::default();
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        processed.pairs.insert((e1, e2));
        assert!(!processed.pairs.is_empty());

        processed.pairs.clear();
        assert!(processed.pairs.is_empty());
    }
}
