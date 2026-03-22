//! Input-related resources

/// Resource tracking the current spawn position for the next fruit
///
/// This position represents the X coordinate where the fruit will be dropped.
/// It is updated based on mouse position and arrow key input, and clamped
/// to stay within the container bounds.
#[derive(bevy::prelude::Resource, Debug, Clone)]
pub struct SpawnPosition {
    /// X position in world coordinates where the fruit will spawn
    pub x: f32,
}

impl Default for SpawnPosition {
    fn default() -> Self {
        Self { x: 0.0 }
    }
}

/// Input mode for controlling fruit position
///
/// Tracks whether the player is currently using keyboard or mouse input.
/// The mode automatically switches based on which input device was used most recently.
#[derive(bevy::prelude::Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Player is using keyboard (arrow keys or A/D)
    /// Default so the held fruit starts at the container center (spawn_pos.x = 0)
    /// and only follows the mouse after the user actually moves the cursor.
    #[default]
    Keyboard,
    /// Player is using mouse cursor
    Mouse,
}

/// Tracks the last known cursor position to detect mouse movement
///
/// Used to distinguish between actual mouse movement and position changes
/// caused by keyboard input. Only switches to mouse mode when the cursor
/// itself moves.
#[derive(bevy::prelude::Resource, Debug, Clone, Default)]
pub struct LastCursorPosition {
    /// Last known cursor position in world coordinates
    pub position: Option<bevy::prelude::Vec2>,
}
