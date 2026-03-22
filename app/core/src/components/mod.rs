//! Game entity components
//!
//! This module defines all the components used in the game's ECS architecture.
//! Components represent data attached to entities and define their behavior.

pub mod container;
pub mod fruit;
pub mod ui;

pub use container::*;
pub use fruit::*;
pub use ui::*;
