//! Game entity components
//!
//! This module defines all the components used in the game's ECS architecture.
//! Components represent data attached to entities and define their behavior.

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

/// Container wall marker component
///
/// Marks an entity as part of the game container (box).
/// Container entities include the left wall, right wall, and bottom wall.
///
/// These entities should have physics bodies (RigidBody::Fixed) and
/// colliders to keep fruits contained.
///
/// # Usage
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use suika_game_core::components::Container;
/// fn spawn_container(mut commands: Commands) {
///     commands.spawn((
///         Container,
///         // ... Transform, RigidBody::Fixed, Collider, etc.
///     ));
/// }
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Container;

/// Bottom wall marker component
///
/// Marks the bottom wall of the container specifically.
/// Used to distinguish the ground from side walls for fruit landing detection.
/// When a falling fruit collides with this, it's considered "landed".
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct BottomWall;

/// Boundary line marker component
///
/// Marks the entity representing the game over boundary line.
/// When fruits stay above this line for too long, the game ends.
///
/// This is typically a visual indicator (a line sprite) and a sensor
/// collider for detecting fruits above the line.
///
/// # Usage
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use suika_game_core::components::BoundaryLine;
/// fn spawn_boundary_line(mut commands: Commands) {
///     commands.spawn((
///         BoundaryLine,
///         // ... Transform, Sprite, Sensor collider, etc.
///     ));
/// }
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct BoundaryLine;

/// Dropping fruit marker component
///
/// Marks a fruit that is currently being controlled by the player
/// before being dropped. This fruit follows the cursor/input position
/// and has not yet been released into the physics simulation.
///
/// Once dropped, this component is removed and the fruit becomes
/// a normal physics-simulated fruit.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Dropping;

/// Next fruit preview marker component
///
/// Marks the entity displaying the preview of the next fruit
/// that will be spawned. This is typically shown in the UI.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct NextFruitPreview;

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
    fn test_container_component_default() {
        let container = Container::default();
        assert_eq!(format!("{:?}", container), "Container");
    }

    #[test]
    fn test_boundary_line_component_default() {
        let boundary = BoundaryLine::default();
        assert_eq!(format!("{:?}", boundary), "BoundaryLine");
    }

    #[test]
    fn test_dropping_component_default() {
        let dropping = Dropping::default();
        assert_eq!(format!("{:?}", dropping), "Dropping");
    }

    #[test]
    fn test_next_fruit_preview_component_default() {
        let preview = NextFruitPreview::default();
        assert_eq!(format!("{:?}", preview), "NextFruitPreview");
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

        let container = Container;
        let _container2 = container;
        let _container3 = container;
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
