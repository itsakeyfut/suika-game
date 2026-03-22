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
    /// Visual scale multiplier (sprite diameter = radius × 2 × sprite_scale).
    /// Does not affect the physics collider.
    pub sprite_scale: f32,
    /// Horizontal anchor offset passed to `Sprite::anchor` (see `FruitConfigEntry::sprite_anchor_x`).
    pub sprite_anchor_x: f32,
    /// Vertical anchor offset passed to `Sprite::anchor` (see `FruitConfigEntry::sprite_anchor_y`).
    pub sprite_anchor_y: f32,
}
