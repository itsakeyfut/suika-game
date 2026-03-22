//! Spawn position movement system

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{Fruit, FruitSpawnState};
use crate::config::{FruitsConfig, FruitsConfigHandle, PhysicsConfig, PhysicsConfigHandle};
use crate::fruit::FruitType;

use super::resources::{InputMode, LastCursorPosition, SpawnPosition};

// ---------------------------------------------------------------------------
// Default values for RON-loaded parameters (fallbacks before configs are loaded)
// ---------------------------------------------------------------------------

/// Default keyboard move speed (px/s) — mirrors `physics.ron` `keyboard_move_speed`.
const DEFAULT_KEYBOARD_MOVE_SPEED: f32 = 300.0;
/// Default container width (px) — mirrors `physics.ron` `container_width`.
const DEFAULT_CONTAINER_WIDTH: f32 = 600.0;
/// Default fruit radius (px) — mirrors the Cherry entry radius in `fruits.ron`.
const DEFAULT_FRUIT_RADIUS: f32 = 20.0;

/// Updates the spawn position and held fruit position based on player input
///
/// Updates spawn position from multiple input sources:
/// 1. Arrow keys: Left/Right (←→) or A/D keys move horizontally
/// 2. Mouse cursor: Position follows the mouse X coordinate
///
/// The input mode automatically switches based on which input device was used most recently:
/// - Pressing arrow/AD keys switches to keyboard mode
/// - Moving the mouse cursor switches to mouse mode
///
/// Only fruits in the Held state are moved. Falling and Landed fruits are not affected.
/// The final position is clamped to stay within container boundaries.
///
/// # System Parameters
///
/// - `keyboard`: Keyboard input state
/// - `windows`: Query for the primary window (to get cursor position)
/// - `camera_query`: Query for camera and its transform (for world position conversion)
/// - `spawn_pos`: Mutable spawn position resource to update
/// - `input_mode`: Current input mode (keyboard or mouse)
/// - `held_fruits`: Query for held fruits to move (only Held state)
/// - `time`: Time resource for delta time (smooth movement with keys)
#[allow(clippy::too_many_arguments)]
pub fn update_spawn_position(
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut spawn_pos: ResMut<SpawnPosition>,
    mut input_mode: ResMut<InputMode>,
    mut last_cursor_pos: ResMut<LastCursorPosition>,
    mut held_fruits: Query<(&mut Transform, &FruitSpawnState, &FruitType), With<Fruit>>,
    time: Res<Time>,
    fruits_config_handle: Res<FruitsConfigHandle>,
    fruits_config_assets: Res<Assets<FruitsConfig>>,
    physics_config_handle: Res<PhysicsConfigHandle>,
    physics_config_assets: Res<Assets<PhysicsConfig>>,
) {
    // Get the configs
    let fruits_config = fruits_config_assets.get(&fruits_config_handle.0);
    let physics_config = physics_config_assets.get(&physics_config_handle.0);
    // Check for keyboard input and switch mode if detected
    let keyboard_input = keyboard.pressed(KeyCode::ArrowLeft)
        || keyboard.pressed(KeyCode::KeyA)
        || keyboard.pressed(KeyCode::ArrowRight)
        || keyboard.pressed(KeyCode::KeyD);

    if keyboard_input {
        *input_mode = InputMode::Keyboard;
    }

    // Handle keyboard movement (only in keyboard mode)
    if *input_mode == InputMode::Keyboard {
        let move_speed = physics_config
            .map(|c| c.keyboard_move_speed)
            .unwrap_or(DEFAULT_KEYBOARD_MOVE_SPEED);

        if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
            spawn_pos.x -= move_speed * time.delta_secs();
        }
        if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
            spawn_pos.x += move_speed * time.delta_secs();
        }
    }

    // Check for mouse movement and switch mode if detected
    if let Ok(window) = windows.single()
        && let Some(cursor_pos) = window.cursor_position()
        && let Ok((camera, camera_transform)) = camera_query.single()
    {
        // Convert viewport coordinates to world coordinates
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            // Detect actual mouse movement by comparing with last cursor position
            const MOUSE_MOVEMENT_THRESHOLD: f32 = 1.0; // pixels
            let mouse_moved = if let Some(last_pos) = last_cursor_pos.position {
                (world_pos - last_pos).length() > MOUSE_MOVEMENT_THRESHOLD
            } else {
                false // First frame, don't switch to mouse mode yet
            };

            if mouse_moved {
                *input_mode = InputMode::Mouse;
            }

            // Update last cursor position
            last_cursor_pos.position = Some(world_pos);

            // Handle mouse cursor position (only in mouse mode)
            if *input_mode == InputMode::Mouse {
                spawn_pos.x = world_pos.x;
            }
        }
    }

    // Get the held fruit's radius for proper clamping
    let held_fruit_radius = if let Some(config) = fruits_config {
        held_fruits
            .iter()
            .find(|(_, state, _)| **state == FruitSpawnState::Held)
            .map(|(_, _, fruit_type)| fruit_type.parameters_from_config(config).radius)
            .unwrap_or(DEFAULT_FRUIT_RADIUS)
    } else {
        DEFAULT_FRUIT_RADIUS
    };

    // Clamp spawn position within container bounds
    // Use the actual fruit radius to allow the fruit to touch the wall
    let container_width = physics_config
        .map(|c| c.container_width)
        .unwrap_or(DEFAULT_CONTAINER_WIDTH);
    let max_x = container_width / 2.0 - held_fruit_radius;
    spawn_pos.x = spawn_pos.x.clamp(-max_x, max_x);

    // Update ONLY held fruit position to match spawn position
    // Falling and Landed fruits are not affected
    for (mut transform, spawn_state, _) in held_fruits.iter_mut() {
        if *spawn_state == FruitSpawnState::Held {
            transform.translation.x = spawn_pos.x;
        }
    }
}
