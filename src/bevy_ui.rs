use bevy::{
    prelude::*,
    window::{MonitorSelection, PrimaryWindow, WindowMode},
    winit::{UpdateMode, WinitSettings},
};

use crate::app_state::AppState;
use crate::confirm_ui::ConfirmUiPlugin;
use crate::face::{FacePlugin, SharedAppState};
use crate::help_ui::HelpUiPlugin;
use crate::screen::{AppScreen, ScreenPlugin};

pub fn run(state: AppState) {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "pib face".into(),
                // Fill the whole screen without a title bar or border.
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
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
        .add_systems(Update, toggle_fullscreen)
        // Only on the face: on the keyboard screen Esc means Cancel.
        .add_systems(Update, exit_on_escape.run_if(in_state(AppScreen::Face)))
        .run();
}

/// F11 switches between fullscreen and a normal window (handy during development).
fn toggle_fullscreen(
    keys: Res<ButtonInput<KeyCode>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::F11) {
        window.mode = match window.mode {
            WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
            _ => WindowMode::Windowed,
        };
    }
}

/// Esc closes the app.
fn exit_on_escape(keys: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
