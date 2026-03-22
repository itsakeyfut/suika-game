//! Container wall marker components

use bevy::prelude::*;

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

/// Left wall marker component
///
/// Marks the left wall of the container.
/// Used to identify and update the left wall during hot-reload.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct LeftWall;

/// Right wall marker component
///
/// Marks the right wall of the container.
/// Used to identify and update the right wall during hot-reload.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct RightWall;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_component_default() {
        let container = Container::default();
        assert_eq!(format!("{:?}", container), "Container");
    }

    #[test]
    fn test_components_are_copy() {
        let container = Container;
        let _container2 = container;
        let _container3 = container;
    }
}
