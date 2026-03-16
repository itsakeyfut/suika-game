//! `FruitSprites` resource — maps [`FruitType`] to loaded [`Image`] handles.
//!
//! Sprites are loaded by the `suika-game-assets` crate at startup via
//! `load_fruit_sprites`.  Core systems (`spawn_fruit`, `spawn_held_fruit`)
//! consult this resource and fall back to the circular placeholder when no
//! sprite is registered for a given fruit type.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::fruit::FruitType;

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Loaded sprite handles for fruit types that have artwork.
///
/// Inserted as a resource at startup by [`crate::GameCorePlugin`].
/// Populated by the `load_fruit_sprites` system in the assets crate.
///
/// # Fallback behaviour
///
/// If a fruit type is absent from `handles`, the spawning systems fall back
/// to the procedurally generated circle texture tinted with
/// [`FruitType::placeholder_color`].
#[derive(Resource, Debug, Default)]
pub struct FruitSprites {
    handles: HashMap<FruitType, Handle<Image>>,
}

impl FruitSprites {
    /// Registers a sprite `handle` for the given `fruit_type`.
    pub fn insert(&mut self, fruit_type: FruitType, handle: Handle<Image>) {
        self.handles.insert(fruit_type, handle);
    }

    /// Returns the handle for `fruit_type`, or `None` if no sprite is loaded.
    pub fn get(&self, fruit_type: FruitType) -> Option<&Handle<Image>> {
        self.handles.get(&fruit_type)
    }

    /// Resolves `(image, color)` for spawning.
    ///
    /// Returns the real sprite with [`Color::WHITE`] when one is registered,
    /// otherwise returns `fallback` paired with [`FruitType::placeholder_color`].
    pub fn resolve(
        &self,
        fruit_type: FruitType,
        fallback: Handle<Image>,
    ) -> (Handle<Image>, Color) {
        match self.get(fruit_type) {
            Some(handle) => (handle.clone(), Color::WHITE),
            None => (fallback, fruit_type.placeholder_color()),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_handle() -> Handle<Image> {
        Handle::default()
    }

    #[test]
    fn test_get_returns_none_when_absent() {
        let sprites = FruitSprites::default();
        assert!(sprites.get(FruitType::Cherry).is_none());
    }

    #[test]
    fn test_insert_and_get() {
        let mut sprites = FruitSprites::default();
        sprites.insert(FruitType::Cherry, make_handle());
        assert!(sprites.get(FruitType::Cherry).is_some());
    }

    #[test]
    fn test_resolve_returns_fallback_when_absent() {
        let sprites = FruitSprites::default();
        let fallback = make_handle();
        let (img, color) = sprites.resolve(FruitType::Strawberry, fallback.clone());
        assert_eq!(img, fallback);
        assert_ne!(color, Color::WHITE, "Fallback should use placeholder color");
    }

    #[test]
    fn test_resolve_returns_white_when_sprite_present() {
        let mut sprites = FruitSprites::default();
        sprites.insert(FruitType::Cherry, make_handle());
        let fallback = make_handle();
        let (_, color) = sprites.resolve(FruitType::Cherry, fallback);
        assert_eq!(color, Color::WHITE, "Real sprite should use white tint");
    }
}
