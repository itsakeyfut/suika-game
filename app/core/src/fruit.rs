//! Fruit type definitions and parameters
//!
//! This module defines the fruit evolution system with 11 fruit types,
//! from Cherry (smallest) to Watermelon (largest).

use bevy::prelude::*;
use crate::config::FruitsConfig;

/// Represents the 11 fruit types in the evolution chain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum FruitType {
    /// Cherry - smallest fruit (stage 1), spawnable
    Cherry,
    /// Strawberry - small fruit (stage 2), spawnable
    Strawberry,
    /// Grape - small fruit (stage 3), spawnable
    Grape,
    /// Dekopon - small-medium fruit (stage 4), spawnable
    Dekopon,
    /// Persimmon - medium fruit (stage 5), spawnable
    Persimmon,
    /// Apple - medium fruit (stage 6), merge only
    Apple,
    /// Pear - medium-large fruit (stage 7), merge only
    Pear,
    /// Peach - large fruit (stage 8), merge only
    Peach,
    /// Pineapple - large fruit (stage 9), merge only
    Pineapple,
    /// Melon - very large fruit (stage 10), merge only
    Melon,
    /// Watermelon - largest fruit (stage 11), merge only
    Watermelon,
}

/// Physical and game parameters for a fruit
#[derive(Debug, Clone, Copy)]
pub struct FruitParams {
    /// Collision radius in pixels
    pub radius: f32,
    /// Mass for physics simulation
    pub mass: f32,
    /// Restitution coefficient (bounciness, 0.0-1.0)
    pub restitution: f32,
    /// Friction coefficient (0.0-1.0)
    pub friction: f32,
    /// Points awarded when this fruit is created by merging
    pub points: u32,
}

impl FruitType {
    /// Returns the next evolution stage, or None if this is the final stage
    ///
    /// # Examples
    ///
    /// ```
    /// # use suika_game_core::fruit::FruitType;
    /// assert_eq!(FruitType::Cherry.next(), Some(FruitType::Strawberry));
    /// assert_eq!(FruitType::Watermelon.next(), None);
    /// ```
    pub fn next(&self) -> Option<FruitType> {
        match self {
            FruitType::Cherry => Some(FruitType::Strawberry),
            FruitType::Strawberry => Some(FruitType::Grape),
            FruitType::Grape => Some(FruitType::Dekopon),
            FruitType::Dekopon => Some(FruitType::Persimmon),
            FruitType::Persimmon => Some(FruitType::Apple),
            FruitType::Apple => Some(FruitType::Pear),
            FruitType::Pear => Some(FruitType::Peach),
            FruitType::Peach => Some(FruitType::Pineapple),
            FruitType::Pineapple => Some(FruitType::Melon),
            FruitType::Melon => Some(FruitType::Watermelon),
            FruitType::Watermelon => None, // Watermelon is the final stage
        }
    }

    /// Returns the physical and game parameters for this fruit type from RON config
    ///
    /// This method reads parameters from the externalized RON configuration,
    /// allowing hot-reload of fruit parameters during gameplay.
    ///
    /// # Panics
    ///
    /// Panics if the fruit config doesn't contain data for this fruit type.
    pub fn parameters_from_config(&self, config: &FruitsConfig) -> FruitParams {
        let index = *self as usize;
        let entry = &config.fruits[index];

        // Calculate mass from radius and mass_multiplier
        let mass = entry.radius * entry.radius * entry.mass_multiplier;

        FruitParams {
            radius: entry.radius,
            mass,
            restitution: entry.restitution,
            friction: entry.friction,
            points: entry.points,
        }
    }

    /// Returns the physical and game parameters for this fruit type (legacy hardcoded values)
    ///
    /// **Note**: This method uses hardcoded values and is kept for backward compatibility
    /// with tests. New code should use `parameters_from_config()` instead.
    ///
    /// Parameters are based on the fruit size progression:
    /// - Radius: 20px to 120px (increments of 10px)
    /// - Points: 10 to 10240 (doubles each stage)
    /// - Mass: calculated from radius squared
    /// - Restitution: decreases slightly with size (more bounce for small fruits)
    /// - Friction: constant across all fruits
    pub fn parameters(&self) -> FruitParams {
        let (radius, points) = match self {
            FruitType::Cherry => (20.0, 10),
            FruitType::Strawberry => (30.0, 20),
            FruitType::Grape => (40.0, 40),
            FruitType::Dekopon => (50.0, 80),
            FruitType::Persimmon => (60.0, 160),
            FruitType::Apple => (70.0, 320),
            FruitType::Pear => (80.0, 640),
            FruitType::Peach => (90.0, 1280),
            FruitType::Pineapple => (100.0, 2560),
            FruitType::Melon => (110.0, 5120),
            FruitType::Watermelon => (120.0, 10240),
        };

        // Mass is proportional to radius squared (approximating 2D area)
        let mass = radius * radius * 0.01;

        // Smaller fruits are slightly bouncier
        let restitution = match self {
            FruitType::Cherry | FruitType::Strawberry | FruitType::Grape => 0.3,
            FruitType::Dekopon | FruitType::Persimmon | FruitType::Apple | FruitType::Pear => 0.25,
            FruitType::Peach | FruitType::Pineapple | FruitType::Melon | FruitType::Watermelon => {
                0.2
            }
        };

        FruitParams {
            radius,
            mass,
            restitution,
            friction: 0.5, // Constant friction for all fruits
            points,
        }
    }

    /// Returns the array of fruits that can be spawned by the player
    ///
    /// Only the first 5 fruits (Cherry through Persimmon) can be spawned.
    /// Larger fruits can only be created through merging.
    pub fn spawnable_fruits() -> [FruitType; 5] {
        [
            FruitType::Cherry,
            FruitType::Strawberry,
            FruitType::Grape,
            FruitType::Dekopon,
            FruitType::Persimmon,
        ]
    }

    /// Returns a placeholder color for this fruit type
    ///
    /// These colors are used for rendering before custom sprites are implemented.
    /// Colors are chosen to be visually distinct and roughly match the fruit's
    /// real-world appearance.
    pub fn placeholder_color(&self) -> Color {
        match self {
            FruitType::Cherry => Color::srgb(0.8, 0.1, 0.2), // Red
            FruitType::Strawberry => Color::srgb(1.0, 0.2, 0.3), // Bright red
            FruitType::Grape => Color::srgb(0.5, 0.2, 0.6),  // Purple
            FruitType::Dekopon => Color::srgb(1.0, 0.5, 0.0), // Orange
            FruitType::Persimmon => Color::srgb(1.0, 0.4, 0.0), // Deep orange
            FruitType::Apple => Color::srgb(0.9, 0.1, 0.1),  // Red
            FruitType::Pear => Color::srgb(0.7, 0.9, 0.3),   // Yellow-green
            FruitType::Peach => Color::srgb(1.0, 0.6, 0.5),  // Pink-orange
            FruitType::Pineapple => Color::srgb(1.0, 0.8, 0.0), // Yellow
            FruitType::Melon => Color::srgb(0.5, 0.9, 0.4),  // Green
            FruitType::Watermelon => Color::srgb(0.2, 0.7, 0.2), // Dark green
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fruit_evolution_chain() {
        // Test complete evolution chain
        assert_eq!(FruitType::Cherry.next(), Some(FruitType::Strawberry));
        assert_eq!(FruitType::Strawberry.next(), Some(FruitType::Grape));
        assert_eq!(FruitType::Grape.next(), Some(FruitType::Dekopon));
        assert_eq!(FruitType::Dekopon.next(), Some(FruitType::Persimmon));
        assert_eq!(FruitType::Persimmon.next(), Some(FruitType::Apple));
        assert_eq!(FruitType::Apple.next(), Some(FruitType::Pear));
        assert_eq!(FruitType::Pear.next(), Some(FruitType::Peach));
        assert_eq!(FruitType::Peach.next(), Some(FruitType::Pineapple));
        assert_eq!(FruitType::Pineapple.next(), Some(FruitType::Melon));
        assert_eq!(FruitType::Melon.next(), Some(FruitType::Watermelon));
        assert_eq!(FruitType::Watermelon.next(), None);
    }

    #[test]
    fn test_spawnable_fruits() {
        let spawnable = FruitType::spawnable_fruits();
        assert_eq!(spawnable.len(), 5);
        assert_eq!(spawnable[0], FruitType::Cherry);
        assert_eq!(spawnable[1], FruitType::Strawberry);
        assert_eq!(spawnable[2], FruitType::Grape);
        assert_eq!(spawnable[3], FruitType::Dekopon);
        assert_eq!(spawnable[4], FruitType::Persimmon);
    }

    #[test]
    fn test_fruit_parameters_radius() {
        // Test that radii increase by 10px per stage
        assert_eq!(FruitType::Cherry.parameters().radius, 20.0);
        assert_eq!(FruitType::Strawberry.parameters().radius, 30.0);
        assert_eq!(FruitType::Grape.parameters().radius, 40.0);
        assert_eq!(FruitType::Dekopon.parameters().radius, 50.0);
        assert_eq!(FruitType::Persimmon.parameters().radius, 60.0);
        assert_eq!(FruitType::Apple.parameters().radius, 70.0);
        assert_eq!(FruitType::Pear.parameters().radius, 80.0);
        assert_eq!(FruitType::Peach.parameters().radius, 90.0);
        assert_eq!(FruitType::Pineapple.parameters().radius, 100.0);
        assert_eq!(FruitType::Melon.parameters().radius, 110.0);
        assert_eq!(FruitType::Watermelon.parameters().radius, 120.0);
    }

    #[test]
    fn test_fruit_parameters_points() {
        // Test that points double each stage
        assert_eq!(FruitType::Cherry.parameters().points, 10);
        assert_eq!(FruitType::Strawberry.parameters().points, 20);
        assert_eq!(FruitType::Grape.parameters().points, 40);
        assert_eq!(FruitType::Dekopon.parameters().points, 80);
        assert_eq!(FruitType::Persimmon.parameters().points, 160);
        assert_eq!(FruitType::Apple.parameters().points, 320);
        assert_eq!(FruitType::Pear.parameters().points, 640);
        assert_eq!(FruitType::Peach.parameters().points, 1280);
        assert_eq!(FruitType::Pineapple.parameters().points, 2560);
        assert_eq!(FruitType::Melon.parameters().points, 5120);
        assert_eq!(FruitType::Watermelon.parameters().points, 10240);
    }

    #[test]
    fn test_fruit_parameters_mass() {
        // Test that mass increases with radius squared
        let cherry_params = FruitType::Cherry.parameters();
        let watermelon_params = FruitType::Watermelon.parameters();

        // Mass should be proportional to radius squared
        assert_eq!(cherry_params.mass, 20.0 * 20.0 * 0.01);
        assert_eq!(watermelon_params.mass, 120.0 * 120.0 * 0.01);
        assert!(watermelon_params.mass > cherry_params.mass);
    }

    #[test]
    fn test_fruit_parameters_restitution() {
        // Test that small fruits are bouncier
        assert_eq!(FruitType::Cherry.parameters().restitution, 0.3);
        assert_eq!(FruitType::Apple.parameters().restitution, 0.25);
        assert_eq!(FruitType::Watermelon.parameters().restitution, 0.2);
    }

    #[test]
    fn test_fruit_parameters_friction() {
        // Test that friction is constant for all fruits
        assert_eq!(FruitType::Cherry.parameters().friction, 0.5);
        assert_eq!(FruitType::Watermelon.parameters().friction, 0.5);
    }

    #[test]
    fn test_placeholder_colors_are_distinct() {
        // Ensure all fruits have different colors
        let colors: Vec<_> = [
            FruitType::Cherry,
            FruitType::Strawberry,
            FruitType::Grape,
            FruitType::Dekopon,
            FruitType::Persimmon,
            FruitType::Apple,
            FruitType::Pear,
            FruitType::Peach,
            FruitType::Pineapple,
            FruitType::Melon,
            FruitType::Watermelon,
        ]
        .iter()
        .map(|f| f.placeholder_color())
        .collect();

        // Check that we have distinct colors (simple check - no two identical)
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(
                    colors[i], colors[j],
                    "Colors should be distinct for visual clarity"
                );
            }
        }
    }
}
