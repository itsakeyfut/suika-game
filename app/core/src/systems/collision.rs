//! Fruit collision detection system
//!
//! This module polls Rapier2D contact pairs every frame to detect when two fruits
//! of the same type are touching, triggering the merge system via `FruitMergeEvent`.
//!
//! # Why polling instead of `CollisionEvent::Started`
//!
//! `CollisionEvent::Started` fires only once when contact *begins*. In a densely
//! packed container, two same-type fruits can be pressed into each other gradually
//! (e.g. squeezed by a third fruit landing on top) without ever generating a new
//! `Started` event. The merge would be permanently missed.
//!
//! By polling `rapier_context.simulation.contact_pairs()` each frame we catch
//! all *currently active* contacts, so no merge opportunity is ever skipped.

use std::collections::HashSet;

use bevy::prelude::*;
use bevy_rapier2d::prelude::ReadRapierContext;

use crate::components::{Fruit, FruitSpawnState, MergeCandidate};
use crate::events::FruitMergeEvent;
use crate::fruit::FruitType;

/// Resource tracking entity pairs processed this frame to prevent duplicate merge events
///
/// Stores normalized (min, max) entity pairs to ensure each collision is only
/// processed once per frame, even if multiple contact pairs fire for the same pair.
#[derive(Resource, Default)]
pub struct ProcessedCollisions {
    pub pairs: HashSet<(Entity, Entity)>,
}

/// Detects active contacts between fruits of the same type and fires `FruitMergeEvent`
///
/// Each frame this system polls `rapier_context.simulation.contact_pairs()` and
/// checks whether both entities are fruits with the same `FruitType`. When a valid
/// merge is detected, both fruits are marked with `MergeCandidate` and a
/// `FruitMergeEvent` is sent.
///
/// # Why polling
///
/// Using `CollisionEvent::Started` misses merges that occur when fruits are slowly
/// pressed together (no new contact-start event fires). Polling active contact pairs
/// every frame ensures every touching same-type pair is eventually merged.
///
/// # Deduplication
///
/// Entity pairs are normalized (min entity, max entity) and tracked in
/// `ProcessedCollisions` to prevent duplicate events within a single frame.
/// A frame-local `HashSet<Entity>` additionally ensures that a single entity
/// cannot be claimed by two different merge pairs in the same frame (which
/// would cause double scoring). Fruits already marked as `MergeCandidate`
/// are skipped.
///
/// # Conditions for a merge
///
/// - The contact pair must have at least one active contact point
/// - Both entities must have the `Fruit` component
/// - Neither entity may already be a `MergeCandidate`
/// - Neither may be in `FruitSpawnState::Held` state (still aimed by the player)
/// - Both must have the same `FruitType`
#[allow(clippy::type_complexity)]
pub fn detect_fruit_contact(
    mut commands: Commands,
    rapier_context: ReadRapierContext,
    fruit_query: Query<(&FruitType, &FruitSpawnState), (With<Fruit>, Without<MergeCandidate>)>,
    transform_query: Query<&Transform>,
    mut merge_events: MessageWriter<FruitMergeEvent>,
    mut processed: ResMut<ProcessedCollisions>,
) {
    let Ok(ctx) = rapier_context.single() else {
        return;
    };

    // Tracks individual entities already claimed for a merge this frame.
    // MergeCandidate is inserted via deferred Commands, so Without<MergeCandidate>
    // won't filter out a just-claimed entity until the next command flush.
    // This local set prevents a single entity from appearing in two merge pairs
    // (e.g. A-B and A-C) within one detect_fruit_contact run, which would
    // otherwise cause double scoring.
    let mut claimed: HashSet<Entity> = HashSet::new();

    for contact_pair in ctx
        .simulation
        .contact_pairs(ctx.colliders, ctx.rigidbody_set)
    {
        // Only consider pairs with at least one active contact point
        if !contact_pair.has_any_active_contact() {
            continue;
        }

        let (Some(entity1), Some(entity2)) = (contact_pair.collider1(), contact_pair.collider2())
        else {
            continue;
        };

        // Normalize pair to prevent duplicate processing (smaller entity first)
        let pair = if entity1 < entity2 {
            (entity1, entity2)
        } else {
            (entity2, entity1)
        };

        if processed.pairs.contains(&pair) {
            continue;
        }

        // Skip entities already claimed for a merge this frame
        if claimed.contains(&entity1) || claimed.contains(&entity2) {
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
        let pos1 = match transform_query.get(entity1) {
            Ok(t) => t.translation.truncate(),
            Err(_) => {
                warn!(
                    "detect_fruit_contact: entity1 {:?} has no Transform",
                    entity1
                );
                Vec2::ZERO
            }
        };
        let pos2 = match transform_query.get(entity2) {
            Ok(t) => t.translation.truncate(),
            Err(_) => {
                warn!(
                    "detect_fruit_contact: entity2 {:?} has no Transform",
                    entity2
                );
                Vec2::ZERO
            }
        };
        let position = (pos1 + pos2) / 2.0;

        // Mark both fruits as merge candidates to prevent further collision processing
        commands.entity(entity1).insert(MergeCandidate);
        commands.entity(entity2).insert(MergeCandidate);

        processed.pairs.insert(pair);
        claimed.insert(entity1);
        claimed.insert(entity2);

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
