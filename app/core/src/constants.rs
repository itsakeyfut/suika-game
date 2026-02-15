//! Game constants and configuration values
//!
//! This module contains all constant values used throughout the game,
//! organized into logical groups: physics, combo system, game over rules,
//! and fruit parameters.

/// Physics and container constants
pub mod physics {
    /// Gravity acceleration in pixels per second squared
    ///
    /// This value controls how fast fruits fall. Negative value means downward.
    /// Range: -800.0 (slow) to -1200.0 (fast)
    pub const GRAVITY: f32 = -980.0;

    /// Container (box) width in pixels
    ///
    /// This is the playable area width where fruits can be dropped.
    pub const CONTAINER_WIDTH: f32 = 600.0;

    /// Container (box) height in pixels
    ///
    /// This is the playable area height from bottom to top.
    pub const CONTAINER_HEIGHT: f32 = 800.0;

    /// Wall thickness in pixels
    ///
    /// The thickness of the left, right, and bottom walls of the container.
    pub const WALL_THICKNESS: f32 = 20.0;

    /// Boundary line Y position in pixels
    ///
    /// This is the "dead line" - if fruits stay above this line for
    /// GAME_OVER_TIMER seconds, the game is over.
    /// Position is relative to the container bottom.
    pub const BOUNDARY_LINE_Y: f32 = 300.0;
}

/// Combo system constants
pub mod combo {
    /// Combo window duration in seconds
    ///
    /// If another merge occurs within this time window after the previous merge,
    /// it counts as a combo. The combo counter resets after this time expires.
    pub const COMBO_WINDOW: f32 = 2.0;

    /// Maximum combo count
    ///
    /// The combo counter will not exceed this value.
    pub const COMBO_MAX: u32 = 10;

    /// Calculates the combo bonus multiplier based on combo count
    ///
    /// # Arguments
    ///
    /// * `combo` - Current combo count (number of consecutive merges)
    ///
    /// # Returns
    ///
    /// Multiplier to apply to the base score (1.0 = no bonus)
    ///
    /// # Combo Bonus Table
    ///
    /// - 0-1 combo: 1.0x (no bonus)
    /// - 2 combo: 1.1x (+10%)
    /// - 3 combo: 1.2x (+20%)
    /// - 4 combo: 1.3x (+30%)
    /// - 5+ combo: 1.5x (+50%)
    ///
    /// # Examples
    ///
    /// ```
    /// # use suika_game_core::constants::combo::bonus_multiplier;
    /// assert_eq!(bonus_multiplier(1), 1.0);   // No bonus
    /// assert_eq!(bonus_multiplier(2), 1.1);   // +10%
    /// assert_eq!(bonus_multiplier(5), 1.5);   // +50%
    /// assert_eq!(bonus_multiplier(10), 1.5);  // +50% (capped)
    /// ```
    pub fn bonus_multiplier(combo: u32) -> f32 {
        match combo {
            0..=1 => 1.0, // No bonus
            2 => 1.1,     // +10%
            3 => 1.2,     // +20%
            4 => 1.3,     // +30%
            5.. => 1.5,   // +50% (max bonus)
        }
    }
}

/// Game over rules
pub mod game_over {
    /// Game over timer duration in seconds
    ///
    /// If a fruit stays above the boundary line for this duration,
    /// the game ends. This prevents instant game over and gives the
    /// player a chance to recover.
    pub const TIMER: f32 = 3.0;
}

/// Persistence and storage constants
pub mod storage {
    /// Directory where save files are stored
    ///
    /// This directory will be created if it doesn't exist when
    /// saving game data (e.g., highscore).
    pub const SAVE_DIR: &str = "save";
}

/// Fruit parameters and calculations
pub mod fruit {
    /// Fruit radii in pixels for all 11 fruit types
    ///
    /// Index corresponds to FruitType enum order:
    /// 0: Cherry, 1: Strawberry, 2: Grape, 3: Dekopon, 4: Persimmon,
    /// 5: Apple, 6: Pear, 7: Peach, 8: Pineapple, 9: Melon, 10: Watermelon
    ///
    /// Radii increase by 10 pixels per stage, from 20px to 120px.
    pub const RADII: [f32; 11] = [
        20.0,  // Cherry
        30.0,  // Strawberry
        40.0,  // Grape
        50.0,  // Dekopon
        60.0,  // Persimmon
        70.0,  // Apple
        80.0,  // Pear
        90.0,  // Peach
        100.0, // Pineapple
        110.0, // Melon
        120.0, // Watermelon
    ];

    /// Points awarded for creating each fruit type through merging
    ///
    /// Index corresponds to FruitType enum order:
    /// 0: Cherry, 1: Strawberry, 2: Grape, 3: Dekopon, 4: Persimmon,
    /// 5: Apple, 6: Pear, 7: Peach, 8: Pineapple, 9: Melon, 10: Watermelon
    ///
    /// Points double with each stage, from 10 to 10240.
    pub const POINTS: [u32; 11] = [
        10,    // Cherry
        20,    // Strawberry
        40,    // Grape
        80,    // Dekopon
        160,   // Persimmon
        320,   // Apple
        640,   // Pear
        1280,  // Peach
        2560,  // Pineapple
        5120,  // Melon
        10240, // Watermelon
    ];

    /// Calculates fruit mass from radius
    ///
    /// Mass is proportional to the area (radius squared) for 2D physics.
    /// This creates realistic physics where larger fruits have more inertia.
    ///
    /// # Arguments
    ///
    /// * `radius` - The fruit's collision radius in pixels
    ///
    /// # Returns
    ///
    /// Mass value for physics simulation
    ///
    /// # Formula
    ///
    /// mass = radius² × 0.01
    ///
    /// # Examples
    ///
    /// ```
    /// # use suika_game_core::constants::fruit::calculate_mass;
    /// assert_eq!(calculate_mass(20.0), 4.0);    // Cherry
    /// assert_eq!(calculate_mass(120.0), 144.0); // Watermelon
    /// ```
    pub fn calculate_mass(radius: f32) -> f32 {
        radius * radius * 0.01
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fruit_radii_progression() {
        // Radii should increase by 10px per stage
        for i in 0..fruit::RADII.len() - 1 {
            assert_eq!(
                fruit::RADII[i + 1] - fruit::RADII[i],
                10.0,
                "Radius progression should be 10px per stage"
            );
        }

        // First and last values
        assert_eq!(fruit::RADII[0], 20.0);
        assert_eq!(fruit::RADII[10], 120.0);
    }

    #[test]
    fn test_fruit_points_progression() {
        // Points should double each stage
        for i in 0..fruit::POINTS.len() - 1 {
            assert_eq!(
                fruit::POINTS[i + 1],
                fruit::POINTS[i] * 2,
                "Points should double each stage"
            );
        }

        // First and last values
        assert_eq!(fruit::POINTS[0], 10);
        assert_eq!(fruit::POINTS[10], 10240);
    }

    #[test]
    fn test_combo_bonus_multiplier() {
        assert_eq!(combo::bonus_multiplier(0), 1.0);
        assert_eq!(combo::bonus_multiplier(1), 1.0);
        assert_eq!(combo::bonus_multiplier(2), 1.1);
        assert_eq!(combo::bonus_multiplier(3), 1.2);
        assert_eq!(combo::bonus_multiplier(4), 1.3);
        assert_eq!(combo::bonus_multiplier(5), 1.5);
        assert_eq!(combo::bonus_multiplier(10), 1.5); // Capped at 1.5x
        assert_eq!(combo::bonus_multiplier(100), 1.5); // Capped at 1.5x
    }

    #[test]
    fn test_fruit_mass_calculation() {
        // Test that mass scales with radius squared
        assert_eq!(fruit::calculate_mass(20.0), 4.0);
        assert_eq!(fruit::calculate_mass(30.0), 9.0);
        assert_eq!(fruit::calculate_mass(40.0), 16.0);
        assert_eq!(fruit::calculate_mass(120.0), 144.0);

        // Larger fruits should have more mass
        assert!(fruit::calculate_mass(120.0) > fruit::calculate_mass(20.0));
    }

    #[test]
    fn test_constant_values() {
        // Verify important constant values
        assert_eq!(physics::GRAVITY, -980.0);
        assert_eq!(physics::CONTAINER_WIDTH, 600.0);
        assert_eq!(physics::CONTAINER_HEIGHT, 800.0);
        assert_eq!(physics::WALL_THICKNESS, 20.0);
        assert_eq!(physics::BOUNDARY_LINE_Y, 300.0);
        assert_eq!(combo::COMBO_WINDOW, 2.0);
        assert_eq!(combo::COMBO_MAX, 10);
        assert_eq!(game_over::TIMER, 3.0);
        assert_eq!(storage::SAVE_DIR, "save");
    }

    #[test]
    fn test_array_lengths() {
        // Both arrays should have 11 elements (one for each fruit type)
        assert_eq!(fruit::RADII.len(), 11);
        assert_eq!(fruit::POINTS.len(), 11);
    }
}
