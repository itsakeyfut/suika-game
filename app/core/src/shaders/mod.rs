//! WGSL shader materials for 2D visual effects.
//!
//! Defines [`Material2d`] types used by particle and overlay systems, plus
//! shared mesh-handle resources.  WGSL source files live in the game's
//! `assets/shaders/` directory and are loaded by Bevy's `AssetServer` at
//! runtime (the standard Bevy 0.17 approach for application shaders).
//!
//! # Usage
//!
//! Add [`ShadersPlugin`] to the application **after** `DefaultPlugins`
//! (which includes the rendering and asset plugins):
//!
//! ```no_run
//! use bevy::prelude::*;
//! use suika_game_core::shaders::ShadersPlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(ShadersPlugin)
//!         .run();
//! }
//! ```
//!
//! When `ShadersPlugin` is **not** present (e.g. in headless tests), systems
//! that accept `Option<Res<UnitQuadMesh>>` / `Option<ResMut<Assets<…>>>` will
//! receive `None` and fall back to plain [`bevy::sprite::Sprite`] rendering.

use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};

// ---------------------------------------------------------------------------
// Shader asset paths (relative to the game's `assets/` directory)
// ---------------------------------------------------------------------------

const SOFT_CIRCLE_SHADER_PATH: &str = "shaders/soft_circle.wgsl";
const RADIAL_GRADIENT_SHADER_PATH: &str = "shaders/radial_gradient.wgsl";
const SCREEN_DROPLET_SHADER_PATH: &str = "shaders/screen_droplet.wgsl";
const WEATHER_POSTPROCESS_SHADER_PATH: &str = "shaders/weather_postprocess.wgsl";
const RAIN_DROP_SHADER_PATH: &str = "shaders/rain_drop.wgsl";
const SUN_SHADER_PATH: &str = "shaders/sun.wgsl";
const CLOUD_PUFF_SHADER_PATH: &str = "shaders/cloud_puff.wgsl";

// ---------------------------------------------------------------------------
// Material definitions
// ---------------------------------------------------------------------------

/// Soft-circle material — renders an SDF circle with smooth anti-aliased edges.
///
/// Used for [`crate::systems::effects::droplet::WaterDroplet`] particles.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SoftCircleMaterial {
    /// RGBA colour in linear space (alpha is faded per-frame by the update system).
    #[uniform(0)]
    pub color: LinearRgba,
}

impl Material2d for SoftCircleMaterial {
    fn fragment_shader() -> ShaderRef {
        SOFT_CIRCLE_SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Radial-gradient material — bright centre, transparent edge.
///
/// Used for [`crate::systems::effects::flash::LocalFlashAnimation`].
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct RadialGradientMaterial {
    /// RGBA colour in linear space.
    #[uniform(0)]
    pub color: LinearRgba,
}

impl Material2d for RadialGradientMaterial {
    fn fragment_shader() -> ShaderRef {
        RADIAL_GRADIENT_SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Screen-droplet material — hollow ring with refraction highlight.
///
/// Used for [`crate::systems::effects::screen_droplet::ScreenDroplet`].
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ScreenDropletMaterial {
    /// RGBA colour in linear space.
    #[uniform(0)]
    pub color: LinearRgba,
}

impl Material2d for ScreenDropletMaterial {
    fn fragment_shader() -> ShaderRef {
        SCREEN_DROPLET_SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Weather post-process overlay material — full-screen vignette tint.
///
/// Used by [`crate::systems::effects::postprocess`].
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WeatherPostprocessMaterial {
    /// RGBA tint colour in linear space.
    #[uniform(0)]
    pub color: LinearRgba,
}

impl Material2d for WeatherPostprocessMaterial {
    fn fragment_shader() -> ShaderRef {
        WEATHER_POSTPROCESS_SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Rain-drop streak material — soft elongated streak with tapered tips and
/// a subtle centre glow simulating light refraction.
///
/// Used for [`crate::systems::effects::rain::RainDrop`] particles.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct RainDropMaterial {
    /// RGBA colour in linear space.
    #[uniform(0)]
    pub color: LinearRgba,
}

impl Material2d for RainDropMaterial {
    fn fragment_shader() -> ShaderRef {
        RAIN_DROP_SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Sun disc material — warm glowing disc with animated rotating light rays.
///
/// Both fields share binding 0 and are packed into a single WGSL uniform struct:
/// - `color`: RGBA tint applied to the sun
/// - `params`: `x` = elapsed time in seconds (drives ray rotation)
///
/// Used for [`crate::systems::effects::sun::SunEffect`].
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SunMaterial {
    /// RGBA tint colour (rgb multiplied into the warm palette; a = overall opacity).
    #[uniform(0)]
    pub color: LinearRgba,
    /// Shader parameters packed as Vec4.  `x` = elapsed time in seconds.
    #[uniform(0)]
    pub params: Vec4,
}

impl Material2d for SunMaterial {
    fn fragment_shader() -> ShaderRef {
        SUN_SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Cloud puff material — fluffy SDF cloud shape rendered via smooth-union circles.
///
/// Used for [`crate::systems::effects::cloud_effect::CloudPuff`] entities.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CloudMaterial {
    /// RGBA colour in linear space (alpha updated per-frame for edge fading).
    #[uniform(0)]
    pub color: LinearRgba,
}

impl Material2d for CloudMaterial {
    fn fragment_shader() -> ShaderRef {
        CLOUD_PUFF_SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// ---------------------------------------------------------------------------
// Shared mesh-handle resources
// ---------------------------------------------------------------------------

/// A 1×1 unit quad mesh shared among all shader-based particles.
///
/// Inserted by [`ShadersPlugin`] at [`Startup`]. Systems that spawn
/// shader particles accept `Option<Res<UnitQuadMesh>>` and fall back to
/// plain [`bevy::sprite::Sprite`] when this resource is absent.
#[derive(Resource)]
pub struct UnitQuadMesh(pub Handle<Mesh>);

/// A very large quad used for full-screen overlay effects.
///
/// Large enough (20 000 × 20 000 world units) to cover the screen at any
/// camera zoom used in this game.
#[derive(Resource)]
pub struct FullScreenQuadMesh(pub Handle<Mesh>);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers all custom [`Material2d`] types and creates shared mesh resources.
///
/// **Must be added after `DefaultPlugins`** because it needs the asset and
/// render plugins that `DefaultPlugins` provides.
pub struct ShadersPlugin;

impl Plugin for ShadersPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<SoftCircleMaterial>::default())
            .add_plugins(Material2dPlugin::<RadialGradientMaterial>::default())
            .add_plugins(Material2dPlugin::<ScreenDropletMaterial>::default())
            .add_plugins(Material2dPlugin::<WeatherPostprocessMaterial>::default())
            .add_plugins(Material2dPlugin::<RainDropMaterial>::default())
            .add_plugins(Material2dPlugin::<SunMaterial>::default())
            .add_plugins(Material2dPlugin::<CloudMaterial>::default())
            .add_systems(Startup, setup_shared_meshes);
    }
}

fn setup_shared_meshes(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    // Unit quad: actual size is controlled via Transform::scale at spawn time.
    let unit = meshes.add(Rectangle::new(1.0, 1.0));
    commands.insert_resource(UnitQuadMesh(unit));

    // Full-screen quad: large enough for any camera configuration.
    let full = meshes.add(Rectangle::new(20_000.0, 20_000.0));
    commands.insert_resource(FullScreenQuadMesh(full));
}
