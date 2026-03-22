//! Circle texture generation system

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::resources::CircleTexture;

/// Generates a white circle image and stores it as [`CircleTexture`].
///
/// Creates a 128×128 RGBA image where every pixel inside the disc is opaque
/// white and every pixel outside is fully transparent.  Fruit sprites tint
/// this texture with `Sprite::color` to achieve their individual colours.
///
/// Run this system once at `Startup` (before any fruit is spawned).
pub fn setup_circle_texture(
    mut circle_texture: ResMut<CircleTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    const DIAMETER: u32 = 128;
    const RADIUS: f32 = 64.0;

    let mut data = vec![0u8; (DIAMETER * DIAMETER * 4) as usize];
    for y in 0..DIAMETER {
        for x in 0..DIAMETER {
            let dx = x as f32 + 0.5 - RADIUS;
            let dy = y as f32 + 0.5 - RADIUS;
            let alpha = if dx * dx + dy * dy <= RADIUS * RADIUS {
                255u8
            } else {
                0u8
            };
            let idx = ((y * DIAMETER + x) * 4) as usize;
            data[idx] = 255; // R — white
            data[idx + 1] = 255; // G — white
            data[idx + 2] = 255; // B — white
            data[idx + 3] = alpha; // A — disc shape
        }
    }

    let image = Image::new(
        Extent3d {
            width: DIAMETER,
            height: DIAMETER,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    circle_texture.0 = images.add(image);
}
