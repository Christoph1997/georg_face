use bevy::{
    prelude::*,
    winit::{UpdateMode, WinitSettings},
};

use crate::app_state::AppState;
use crate::confirm_ui::ConfirmUiPlugin;
use crate::face::{FacePlugin, SharedAppState};
use crate::help_ui::HelpUiPlugin;
use crate::screen::ScreenPlugin;

pub fn run(state: AppState) {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "pib face".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(SharedAppState(state))
        // Continuous updates keep the animation smooth.
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        })
        .add_plugins((ScreenPlugin, FacePlugin, HelpUiPlugin, ConfirmUiPlugin))
        .run();
}
