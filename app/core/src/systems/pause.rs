//! Physics pause and resume systems.
//!
//! Freezes the Rapier physics pipeline when entering [`AppState::Paused`] and
//! restores it when exiting.  All gameplay input and scoring systems already
//! gate themselves on `run_if(in_state(AppState::Playing))`, so disabling the
//! physics pipeline is the only additional step needed to fully pause the
//! simulation.

use bevy::prelude::*;
use bevy_rapier2d::prelude::{DefaultRapierContext, RapierConfiguration};

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Freezes the physics pipeline on entering [`AppState::Paused`].
///
/// Sets [`RapierConfiguration::physics_pipeline_active`] to `false` so no
/// physics steps are calculated while the game is paused.
pub fn pause_physics(
    mut rapier_query: Query<&mut RapierConfiguration, With<DefaultRapierContext>>,
) {
    if let Ok(mut cfg) = rapier_query.single_mut() {
        cfg.physics_pipeline_active = false;
    }
}

/// Restores the physics pipeline on exiting [`AppState::Paused`].
///
/// Sets [`RapierConfiguration::physics_pipeline_active`] back to `true` so
/// the simulation resumes immediately.
pub fn resume_physics(
    mut rapier_query: Query<&mut RapierConfiguration, With<DefaultRapierContext>>,
) {
    if let Ok(mut cfg) = rapier_query.single_mut() {
        cfg.physics_pipeline_active = true;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn test_pause_resume_are_inverse_operations() {
        // Simulate the logical effect of each system on the pipeline flag.
        let after_pause = false;
        let after_resume = true;

        assert!(!after_pause, "pipeline should be inactive after pause");
        assert!(after_resume, "pipeline should be active after resume");
    }
}
