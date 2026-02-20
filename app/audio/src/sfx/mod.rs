//! Sound-effect (SFX) playback systems.
//!
//! This module contains Bevy systems that play one-shot sound effects in
//! response to game events.
//!
//! # Sub-modules
//!
//! | Module | Systems |
//! |--------|---------|
//! | [`game`] | [`play_merge_sfx`], [`play_combo_sfx`], [`play_gameover_sfx`] |
//! | [`ui`]   | [`play_ui_sfx`], [`play_keyboard_ui_sfx`] |

pub mod game;
pub mod ui;

pub use game::*;
pub use ui::*;

use suika_game_core::fruit::FruitType;

// ---------------------------------------------------------------------------
// Merge SFX category
// ---------------------------------------------------------------------------

/// Internal category used to select the right handle and pitch for a merge.
pub(super) enum MergeSfxCategory {
    /// Cherry, Strawberry, Grape — high-pitched pop.
    Small,
    /// Dekopon, Persimmon, Apple, Pear — mid-pitched pop.
    Medium,
    /// Peach, Pineapple — low-pitched thud.
    Large,
    /// Melon → Watermelon — special fanfare, no pitch shift.
    Watermelon,
}

impl MergeSfxCategory {
    /// Classifies a [`FruitType`] into the appropriate SFX category.
    pub(super) fn from_fruit(fruit: FruitType) -> Self {
        match fruit {
            FruitType::Cherry | FruitType::Strawberry | FruitType::Grape => Self::Small,
            FruitType::Dekopon | FruitType::Persimmon | FruitType::Apple | FruitType::Pear => {
                Self::Medium
            }
            FruitType::Peach | FruitType::Pineapple => Self::Large,
            // Two Melons merging → Watermelon fanfare.
            FruitType::Melon | FruitType::Watermelon => Self::Watermelon,
        }
    }
}
