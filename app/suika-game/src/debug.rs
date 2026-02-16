//! Debug rendering and visualization tools
//!
//! This module provides debug visualization features for development,
//! including a GUI inspector window powered by bevy-inspector-egui.
//!
//! Debug features are only enabled when the `dev-tools` feature is active
//! (typically in debug builds) and are automatically stripped from release builds.

use bevy::prelude::*;

/// Debug plugin for development tools and visualizations
///
/// This plugin adds debug rendering capabilities including:
/// - Inspector GUI window (bevy-inspector-egui)
/// - Physics collider visualization (Rapier debug renderer)
/// - Resource and component inspection
///
/// # Feature Gating
///
/// All debug features are conditionally compiled and only available
/// when the `dev-tools` feature is enabled. They are completely
/// removed from release builds.
///
/// # Controls
///
/// The inspector window can be used to:
/// - Toggle physics collider rendering via DebugRenderContext
/// - Inspect entities and their components
/// - Modify resource values at runtime
/// - View game state in real-time
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(all(debug_assertions, feature = "dev-tools"))]
        {
            use bevy_inspector_egui::bevy_egui::EguiPlugin;
            use bevy_inspector_egui::quick::WorldInspectorPlugin;
            use bevy_rapier2d::render::RapierDebugRenderPlugin;

            info!("Debug mode enabled - inspector GUI window available");

            // Add Rapier debug renderer
            app.add_plugins(RapierDebugRenderPlugin::default());

            // Add egui plugin first (required by WorldInspectorPlugin)
            app.add_plugins(EguiPlugin::default());

            // Add inspector GUI
            app.add_plugins(WorldInspectorPlugin::new());
        }

        #[cfg(not(all(debug_assertions, feature = "dev-tools")))]
        {
            // In release builds or without dev-tools feature, this plugin does nothing
            #[cfg(not(debug_assertions))]
            info!("Release mode - debug rendering disabled");

            #[cfg(all(debug_assertions, not(feature = "dev-tools")))]
            info!("Debug mode without dev-tools feature - debug rendering disabled");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_plugin_can_be_constructed() {
        // Test that the plugin can be constructed
        let _plugin = DebugPlugin;
        // If we get here without panicking, the test passes
    }

    #[cfg(not(feature = "dev-tools"))]
    #[test]
    fn test_debug_plugin_integrates_with_minimal_app() {
        // When dev-tools is disabled, DebugPlugin should work with MinimalPlugins
        // (it does nothing in this case, but shouldn't panic)
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(DebugPlugin);

        // The plugin should build without errors
        app.update();
    }

    #[cfg(all(debug_assertions, feature = "dev-tools"))]
    #[test]
    fn test_debug_plugin_requires_full_bevy_setup() {
        // When dev-tools is enabled, DebugPlugin adds EguiPlugin which requires
        // full Bevy graphics setup (not available in test environment with MinimalPlugins).
        // This test just documents that limitation - we can't test the full integration
        // without DefaultPlugins and a rendering context.

        // We can at least verify the plugin type exists
        let _plugin = DebugPlugin;

        // Integration testing with dev-tools enabled would require:
        // - DefaultPlugins
        // - Proper graphics context
        // - Window system
        // These are not available in unit test environment
    }

    #[cfg(all(debug_assertions, feature = "dev-tools"))]
    #[test]
    fn test_dev_tools_feature_enabled() {
        // When dev-tools feature is enabled in debug mode,
        // verify that we can import the inspector components
        use bevy_inspector_egui::quick::WorldInspectorPlugin;

        let _plugin = WorldInspectorPlugin::new();
        // If we get here, the feature is properly enabled
    }

    #[cfg(not(feature = "dev-tools"))]
    #[test]
    fn test_dev_tools_feature_disabled() {
        // When dev-tools feature is disabled, the inspector should not be compiled
        // This test just verifies the feature flag works
        assert!(
            !cfg!(feature = "dev-tools"),
            "dev-tools feature should be disabled in this test configuration"
        );
    }
}
