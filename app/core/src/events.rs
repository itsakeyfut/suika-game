//! Game events for event-driven architecture
//!
//! This module defines custom Bevy events used to decouple game systems
//! and enable reactive programming patterns.
//!
//! Events are sent by one system and can be consumed by multiple listeners,
//! allowing for clean separation of concerns between different game mechanics.

use bevy::prelude::*;

use crate::fruit::FruitType;

/// Event triggered when two fruits of the same type collide and merge
///
/// This event is sent by the collision detection system when it detects
/// two identical fruits touching. Multiple systems can listen to this event:
/// - Merge handler: Despawns old fruits and spawns the next evolution
/// - Score system: Awards points and manages combos
/// - Effects system: Plays merge animation and particles
/// - Audio system: Plays merge sound effect
///
/// # Fields
///
/// * `entity1` - First fruit entity involved in the merge
/// * `entity2` - Second fruit entity involved in the merge
/// * `fruit_type` - Type of the fruits being merged (both are the same type)
/// * `position` - World position where the merge occurs (midpoint between the two fruits)
///
/// # Example
///
/// ```ignore
/// use bevy::prelude::*;
/// use suika_game_core::events::FruitMergeEvent;
/// use suika_game_core::fruit::FruitType;
///
/// fn handle_merge(mut merge_events: MessageReader<FruitMergeEvent>) {
///     for event in merge_events.read() {
///         println!("Merging {:?} at {:?}", event.fruit_type, event.position);
///         // Handle merge logic...
///     }
/// }
/// ```
#[derive(Message, Debug, Clone)]
pub struct FruitMergeEvent {
    /// First fruit entity to be merged
    pub entity1: Entity,

    /// Second fruit entity to be merged
    pub entity2: Entity,

    /// Type of fruits being merged (both fruits have the same type)
    pub fruit_type: FruitType,

    /// World position where the merge occurs (typically the midpoint)
    pub position: Vec2,
}
