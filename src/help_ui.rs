use bevy::{
    input::{
        keyboard::{Key, KeyboardInput},
        touch::Touches,
    },
    prelude::*,
    window::PrimaryWindow,
};

use crate::emotion::Emotion;
use crate::face::{EYE_COLOR, SharedAppState};
use crate::pointer::Pointers;
use crate::screen::{AppScreen, HelpResult, despawn_all};

/// Text shown when the task manager sends no text of its own.
pub const DEFAULT_TITLE: &str = "Please enter your favorite drink";
const THANKS: &str = "Thank you!";
const MAX_LEN: usize = 40;
/// Seconds without input before the screen gives up.
const TIMEOUT_SECS: f32 = 60.0;
const THANKS_SECS: f32 = 2.0;

const KEY_BG: Color = Color::srgb(0.10, 0.12, 0.16);
const KEY_HOVER: Color = Color::srgb(0.16, 0.22, 0.30);
const FIELD_BG: Color = Color::srgb(0.07, 0.08, 0.11);

const ROWS: [&str; 3] = ["qwertyuiop", "asdfghjkl", "zxcvbnm"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyAction {
    Char(char),
    Space,
    Backspace,
    Shift,
    Enter,
    Cancel,
}

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    None,
    Submit,
    Cancel,
}

/// Text entered so far plus screen timers.
#[derive(Resource)]
struct InputBuffer {
    /// Text shown above the keyboard (from the request).
    title: String,
    text: String,
    shift: bool,
    idle: f32,
    /// Remaining seconds of the "Thank you" message.
    thanks: Option<f32>,
    /// Result already reported; ignore further input and timers.
    done: bool,
}

impl InputBuffer {
    fn new() -> Self {
        Self {
            title: DEFAULT_TITLE.to_string(),
            text: String::new(),
            shift: true, // first letter capitalised
            idle: 0.0,
            thanks: None,
            done: false,
        }
    }

    fn apply(&mut self, action: KeyAction) -> Outcome {
        if self.done {
            return Outcome::None;
        }
        self.idle = 0.0;
        match action {
            KeyAction::Char(c) => {
                if self.text.chars().count() < MAX_LEN {
                    if self.shift {
                        self.text.extend(c.to_uppercase());
                    } else {
                        self.text.push(c);
                    }
                }
                self.shift = false;
            }
            KeyAction::Space => {
                if !self.text.is_empty()
                    && !self.text.ends_with(' ')
                    && self.text.chars().count() < MAX_LEN
                {
                    self.text.push(' ');
                }
            }
            KeyAction::Backspace => {
                self.text.pop();
            }
            KeyAction::Shift => self.shift = !self.shift,
            KeyAction::Enter => {
                if !self.text.trim().is_empty() {
                    return Outcome::Submit;
                }
            }
            KeyAction::Cancel => return Outcome::Cancel,
        }
        Outcome::None
    }
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component)]
struct HelpUi;
#[derive(Component)]
struct TitleText;
#[derive(Component)]
struct InputText;
#[derive(Component)]
struct KeyLabel(char);
#[derive(Component)]
struct KeyButton(KeyAction);

pub struct HelpUiPlugin;

impl Plugin for HelpUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputBuffer>()
            .add_systems(Update, trigger_key.run_if(in_state(AppScreen::Face)))
            .add_systems(OnEnter(AppScreen::DrinkRequest), spawn_help_ui)
            .add_systems(OnExit(AppScreen::DrinkRequest), despawn_all::<HelpUi>)
            .add_systems(
                Update,
                (press_buttons, physical_keyboard, help_timers, update_display)
                    .chain()
                    .run_if(in_state(AppScreen::DrinkRequest)),
            );
    }
}

/// Dev trigger without ROS 2: press H on the face screen.
fn trigger_key(keys: Res<ButtonInput<KeyCode>>, state: Res<SharedAppState>) {
    if keys.just_pressed(KeyCode::KeyH) {
        state.0.request_help(DEFAULT_TITLE);
    }
}

fn spawn_help_ui(
    mut commands: Commands,
    state: Res<SharedAppState>,
    mut buf: ResMut<InputBuffer>,
) {
    *buf = InputBuffer::new();
    buf.title = state.0.help_title();
    let title = buf.title.clone();

    commands
        .spawn((
            HelpUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|root| {
            root.spawn((
                TitleText,
                Text::new(title),
                TextFont::from_font_size(46.0),
                TextColor(EYE_COLOR),
            ));

            root.spawn((
                Node {
                    width: Val::Percent(72.0),
                    height: Val::Vh(11.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(FIELD_BG),
                children![(
                    InputText,
                    Text::new(""),
                    TextFont::from_font_size(44.0),
                    TextColor(Color::WHITE),
                )],
            ));

            for letters in ROWS {
                root.spawn(Node {
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    if letters == "zxcvbnm" {
                        spawn_key(row, KeyAction::Shift, "Shift", 11.0, None);
                    }
                    for c in letters.chars() {
                        spawn_key(row, KeyAction::Char(c), &c.to_string(), 7.0, Some(c));
                    }
                    if letters == "zxcvbnm" {
                        spawn_key(row, KeyAction::Backspace, "<-", 11.0, None);
                    }
                });
            }

            root.spawn(Node {
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row| {
                spawn_key(row, KeyAction::Cancel, "Cancel", 12.0, None);
                spawn_key(row, KeyAction::Space, "Space", 32.0, None);
                spawn_key(row, KeyAction::Enter, "OK", 12.0, None);
            });
        });
}

fn spawn_key(
    row: &mut ChildSpawnerCommands,
    action: KeyAction,
    label: &str,
    width_vw: f32,
    letter: Option<char>,
) {
    let mut key = row.spawn((
        KeyButton(action),
        Node {
            width: Val::Vw(width_vw),
            height: Val::Vh(10.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(KEY_BG),
    ));
    key.with_children(|k| {
        let mut text = k.spawn((
            Text::new(label),
            TextFont::from_font_size(30.0),
            TextColor(Color::WHITE),
        ));
        if let Some(c) = letter {
            text.insert(KeyLabel(c));
        }
    });
}

fn handle_outcome(outcome: Outcome, buf: &mut InputBuffer, state: &SharedAppState) {
    match outcome {
        Outcome::Submit => {
            state
                .0
                .report_help(HelpResult::Answered(buf.text.trim().to_string()));
            buf.thanks = Some(THANKS_SECS);
            buf.done = true;
        }
        Outcome::Cancel => {
            state.0.report_help(HelpResult::Cancelled);
            state.0.set_screen(AppScreen::Face);
            buf.done = true;
        }
        Outcome::None => {}
    }
}

/// Handles mouse clicks and touchscreen taps by hit-testing the keys itself.
///
/// Bevy's `Interaction` prefers the OS mouse cursor position over touch
/// positions, so on a touchscreen (where the cursor sits elsewhere or is
/// stale) taps hit the wrong spot. Here each pointer uses its own position.
fn press_buttons(
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut keys: Query<(&ComputedNode, &UiGlobalTransform, &KeyButton, &mut BackgroundColor)>,
    mut buf: ResMut<InputBuffer>,
    state: Res<SharedAppState>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let pointers = Pointers::gather(window, &mouse, &touches);

    for (node, transform, key, mut bg) in &mut keys {
        if pointers.pressed_on(node, transform) {
            let outcome = buf.apply(key.0);
            handle_outcome(outcome, &mut buf, &state);
        }

        let color = if pointers.held_on(node, transform) {
            EYE_COLOR
        } else if pointers.hovering(node, transform) {
            KEY_HOVER
        } else {
            KEY_BG
        };
        if bg.0 != color {
            bg.0 = color;
        }
    }
}

/// Lets a physical keyboard type into the same buffer (development).
fn physical_keyboard(
    mut events: MessageReader<KeyboardInput>,
    mut buf: ResMut<InputBuffer>,
    state: Res<SharedAppState>,
) {
    for ev in events.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        let mut actions = Vec::new();
        match &ev.logical_key {
            Key::Character(s) => {
                for c in s.chars() {
                    if c.is_ascii_alphabetic() {
                        actions.push(KeyAction::Char(c.to_ascii_lowercase()));
                    }
                }
            }
            Key::Space => actions.push(KeyAction::Space),
            Key::Backspace => actions.push(KeyAction::Backspace),
            Key::Enter => actions.push(KeyAction::Enter),
            Key::Escape => actions.push(KeyAction::Cancel),
            _ => {}
        }
        for action in actions {
            let outcome = buf.apply(action);
            handle_outcome(outcome, &mut buf, &state);
        }
    }
}

fn help_timers(time: Res<Time>, mut buf: ResMut<InputBuffer>, state: Res<SharedAppState>) {
    let dt = time.delta_secs();

    if let Some(left) = buf.thanks {
        let left = left - dt;
        if left <= 0.0 {
            buf.thanks = None;
            // Back to the normal eyes; a short happy reaction made the
            // first blinks look more closed than usual.
            state.0.set_emotion(Emotion::Idle);
            state.0.set_screen(AppScreen::Face);
        } else {
            buf.thanks = Some(left);
        }
        return;
    }

    if buf.done {
        return;
    }
    buf.idle += dt;
    if buf.idle > TIMEOUT_SECS {
        state.0.report_help(HelpResult::Timeout);
        state.0.set_screen(AppScreen::Face);
        buf.done = true;
    }
}

fn update_display(
    time: Res<Time>,
    buf: Res<InputBuffer>,
    mut texts: Query<(
        &mut Text,
        Option<&KeyLabel>,
        Option<&TitleText>,
        Option<&InputText>,
    )>,
) {
    let cursor = if buf.thanks.is_none() && (time.elapsed_secs() * 2.0) as u32 % 2 == 0 {
        "_"
    } else {
        ""
    };

    for (mut text, label, title, input) in &mut texts {
        if let Some(KeyLabel(c)) = label {
            let shown = if buf.shift { c.to_ascii_uppercase() } else { *c };
            if text.0.chars().next() != Some(shown) {
                text.0 = shown.to_string();
            }
        } else if title.is_some() {
            let wanted = if buf.thanks.is_some() { THANKS } else { buf.title.as_str() };
            if text.0 != wanted {
                text.0 = wanted.to_string();
            }
        } else if input.is_some() {
            text.0 = format!("{}{}", buf.text, cursor);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_str(buf: &mut InputBuffer, s: &str) {
        for c in s.chars() {
            buf.apply(KeyAction::Char(c));
        }
    }

    #[test]
    fn capitalises_first_letter_only() {
        let mut b = InputBuffer::new();
        type_str(&mut b, "cola");
        assert_eq!(b.text, "Cola");
    }

    #[test]
    fn backspace_and_space() {
        let mut b = InputBuffer::new();
        type_str(&mut b, "tea");
        b.apply(KeyAction::Backspace);
        b.apply(KeyAction::Space);
        b.apply(KeyAction::Space); // no double space
        assert_eq!(b.text, "Te ");
    }

    #[test]
    fn empty_submit_rejected() {
        let mut b = InputBuffer::new();
        assert_eq!(b.apply(KeyAction::Enter), Outcome::None);
        b.apply(KeyAction::Space);
        assert_eq!(b.apply(KeyAction::Enter), Outcome::None);
        type_str(&mut b, "water");
        assert_eq!(b.apply(KeyAction::Enter), Outcome::Submit);
    }
}
