//! Fruit spawning system
//!
//! This module handles spawning fruits into the game world with appropriate
//! physics bodies, colliders, and visual representation.

pub mod circle;
pub mod fruit;
pub mod plugin;

pub use circle::setup_circle_texture;
pub use fruit::spawn_fruit;
pub use plugin::SpawnPlugin;
