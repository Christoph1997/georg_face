use bevy::{
    input::{
        keyboard::{Key, KeyboardInput},
        touch::Touches,
    },
    prelude::*,
    window::PrimaryWindow,
};

use crate::face::{EYE_COLOR, SharedAppState};
use crate::pointer::Pointers;
use crate::screen::{AppScreen, ConfirmResult, despawn_all};

/// Question shown when the task manager sends no text of its own.
pub const DEFAULT_QUESTION: &str = "Do you want to grab that bottle?";
/// Seconds without an answer before the screen gives up.
const TIMEOUT_SECS: f32 = 30.0;

const YES_BG: Color = Color::srgb(0.08, 0.38, 0.22);
const YES_HOT: Color = Color::srgb(0.14, 0.65, 0.38);
const NO_BG: Color = Color::srgb(0.42, 0.13, 0.13);
const NO_HOT: Color = Color::srgb(0.75, 0.22, 0.22);

#[derive(Component)]
struct ConfirmUi;

#[derive(Component)]
struct AnswerButton(bool); // true = Yes

/// Timer for the current question.
#[derive(Resource, Default)]
struct ConfirmState {
    idle: f32,
    /// Answer already reported; ignore further input and timers.
    done: bool,
}

pub struct ConfirmUiPlugin;

impl Plugin for ConfirmUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConfirmState>()
            .add_systems(Update, trigger_key.run_if(in_state(AppScreen::Face)))
            .add_systems(OnEnter(AppScreen::Confirm), spawn_confirm_ui)
            .add_systems(OnExit(AppScreen::Confirm), despawn_all::<ConfirmUi>)
            .add_systems(
                Update,
                (press_buttons, physical_keyboard, timeout)
                    .chain()
                    .run_if(in_state(AppScreen::Confirm)),
            );
    }
}

/// Dev trigger without ROS 2: press C on the face screen.
fn trigger_key(keys: Res<ButtonInput<KeyCode>>, state: Res<SharedAppState>) {
    if keys.just_pressed(KeyCode::KeyC) {
        state.0.request_confirm(DEFAULT_QUESTION);
    }
}

fn spawn_confirm_ui(mut commands: Commands, state: Res<SharedAppState>, mut cs: ResMut<ConfirmState>) {
    *cs = ConfirmState::default();
    let question = state.0.confirm_question();

    commands
        .spawn((
            ConfirmUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Vh(10.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Percent(80.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(
                    Text::new(question),
                    TextFont::from_font_size(60.0),
                    TextColor(EYE_COLOR),
                    TextLayout::justify(Justify::Center),
                )],
            ));

            root.spawn(Node {
                column_gap: Val::Vw(6.0),
                ..default()
            })
            .with_children(|row| {
                spawn_button(row, true, "Yes", YES_BG);
                spawn_button(row, false, "No", NO_BG);
            });
        });
}

fn spawn_button(row: &mut ChildSpawnerCommands, yes: bool, label: &str, bg: Color) {
    row.spawn((
        AnswerButton(yes),
        Node {
            width: Val::Vw(26.0),
            height: Val::Vh(24.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(bg),
        children![(
            Text::new(label),
            TextFont::from_font_size(72.0),
            TextColor(Color::WHITE),
        )],
    ));
}

fn answer(yes: bool, cs: &mut ConfirmState, state: &SharedAppState) {
    if cs.done {
        return;
    }
    cs.done = true;
    state
        .0
        .report_confirm(if yes { ConfirmResult::Yes } else { ConfirmResult::No });
    state.0.set_screen(AppScreen::Face);
}

/// Touch and mouse: each pointer is hit-tested with its own position.
fn press_buttons(
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut buttons: Query<(&ComputedNode, &UiGlobalTransform, &AnswerButton, &mut BackgroundColor)>,
    mut cs: ResMut<ConfirmState>,
    state: Res<SharedAppState>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let pointers = Pointers::gather(window, &mouse, &touches);

    for (node, transform, button, mut bg) in &mut buttons {
        if pointers.pressed_on(node, transform) {
            cs.idle = 0.0;
            answer(button.0, &mut cs, &state);
        }

        let (normal, hot) = if button.0 { (YES_BG, YES_HOT) } else { (NO_BG, NO_HOT) };
        let active = pointers.held_on(node, transform) || pointers.hovering(node, transform);
        let color = if active { hot } else { normal };
        if bg.0 != color {
            bg.0 = color;
        }
    }
}

/// Development: Y = yes, N = no.
fn physical_keyboard(
    mut events: MessageReader<KeyboardInput>,
    mut cs: ResMut<ConfirmState>,
    state: Res<SharedAppState>,
) {
    for ev in events.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        if let Key::Character(s) = &ev.logical_key {
            match s.to_ascii_lowercase().as_str() {
                "y" => answer(true, &mut cs, &state),
                "n" => answer(false, &mut cs, &state),
                _ => {}
            }
        }
    }
}

fn timeout(time: Res<Time>, mut cs: ResMut<ConfirmState>, state: Res<SharedAppState>) {
    if cs.done {
        return;
    }
    cs.idle += time.delta_secs();
    if cs.idle > TIMEOUT_SECS {
        cs.done = true;
        state.0.report_confirm(ConfirmResult::Timeout);
        state.0.set_screen(AppScreen::Face);
    }
}
