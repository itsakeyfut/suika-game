//! Weather system — current weather state and visual parameters.
//!
//! The weather for each game session is determined randomly when the player
//! enters [`crate::states::AppState::Playing`].  It remains constant for the
//! duration of that session and drives rain particles, screen-droplet spawning,
//! the full-screen colour overlay, and the camera bloom setting.

use bevy::prelude::*;
use rand::RngExt;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// One of three possible weather conditions for a play session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeatherState {
    /// Clear sunny conditions — camera bloom, warm sky.
    #[default]
    Sunny,
    /// Rain — falling rain particles, screen droplets, cool blue colour cast.
    Rainy,
    /// Overcast — grey desaturated overlay, no rain.
    Cloudy,
}

/// Visual and gameplay parameters derived from a [`WeatherState`].
#[derive(Debug, Clone, Copy)]
pub struct WeatherParams {
    /// Maximum simultaneous rain-drop particles (0 = no rain).
    pub max_rain_particles: u32,
    /// Random screen-droplet spawn events per second during rain (0 = none).
    pub screen_droplet_rate: f32,
    /// RGBA colour for the full-screen weather overlay (alpha=0 → no overlay).
    pub overlay_color: [f32; 4],
    /// Whether to enable camera bloom.
    pub bloom: bool,
    /// Bloom intensity (ignored when `bloom` is false).
    pub bloom_intensity: f32,
}

impl WeatherState {
    /// Returns the visual/gameplay parameters for this weather state.
    pub fn params(self) -> WeatherParams {
        match self {
            WeatherState::Sunny => WeatherParams {
                max_rain_particles: 0,
                screen_droplet_rate: 0.0,
                overlay_color: [0.0, 0.0, 0.0, 0.0], // transparent — no overlay
                bloom: true,
                bloom_intensity: 0.25,
            },
            WeatherState::Rainy => WeatherParams {
                max_rain_particles: 200,
                screen_droplet_rate: 0.4,
                overlay_color: [0.04, 0.04, 0.12, 0.10],
                bloom: false,
                bloom_intensity: 0.0,
            },
            WeatherState::Cloudy => WeatherParams {
                max_rain_particles: 0,
                screen_droplet_rate: 0.0,
                overlay_color: [0.08, 0.08, 0.08, 0.07],
                bloom: false,
                bloom_intensity: 0.0,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// The current weather for the active game session.
///
/// Initialised to [`WeatherState::Sunny`] (default) and randomised by
/// [`randomize_weather`] on every `OnEnter(AppState::Playing)`.
#[derive(Resource, Debug, Default)]
pub struct CurrentWeather {
    /// The active weather condition.
    pub state: WeatherState,
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Selects a random [`WeatherState`] at the start of each play session.
///
/// Registered by [`crate::GameCorePlugin`] on `OnEnter(AppState::Playing)`.
/// The three weather conditions are equally likely (~33 % each).
pub fn randomize_weather(mut weather: ResMut<CurrentWeather>) {
    let mut rng = rand::rng();
    let roll: f32 = rng.random();
    weather.state = if roll < 0.333 {
        WeatherState::Sunny
    } else if roll < 0.667 {
        WeatherState::Rainy
    } else {
        WeatherState::Cloudy
    };
    info!("Weather randomised → {:?}", weather.state);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_state_default_is_sunny() {
        assert_eq!(WeatherState::default(), WeatherState::Sunny);
    }

    #[test]
    fn test_current_weather_default() {
        let w = CurrentWeather::default();
        assert_eq!(w.state, WeatherState::Sunny);
    }

    #[test]
    fn test_sunny_params() {
        let p = WeatherState::Sunny.params();
        assert_eq!(p.max_rain_particles, 0);
        assert!(p.bloom);
        assert!(p.bloom_intensity > 0.0);
        // No overlay colour
        assert_eq!(p.overlay_color[3], 0.0);
    }

    #[test]
    fn test_rainy_params() {
        let p = WeatherState::Rainy.params();
        assert!(p.max_rain_particles > 0, "Rainy should have rain particles");
        assert!(
            p.screen_droplet_rate > 0.0,
            "Rainy should spawn screen droplets"
        );
        assert!(
            p.overlay_color[3] > 0.0,
            "Rainy should have a visible overlay"
        );
        assert!(!p.bloom);
    }

    #[test]
    fn test_cloudy_params() {
        let p = WeatherState::Cloudy.params();
        assert_eq!(p.max_rain_particles, 0);
        assert_eq!(p.screen_droplet_rate, 0.0);
        assert!(
            p.overlay_color[3] > 0.0,
            "Cloudy should have a visible overlay"
        );
        assert!(!p.bloom);
    }

    #[test]
    fn test_randomize_weather_produces_valid_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CurrentWeather>();
        app.add_systems(bevy::app::Startup, randomize_weather);
        app.update();

        let weather = app.world().resource::<CurrentWeather>();
        let valid = matches!(
            weather.state,
            WeatherState::Sunny | WeatherState::Rainy | WeatherState::Cloudy
        );
        assert!(valid, "randomize_weather must produce a valid WeatherState");
    }
}
