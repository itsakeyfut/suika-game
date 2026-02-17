//! Flash visual effect systems
//!
//! Two types of flash effects on fruit merge:
//! - **Local flash**: a bright circle at the merge point that expands and fades
//! - **Screen flash**: a full-screen overlay for large-fruit merges (Pineapple+)

use bevy::prelude::*;

use crate::config::{FlashConfig, FlashConfigHandle};
use crate::events::FruitMergeEvent;

// --- Constants ---

/// Duration of the local flash in seconds
pub const LOCAL_FLASH_DURATION: f32 = 0.3;
/// Starting alpha for the local flash sprite
pub const LOCAL_FLASH_INITIAL_ALPHA: f32 = 0.6;
/// Size multiplier: flash sprite starts at `fruit_radius * multiplier`
pub const LOCAL_FLASH_SIZE_MULTIPLIER: f32 = 2.5;

/// Duration of the screen flash in seconds
pub const SCREEN_FLASH_DURATION: f32 = 0.25;
/// Starting alpha for the screen flash overlay
pub const SCREEN_FLASH_INITIAL_ALPHA: f32 = 0.35;
/// Minimum fruit index (0-based) that triggers a screen flash.
/// Index 8 = Pineapple (the 9th fruit in the evolution chain).
pub const SCREEN_FLASH_MIN_INDEX: usize = 8;

// --- Components ---

/// Local flash animation component
///
/// A sprite at the merge position that expands and fades out.
/// Uses the merged fruit's placeholder color.
#[derive(Component, Debug)]
pub struct LocalFlashAnimation {
    /// Elapsed time in seconds
    pub elapsed: f32,
    /// Total duration in seconds
    pub duration: f32,
    /// Initial size of the sprite (will expand over time)
    pub initial_size: Vec2,
    /// Color (derived from the merged fruit)
    pub color: Color,
}

/// Screen flash animation component
///
/// A large full-screen overlay sprite at Z=999 that fades out quickly.
/// Only triggered for high-index fruit merges (Pineapple and above).
#[derive(Component, Debug)]
pub struct ScreenFlashAnimation {
    /// Elapsed time in seconds
    pub elapsed: f32,
    /// Total duration in seconds
    pub duration: f32,
}

// --- Systems ---

/// Spawns flash effects on fruit merge events
///
/// For every merge:
/// - Spawns a local flash at the merge position (all merges)
///
/// For large-fruit merges (index >= `SCREEN_FLASH_MIN_INDEX`):
/// - Also spawns a full-screen flash overlay
pub fn spawn_merge_flash(
    mut commands: Commands,
    mut merge_events: MessageReader<FruitMergeEvent>,
    fruits_config_handle: Option<Res<crate::config::FruitsConfigHandle>>,
    fruits_config_assets: Option<Res<Assets<crate::config::FruitsConfig>>>,
    flash_config_handle: Option<Res<FlashConfigHandle>>,
    flash_config_assets: Option<Res<Assets<FlashConfig>>>,
) {
    let fruit_config = fruits_config_handle
        .as_ref()
        .zip(fruits_config_assets.as_ref())
        .and_then(|(h, a)| a.get(&h.0));
    let flash_cfg = flash_config_handle
        .as_ref()
        .zip(flash_config_assets.as_ref())
        .and_then(|(h, a)| a.get(&h.0));

    let local_duration = flash_cfg
        .map(|c| c.local_duration)
        .unwrap_or(LOCAL_FLASH_DURATION);
    let local_initial_alpha = flash_cfg
        .map(|c| c.local_initial_alpha)
        .unwrap_or(LOCAL_FLASH_INITIAL_ALPHA);
    let local_size_multiplier = flash_cfg
        .map(|c| c.local_size_multiplier)
        .unwrap_or(LOCAL_FLASH_SIZE_MULTIPLIER);
    let screen_duration = flash_cfg
        .map(|c| c.screen_duration)
        .unwrap_or(SCREEN_FLASH_DURATION);
    let screen_initial_alpha = flash_cfg
        .map(|c| c.screen_initial_alpha)
        .unwrap_or(SCREEN_FLASH_INITIAL_ALPHA);
    let screen_flash_min_index = flash_cfg
        .map(|c| c.screen_flash_min_index)
        .unwrap_or(SCREEN_FLASH_MIN_INDEX);

    for event in merge_events.read() {
        let color = event.fruit_type.placeholder_color();

        // Determine initial flash size from fruit radius (fallback if config not loaded)
        let fruit_radius = fruit_config
            .and_then(|c| event.fruit_type.try_parameters_from_config(c))
            .map(|p| p.radius)
            .unwrap_or(30.0);

        let initial_size = Vec2::splat(fruit_radius * local_size_multiplier);

        // Spawn local flash at Z=5 (above fruits but below UI)
        // TODO: 将来的に Material2d + WGSL フラグメントシェーダーで
        //       放射状グラデーション（中心が明るく、外に向かってフェード）に変更する
        commands.spawn((
            LocalFlashAnimation {
                elapsed: 0.0,
                duration: local_duration,
                initial_size,
                color,
            },
            Sprite {
                color: color.with_alpha(local_initial_alpha),
                custom_size: Some(initial_size),
                ..default()
            },
            Transform::from_translation(event.position.extend(5.0)),
        ));

        // Screen flash for large-fruit merges only
        let fruit_index = event.fruit_type as usize;
        if fruit_index >= screen_flash_min_index {
            commands.spawn((
                ScreenFlashAnimation {
                    elapsed: 0.0,
                    duration: screen_duration,
                },
                Sprite {
                    color: color.with_alpha(screen_initial_alpha),
                    // Covers the full screen — large enough for any camera zoom
                    custom_size: Some(Vec2::splat(10_000.0)),
                    ..default()
                },
                // Z=999 puts this above everything else
                Transform::from_translation(Vec3::new(0.0, 0.0, 999.0)),
            ));
        }
    }
}

/// Animates local flash: expands the sprite and fades out the alpha
///
/// Each frame:
/// 1. Increments elapsed
/// 2. Fades alpha from `LOCAL_FLASH_INITIAL_ALPHA` → 0
/// 3. Expands sprite size slightly
/// 4. Despawns when duration is reached
pub fn animate_local_flash(
    mut commands: Commands,
    mut flashes: Query<(
        Entity,
        &mut LocalFlashAnimation,
        &mut Sprite,
        &mut Transform,
    )>,
    time: Res<Time>,
    flash_config_handle: Option<Res<FlashConfigHandle>>,
    flash_config_assets: Option<Res<Assets<FlashConfig>>>,
) {
    let flash_cfg = flash_config_handle
        .as_ref()
        .zip(flash_config_assets.as_ref())
        .and_then(|(h, a)| a.get(&h.0));
    let initial_alpha = flash_cfg
        .map(|c| c.local_initial_alpha)
        .unwrap_or(LOCAL_FLASH_INITIAL_ALPHA);

    for (entity, mut flash, mut sprite, mut transform) in flashes.iter_mut() {
        flash.elapsed += time.delta_secs();

        if flash.elapsed >= flash.duration {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = (flash.elapsed / flash.duration).clamp(0.0, 1.0);
        let alpha = initial_alpha * (1.0 - progress);
        // Expand to 1.5× the initial size over the duration
        let size = flash.initial_size * (1.0 + progress * 0.5);

        sprite.color = flash.color.with_alpha(alpha);
        sprite.custom_size = Some(size);
        transform.scale = Vec3::ONE; // size is managed via custom_size, not scale
    }
}

/// Animates screen flash: fades out the alpha overlay
///
/// Each frame:
/// 1. Increments elapsed
/// 2. Fades alpha from `SCREEN_FLASH_INITIAL_ALPHA` → 0
/// 3. Despawns when duration is reached
pub fn animate_screen_flash(
    mut commands: Commands,
    mut flashes: Query<(Entity, &mut ScreenFlashAnimation, &mut Sprite)>,
    time: Res<Time>,
    flash_config_handle: Option<Res<FlashConfigHandle>>,
    flash_config_assets: Option<Res<Assets<FlashConfig>>>,
) {
    let flash_cfg = flash_config_handle
        .as_ref()
        .zip(flash_config_assets.as_ref())
        .and_then(|(h, a)| a.get(&h.0));
    let initial_alpha = flash_cfg
        .map(|c| c.screen_initial_alpha)
        .unwrap_or(SCREEN_FLASH_INITIAL_ALPHA);

    for (entity, mut flash, mut sprite) in flashes.iter_mut() {
        flash.elapsed += time.delta_secs();

        if flash.elapsed >= flash.duration {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = (flash.elapsed / flash.duration).clamp(0.0, 1.0);
        let alpha = initial_alpha * (1.0 - progress);
        let base_color = sprite.color.with_alpha(1.0); // preserve hue, update alpha
        sprite.color = base_color.with_alpha(alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fruit::FruitType;

    #[test]
    fn test_local_flash_alpha_reaches_zero_at_duration() {
        let flash = LocalFlashAnimation {
            elapsed: LOCAL_FLASH_DURATION,
            duration: LOCAL_FLASH_DURATION,
            initial_size: Vec2::splat(50.0),
            color: Color::WHITE,
        };
        let progress = (flash.elapsed / flash.duration).clamp(0.0, 1.0);
        let alpha = LOCAL_FLASH_INITIAL_ALPHA * (1.0 - progress);
        assert!(
            alpha.abs() < f32::EPSILON,
            "Alpha should reach 0 at end of duration"
        );
    }

    #[test]
    fn test_screen_flash_alpha_reaches_zero_at_duration() {
        let flash = ScreenFlashAnimation {
            elapsed: SCREEN_FLASH_DURATION,
            duration: SCREEN_FLASH_DURATION,
        };
        let progress = (flash.elapsed / flash.duration).clamp(0.0, 1.0);
        let alpha = SCREEN_FLASH_INITIAL_ALPHA * (1.0 - progress);
        assert!(
            alpha.abs() < f32::EPSILON,
            "Screen flash alpha should reach 0 at end of duration"
        );
    }

    #[test]
    fn test_screen_flash_triggered_for_pineapple_and_above() {
        // Pineapple is index 8, Melon=9, Watermelon=10
        let large_fruits = [
            FruitType::Pineapple,
            FruitType::Melon,
            FruitType::Watermelon,
        ];
        for fruit in large_fruits {
            assert!(
                fruit as usize >= SCREEN_FLASH_MIN_INDEX,
                "{fruit:?} (index {}) should trigger screen flash (min index {SCREEN_FLASH_MIN_INDEX})",
                fruit as usize
            );
        }
    }

    #[test]
    fn test_screen_flash_not_triggered_for_small_fruits() {
        // Cherry=0 through Peach=7 should NOT trigger screen flash
        let small_fruits = [
            FruitType::Cherry,
            FruitType::Strawberry,
            FruitType::Grape,
            FruitType::Dekopon,
            FruitType::Persimmon,
            FruitType::Apple,
            FruitType::Pear,
            FruitType::Peach,
        ];
        for fruit in small_fruits {
            assert!(
                (fruit as usize) < SCREEN_FLASH_MIN_INDEX,
                "{fruit:?} (index {}) should NOT trigger screen flash",
                fruit as usize
            );
        }
    }

    #[test]
    fn test_animate_local_flash_despawns_when_done() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, animate_local_flash);

        let flash = LocalFlashAnimation {
            elapsed: LOCAL_FLASH_DURATION, // Already done
            duration: LOCAL_FLASH_DURATION,
            initial_size: Vec2::splat(50.0),
            color: Color::WHITE,
        };

        let entity = app
            .world_mut()
            .spawn((flash, Sprite::default(), Transform::default()))
            .id();

        app.update();

        assert!(
            app.world().get_entity(entity).is_err(),
            "Local flash entity should be despawned when duration is reached"
        );
    }

    #[test]
    fn test_animate_screen_flash_despawns_when_done() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, animate_screen_flash);

        let flash = ScreenFlashAnimation {
            elapsed: SCREEN_FLASH_DURATION, // Already done
            duration: SCREEN_FLASH_DURATION,
        };

        let entity = app
            .world_mut()
            .spawn((flash, Sprite::default(), Transform::default()))
            .id();

        app.update();

        assert!(
            app.world().get_entity(entity).is_err(),
            "Screen flash entity should be despawned when duration is reached"
        );
    }
}
