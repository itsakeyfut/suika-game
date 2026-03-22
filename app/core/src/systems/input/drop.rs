//! Fruit drop and landing detection systems

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[cfg(test)]
use bevy_rapier2d::rapier::geometry::CollisionEventFlags;

use crate::components::{BottomWall, Fruit, FruitSpawnState};
use crate::config::{FruitsConfig, FruitsConfigHandle, PhysicsConfig, PhysicsConfigHandle};
use crate::fruit::FruitType;

/// Detects when falling fruits land (collide with ground or other fruits)
///
/// Monitors collision events and transitions falling fruits to Landed state
/// when they collide with the bottom wall (ground) or other fruits.
/// Side walls are ignored - only ground collisions count as landing.
/// This triggers the spawning of the next fruit.
///
/// # System Parameters
///
/// - `collision_events`: Rapier collision message reader
/// - `fruit_query`: Query for fruits and their spawn state
/// - `bottom_wall_query`: Query for bottom wall entity (ground)
pub fn detect_fruit_landing(
    mut collision_events: MessageReader<CollisionEvent>,
    mut fruit_query: Query<&mut FruitSpawnState, With<Fruit>>,
    bottom_wall_query: Query<Entity, With<BottomWall>>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = event {
            // Collect entities to update (to avoid borrow checker issues)
            let mut entities_to_land = Vec::new();

            // Check if entity1 is a falling fruit
            if let Ok(spawn_state) = fruit_query.get(*entity1)
                && *spawn_state == FruitSpawnState::Falling
            {
                let hit_bottom_wall = bottom_wall_query.contains(*entity2);
                let hit_fruit = fruit_query.contains(*entity2);

                if hit_bottom_wall || hit_fruit {
                    entities_to_land.push((*entity1, hit_bottom_wall));
                }
            }

            // Check if entity2 is a falling fruit
            if let Ok(spawn_state) = fruit_query.get(*entity2)
                && *spawn_state == FruitSpawnState::Falling
            {
                let hit_bottom_wall = bottom_wall_query.contains(*entity1);
                let hit_fruit = fruit_query.contains(*entity1);

                if hit_bottom_wall || hit_fruit {
                    entities_to_land.push((*entity2, hit_bottom_wall));
                }
            }

            // Now update the states
            for (entity, hit_bottom_wall) in entities_to_land {
                if let Ok(mut spawn_state) = fruit_query.get_mut(entity) {
                    *spawn_state = FruitSpawnState::Landed;
                    info!(
                        "Fruit landed (collided with {})",
                        if hit_bottom_wall { "ground" } else { "fruit" }
                    );
                }
            }
        }
    }
}

/// Handles player input for dropping held fruits
///
/// Drops the currently held fruit when:
/// - Mouse left button is pressed
/// - Space key is pressed
///
/// After dropping, the fruit transitions from Held to Falling state,
/// becomes a dynamic rigid body, and gets physics properties.
///
/// # System Parameters
///
/// - `commands`: For adding/removing components
/// - `mouse_button`: Mouse button input state
/// - `keyboard`: Keyboard input state
/// - `held_fruits`: Query for held fruits to drop
#[allow(clippy::too_many_arguments)]
pub fn handle_fruit_drop_input(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut held_fruits: Query<(Entity, &FruitType, &mut FruitSpawnState), With<Fruit>>,
    fruits_config_handle: Res<FruitsConfigHandle>,
    fruits_config_assets: Res<Assets<FruitsConfig>>,
    physics_config_handle: Res<PhysicsConfigHandle>,
    physics_config_assets: Res<Assets<PhysicsConfig>>,
) {
    // Get the configs, return early if not loaded yet
    let Some(fruits_config) = fruits_config_assets.get(&fruits_config_handle.0) else {
        return;
    };
    let Some(physics_config) = physics_config_assets.get(&physics_config_handle.0) else {
        return;
    };

    if mouse_button.just_pressed(MouseButton::Left) || keyboard.just_pressed(KeyCode::Space) {
        for (entity, fruit_type, mut spawn_state) in held_fruits.iter_mut() {
            if *spawn_state == FruitSpawnState::Held {
                // Transition to Falling state
                *spawn_state = FruitSpawnState::Falling;

                let params = fruit_type.parameters_from_config(fruits_config);

                // Convert to dynamic rigid body with physics properties
                // Reset velocity to prevent diagonal falling due to kinematic movement
                commands.entity(entity).insert((
                    RigidBody::Dynamic,
                    Velocity::zero(), // Reset velocity to drop straight down
                    Restitution {
                        coefficient: params.restitution,
                        combine_rule: CoefficientCombineRule::Min, // Use minimum restitution in collisions
                    },
                    Friction::coefficient(params.friction),
                    ColliderMassProperties::Mass(params.mass),
                    Damping {
                        linear_damping: physics_config.fruit_linear_damping,
                        angular_damping: physics_config.fruit_angular_damping,
                    },
                    GravityScale(1.0),
                ));

                info!("Dropped fruit: {:?}", fruit_type);
            }
        }
    }
}
