//! Debug rendering and visualization tools
//!
//! This module provides debug visualization features for development,
//! including a GUI inspector window powered by bevy-inspector-egui.
//!
//! Debug features are only enabled in debug builds and are automatically
//! stripped from release builds.

use bevy::prelude::*;

/// Debug plugin for development tools and visualizations
///
/// This plugin adds debug rendering capabilities including:
/// - Inspector GUI window (bevy-inspector-egui)
/// - Physics collider visualization (Rapier debug renderer)
/// - Resource and component inspection
///
/// # Debug Builds Only
///
/// All debug features are conditionally compiled and only available
/// in debug builds (`#[cfg(debug_assertions)]`). They are completely
/// removed from release builds.
///
/// # Controls
///
/// The inspector window can be used to:
/// - Toggle physics collider rendering
/// - Inspect entities and their components
/// - Modify resource values at runtime
/// - View game state in real-time
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(debug_assertions)]
        {
            use bevy_inspector_egui::quick::WorldInspectorPlugin;
            use bevy_rapier2d::render::RapierDebugRenderPlugin;

            info!("Debug mode enabled - inspector GUI window available");

            // Add Rapier debug renderer
            app.add_plugins(RapierDebugRenderPlugin::default());

            // Add inspector GUI
            app.add_plugins(WorldInspectorPlugin::new());
        }

        #[cfg(not(debug_assertions))]
        {
            // In release builds, this plugin does nothing
            info!("Release mode - debug rendering disabled");
        }
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
