//! InputPlugin — registers all input systems

use bevy::prelude::*;

use crate::states::AppState;

use super::drop::{detect_fruit_landing, handle_fruit_drop_input};
use super::movement::update_spawn_position;
use super::resources::{InputMode, LastCursorPosition, SpawnPosition};
use super::spawn::spawn_held_fruit;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnPosition>()
            .init_resource::<InputMode>()
            .init_resource::<LastCursorPosition>();
        app.add_systems(
            Update,
            (
                update_spawn_position,
                handle_fruit_drop_input.after(update_spawn_position),
                detect_fruit_landing,
                spawn_held_fruit.after(detect_fruit_landing),
            )
                .run_if(in_state(AppState::Playing)),
        );
    }
}
