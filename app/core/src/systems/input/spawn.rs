//! Held fruit spawning system

use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_rapier2d::prelude::*;

use crate::components::{Fruit, FruitSpawnState};
use crate::config::{
    FruitsConfig, FruitsConfigHandle, GameRulesConfig, GameRulesConfigHandle, PhysicsConfig,
    PhysicsConfigHandle,
};
use crate::resources::{CircleTexture, FruitSprites, NextFruitType};

use super::resources::SpawnPosition;

/// Default spawnable fruit count — mirrors `game_rules.ron` `spawnable_fruit_count`.
const DEFAULT_SPAWNABLE_FRUIT_COUNT: usize = 5;

/// Spawns a new held fruit if none exists
///
/// This system runs once at startup and after each fruit lands.
/// It creates a fruit in the Held state that hovers at the top of the container.
///
/// **Important**: Will NOT spawn if there's a falling fruit (waiting for it to land first).
///
/// After spawning the fruit, the next fruit type is randomized for the preview display.
///
/// # System Parameters
///
/// - `commands`: For spawning new fruit entities
/// - `next_fruit`: The type of fruit to spawn (mutable to randomize after spawn)
/// - `spawn_pos`: Current spawn position (X coordinate)
/// - `fruit_states`: Query to check fruit spawn states
#[allow(clippy::too_many_arguments)]
pub fn spawn_held_fruit(
    mut commands: Commands,
    mut next_fruit: ResMut<NextFruitType>,
    mut spawn_pos: ResMut<SpawnPosition>,
    fruit_states: Query<&FruitSpawnState, With<Fruit>>,
    fruits_config_handle: Res<FruitsConfigHandle>,
    fruits_config_assets: Res<Assets<FruitsConfig>>,
    physics_config_handle: Res<PhysicsConfigHandle>,
    physics_config_assets: Res<Assets<PhysicsConfig>>,
    rules_config_handle: Option<Res<GameRulesConfigHandle>>,
    rules_config_assets: Option<Res<Assets<GameRulesConfig>>>,
    circle_texture: Res<CircleTexture>,
    fruit_sprites: Option<Res<FruitSprites>>,
) {
    // Get the configs, return early if not loaded yet
    let Some(fruits_config) = fruits_config_assets.get(&fruits_config_handle.0) else {
        warn!("Fruits config not loaded yet, cannot spawn fruit");
        return;
    };
    let Some(physics_config) = physics_config_assets.get(&physics_config_handle.0) else {
        warn!("Physics config not loaded yet, cannot spawn fruit");
        return;
    };

    // Spawnable count from game rules (default to 5 if config not yet loaded)
    let spawnable_count = rules_config_handle
        .as_ref()
        .zip(rules_config_assets.as_ref())
        .and_then(|(h, a)| a.get(&h.0))
        .map(|r| r.spawnable_fruit_count)
        .unwrap_or(DEFAULT_SPAWNABLE_FRUIT_COUNT);

    // Count fruits by state in a single iteration
    let (held_count, falling_count, landed_count) =
        fruit_states
            .iter()
            .fold((0u32, 0u32, 0u32), |(h, f, l), state| match *state {
                FruitSpawnState::Held => (h + 1, f, l),
                FruitSpawnState::Falling => (h, f + 1, l),
                FruitSpawnState::Landed => (h, f, l + 1),
            });

    if held_count > 0 || falling_count > 0 || landed_count > 0 {
        trace!(
            "Fruit states - Held: {}, Falling: {}, Landed: {}",
            held_count, falling_count, landed_count
        );
    }

    // Only spawn if:
    // 1. No fruit in Held state
    // 2. No fruit in Falling state (wait for it to land first)
    if held_count == 0 && falling_count == 0 {
        // On the very first spawn (nothing exists yet), initialize X from config
        if held_count == 0 && falling_count == 0 && landed_count == 0 {
            spawn_pos.x = physics_config
                .fruit_spawn_x_offset
                .max(-physics_config.container_width / 2.0)
                .min(physics_config.container_width / 2.0);
        }

        let spawn_y = physics_config.container_height / 2.0 - physics_config.fruit_spawn_y_offset;
        let params = next_fruit.get().parameters_from_config(fruits_config);

        commands.spawn((
            // Fruit marker and type
            Fruit,
            next_fruit.get(),
            FruitSpawnState::Held,
            // Sprite: use the real asset when available, otherwise a tinted circle.
            {
                let (image, color) = fruit_sprites
                    .as_ref()
                    .map(|s| s.resolve(next_fruit.get(), circle_texture.0.clone()))
                    .unwrap_or_else(|| {
                        (
                            circle_texture.0.clone(),
                            next_fruit.get().placeholder_color(),
                        )
                    });
                Sprite {
                    image,
                    color,
                    custom_size: Some(Vec2::splat(params.radius * 2.0 * params.sprite_scale)),
                    ..default()
                }
            },
            // Sprite anchor offset (horizontal + vertical) for fine-tuned alignment.
            Anchor(Vec2::new(params.sprite_anchor_x, params.sprite_anchor_y)),
            Transform::from_xyz(spawn_pos.x, spawn_y, 0.0),
            // Kinematic body (no gravity, manually controlled)
            RigidBody::KinematicPositionBased,
            // Collision shape (for preview, not for physics yet)
            Collider::ball(params.radius),
            // Enable collision events (required for Rapier)
            ActiveEvents::COLLISION_EVENTS,
            // Disable sleeping to allow continuous physics interactions
            Sleeping::disabled(),
        ));

        info!("Spawned held fruit: {:?}", next_fruit.get());

        // Randomize next fruit type for preview display
        // This ensures the preview shows the NEXT fruit, not the current held fruit
        next_fruit.randomize(spawnable_count);
    }
}
