
use std::collections::VecDeque;
use bevy::{
    prelude::*,
    winit::{UpdateMode, WinitSettings}
};
use crate::app_state::AppState;
use std::time::Duration;

const MAX_DISPLAYED_MESSAGES: usize = 20;

#[derive(Resource, Clone)]
struct SharedAppState(AppState);

#[derive(Component)]
struct OutputText;

pub fn run(state: AppState) {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(SharedAppState(state))
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::reactive(Duration::from_secs_f64(1.0 / 30.0)),
            unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs(5)),
        })
        .insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.12)))
        .add_systems(Startup, setup)
        .add_systems(Update, update_output)
        .run();
}

fn setup(mut commands: Commands) {
    // Required for Bevy UI rendering.
    commands.spawn(Camera2d);

    // Main text display.
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.09, 0.12)),
        children![
            (
                Text::new("ROS 2 Subscriber Output\n\nWaiting for messages..."),
                TextFont::from_font_size(36.),
                TextColor(Color::srgb(0.85, 0.9, 0.95)),
                OutputText,
            )
        ],
    ));
}

fn update_output(
    state: Res<SharedAppState>,
    mut last_seen: Local<u64>,
    mut history: Local<VecDeque<String>>,
    mut text_query: Query<&mut Text, With<OutputText>>,
) {
    // Collect new messages from the shared application state.
    for (id, message) in state.0.messages_after(*last_seen) {
        history.push_back(message);
        *last_seen = (*last_seen).max(id);

        while history.len() > MAX_DISPLAYED_MESSAGES {
            history.pop_front();
        }
    }

    // Avoid updating the UI when there is nothing new to display.
    if history.is_empty() {
        return;
    }

    let mut output = String::from("ROS 2 Subscriber Output\n\n");

    for message in history.iter() {
        output.push_str(message);
        output.push('\n');
    }

    if let Ok(mut text) = text_query.single_mut() {
        text.0 = output;
    }
}
