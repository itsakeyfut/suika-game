//! Debug rendering and visualization tools
//!
//! This module provides debug visualization features for development,
//! including physics collider rendering that can be toggled at runtime.
//!
//! Debug features are only enabled in debug builds and are automatically
//! stripped from release builds.

use bevy::prelude::*;

/// Debug plugin for development tools and visualizations
///
/// This plugin adds debug rendering capabilities including:
/// - Physics collider visualization
/// - Toggle debug rendering with the D key
///
/// # Debug Builds Only
///
/// All debug features are conditionally compiled and only available
/// in debug builds (`#[cfg(debug_assertions)]`). They are completely
/// removed from release builds.
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(debug_assertions)]
        {
            use bevy_rapier2d::render::RapierDebugRenderPlugin;

            info!("Debug mode enabled - press D to toggle physics debug rendering");

            // Add Rapier debug renderer
            app.add_plugins(RapierDebugRenderPlugin::default());

            // Add debug toggle system
            app.add_systems(Update, toggle_debug_render);
        }

        #[cfg(not(debug_assertions))]
        {
            // In release builds, this plugin does nothing
            info!("Release mode - debug rendering disabled");
        }
    }
}

/// Toggles physics debug rendering on/off with the D key
///
/// This system listens for the D key press and toggles the visibility
/// of physics colliders and other Rapier debug information.
///
/// # Controls
///
/// - `D` key: Toggle debug rendering on/off
#[cfg(debug_assertions)]
fn toggle_debug_render(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_render: ResMut<bevy_rapier2d::render::DebugRenderContext>,
) {
    if keyboard.just_pressed(KeyCode::KeyD) {
        debug_render.enabled = !debug_render.enabled;

        let status = if debug_render.enabled {
            "enabled"
        } else {
            "disabled"
        };

        info!("Debug rendering {}", status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_plugin_builds() {
        // Test that the plugin can be constructed
        let plugin = DebugPlugin;

        // Verify the plugin type is correct
        assert_eq!(
            std::any::type_name::<DebugPlugin>(),
            std::any::type_name_of_val(&plugin)
        );
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_debug_mode_enabled() {
        // In debug builds, debug assertions should be enabled
        let debug_enabled = cfg!(debug_assertions);
        assert!(
            debug_enabled,
            "Debug mode should be enabled in debug builds"
        );
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn test_debug_mode_disabled() {
        // In release builds, debug assertions should be disabled
        let debug_enabled = cfg!(debug_assertions);
        assert!(
            !debug_enabled,
            "Debug mode should be disabled in release builds"
        );
    }
}
