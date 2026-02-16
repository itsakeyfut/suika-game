//! Fruit type definitions and parameters
//!
//! This module defines the fruit evolution system with 11 fruit types,
//! from Cherry (smallest) to Watermelon (largest).

use crate::config::FruitsConfig;
use bevy::prelude::*;

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
    /// Returns `None` if the config doesn't contain an entry for this fruit type.
    /// This can happen during hot-reload if the RON file is temporarily invalid.
    pub fn try_parameters_from_config(&self, config: &FruitsConfig) -> Option<FruitParams> {
        let index = *self as usize;
        let entry = config.fruits.get(index)?;

        // Calculate mass from radius and mass_multiplier
        let mass = entry.radius * entry.radius * entry.mass_multiplier;

        Some(FruitParams {
            radius: entry.radius,
            mass,
            restitution: entry.restitution,
            friction: entry.friction,
            points: entry.points,
        })
    }

    /// Returns the physical and game parameters for this fruit type from RON config
    ///
    /// # Panics
    ///
    /// Panics if the fruit config doesn't contain data for this fruit type.
    /// Use `try_parameters_from_config` for graceful error handling during hot-reload.
    pub fn parameters_from_config(&self, config: &FruitsConfig) -> FruitParams {
        self.try_parameters_from_config(config).unwrap_or_else(|| {
            panic!(
                "FruitsConfig missing entry for {:?} (index {}). Expected at least {} entries.",
                self,
                *self as usize,
                (*self as usize) + 1
            )
        })
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
