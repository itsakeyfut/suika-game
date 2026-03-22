//! UI-related components

use bevy::prelude::*;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
