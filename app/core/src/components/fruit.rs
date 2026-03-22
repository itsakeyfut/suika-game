//! Fruit-related components

use bevy::prelude::*;

/// Fruit entity component
///
/// Marks an entity as a fruit in the game. The fruit's type is stored
/// separately using the `FruitType` component.
///
/// # Usage
///
/// Fruits should have both this component and a `FruitType` component:
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use suika_game_core::components::Fruit;
/// # use suika_game_core::fruit::FruitType;
/// fn spawn_fruit(mut commands: Commands) {
///     commands.spawn((
///         Fruit,
///         FruitType::Cherry,
///         // ... other components (Transform, Sprite, RigidBody, etc.)
///     ));
/// }
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Fruit;

/// Merge candidate marker component
///
/// Marks a fruit that is currently in the process of merging
/// with another fruit. Fruits with this component should not
/// participate in additional merges until the current merge completes.
///
/// This component is typically added when a collision is detected
/// and removed when the merge animation/transition finishes.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct MergeCandidate;

/// Fruit spawn state component
///
/// Tracks whether a fruit has been dropped and landed.
/// Used to prevent game over detection until fruits settle.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FruitSpawnState {
    /// Fruit is being held by player, not yet dropped
    #[default]
    Held,
    /// Fruit has been dropped and is falling
    Falling,
    /// Fruit has landed and settled
    Landed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fruit_component_default() {
        let fruit = Fruit::default();
        assert_eq!(format!("{:?}", fruit), "Fruit");
    }

    #[test]
    fn test_merge_candidate_component_default() {
        let merge = MergeCandidate::default();
        assert_eq!(format!("{:?}", merge), "MergeCandidate");
    }

    #[test]
    fn test_fruit_spawn_state_default() {
        let state = FruitSpawnState::default();
        assert_eq!(state, FruitSpawnState::Held);
    }

    #[test]
    fn test_fruit_spawn_state_transitions() {
        let held = FruitSpawnState::Held;
        let falling = FruitSpawnState::Falling;
        let landed = FruitSpawnState::Landed;

        assert_ne!(held, falling);
        assert_ne!(falling, landed);
        assert_ne!(held, landed);

        assert_eq!(held, FruitSpawnState::Held);
        assert_eq!(falling, FruitSpawnState::Falling);
        assert_eq!(landed, FruitSpawnState::Landed);
    }

    #[test]
    fn test_components_are_copy() {
        // Verify that marker components implement Copy
        let fruit = Fruit;
        let _fruit2 = fruit;
        let _fruit3 = fruit; // Should compile because Fruit is Copy
    }

    #[test]
    fn test_components_are_clone() {
        let fruit = Fruit;
        let _fruit2 = fruit.clone();

        let state = FruitSpawnState::Falling;
        let state2 = state.clone();
        assert_eq!(state, state2);
    }
}
