//! Next-fruit-type resource

use bevy::prelude::*;

use crate::fruit::FruitType;

/// Next fruit type resource
///
/// Stores the type of fruit that will be spawned next.
/// This allows the UI to display a preview of the upcoming fruit.
#[derive(Resource, Debug, Clone)]
pub struct NextFruitType(pub FruitType);

impl Default for NextFruitType {
    fn default() -> Self {
        Self(FruitType::Cherry)
    }
}

impl NextFruitType {
    /// Gets the current next fruit type
    pub fn get(&self) -> FruitType {
        self.0
    }

    /// Sets a new next fruit type
    pub fn set(&mut self, fruit_type: FruitType) {
        self.0 = fruit_type;
    }

    /// Generates a random spawnable fruit type from the full built-in list.
    ///
    /// Returns one of the 5 spawnable fruit types (Cherry through Persimmon)
    /// with equal probability.  Prefer [`randomize`] when the spawnable count
    /// comes from `GameRulesConfig`.
    pub fn random() -> FruitType {
        use rand::RngExt;
        let spawnable = FruitType::spawnable_fruits();
        let index = rand::rng().random_range(0..spawnable.len());
        spawnable[index]
    }

    /// Updates to a new random fruit type, respecting the configured count.
    ///
    /// `spawnable_count` is read from `GameRulesConfig::spawnable_fruit_count`
    /// and determines how many of the leading entries in
    /// `FruitType::spawnable_fruits()` are eligible.  Values outside the range
    /// `1..=5` are clamped silently.
    pub fn randomize(&mut self, spawnable_count: usize) {
        use rand::RngExt;
        let spawnable = FruitType::spawnable_fruits();
        let n = spawnable_count.clamp(1, spawnable.len());
        self.0 = spawnable[rand::rng().random_range(0..n)];
    }
}
