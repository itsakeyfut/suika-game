//! UI style constants: colors, font sizes, and layout values.
//!
//! All UI systems should source their visual properties from this module
//! to ensure a consistent look across every screen.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Color palette
// ---------------------------------------------------------------------------

/// Background color — light beige used as the base for all screens.
pub const BG_COLOR: Color = Color::srgb(0.95, 0.95, 0.90);

/// Primary color — green used for main actions and positive UI elements.
pub const PRIMARY_COLOR: Color = Color::srgb(0.3, 0.6, 0.3);

/// Secondary color — orange used for accents and secondary UI elements.
pub const SECONDARY_COLOR: Color = Color::srgb(0.9, 0.5, 0.2);

/// Text color — near-black used for all body text.
pub const TEXT_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);

/// Highlight color — bright yellow used for scores, combos, and emphasis.
pub const HIGHLIGHT_COLOR: Color = Color::srgb(1.0, 0.9, 0.0);

// ---------------------------------------------------------------------------
// Button colors
// ---------------------------------------------------------------------------

/// Button color in its default (resting) state.
pub const BUTTON_NORMAL: Color = Color::srgb(0.4, 0.7, 0.4);

/// Button color when the cursor hovers over it.
pub const BUTTON_HOVER: Color = Color::srgb(0.5, 0.8, 0.5);

/// Button color while the mouse button is held down.
pub const BUTTON_PRESSED: Color = Color::srgb(0.3, 0.5, 0.3);

// ---------------------------------------------------------------------------
// Font sizes
// ---------------------------------------------------------------------------

/// Huge font size (72 px) — used for screen titles such as the game title.
pub const FONT_SIZE_HUGE: f32 = 72.0;

/// Large font size (48 px) — used for the score display and headings.
pub const FONT_SIZE_LARGE: f32 = 48.0;

/// Medium font size (32 px) — used for buttons and UI labels.
pub const FONT_SIZE_MEDIUM: f32 = 32.0;

/// Small font size (24 px) — used for supplementary information such as the
/// highscore or control hints.
pub const FONT_SIZE_SMALL: f32 = 24.0;

// ---------------------------------------------------------------------------
// Button sizes
// ---------------------------------------------------------------------------

/// Width of a large button (px) — used for primary actions like Start/Retry.
pub const BUTTON_LARGE_WIDTH: f32 = 240.0;

/// Height of a large button (px) — used for primary actions like Start/Retry.
pub const BUTTON_LARGE_HEIGHT: f32 = 80.0;

/// Width of a medium button (px) — used for secondary actions.
pub const BUTTON_MEDIUM_WIDTH: f32 = 200.0;

/// Height of a medium button (px) — used for secondary actions.
pub const BUTTON_MEDIUM_HEIGHT: f32 = 60.0;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_sizes_are_ordered() {
        assert!(FONT_SIZE_HUGE > FONT_SIZE_LARGE);
        assert!(FONT_SIZE_LARGE > FONT_SIZE_MEDIUM);
        assert!(FONT_SIZE_MEDIUM > FONT_SIZE_SMALL);
        assert!(FONT_SIZE_SMALL > 0.0);
    }

    #[test]
    fn test_button_colors_are_distinct() {
        // Each button state should have a visually distinguishable color.
        let normal = BUTTON_NORMAL.to_srgba();
        let hover = BUTTON_HOVER.to_srgba();
        let pressed = BUTTON_PRESSED.to_srgba();

        assert_ne!(normal.green, hover.green, "NORMAL and HOVER should differ");
        assert_ne!(
            normal.green, pressed.green,
            "NORMAL and PRESSED should differ"
        );
        assert_ne!(
            hover.green, pressed.green,
            "HOVER and PRESSED should differ"
        );
    }

    #[test]
    fn test_colors_are_in_valid_range() {
        for color in [
            BG_COLOR,
            PRIMARY_COLOR,
            SECONDARY_COLOR,
            TEXT_COLOR,
            HIGHLIGHT_COLOR,
            BUTTON_NORMAL,
            BUTTON_HOVER,
            BUTTON_PRESSED,
        ] {
            let srgba = color.to_srgba();
            assert!(
                (0.0..=1.0).contains(&srgba.red),
                "red out of range: {}",
                srgba.red
            );
            assert!(
                (0.0..=1.0).contains(&srgba.green),
                "green out of range: {}",
                srgba.green
            );
            assert!(
                (0.0..=1.0).contains(&srgba.blue),
                "blue out of range: {}",
                srgba.blue
            );
        }
    }
}
