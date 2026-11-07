use bevy::prelude::*;

use crate::face::SharedAppState;

/// Which screen pib currently shows.
#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AppScreen {
    /// The animated eyes.
    #[default]
    Face,
    /// "Please enter your favorite drink" with on-screen keyboard.
    DrinkRequest,
    /// A question from the task manager with Yes / No touch buttons.
    Confirm,
}

/// Outcome of a confirmation question, reported back to ROS 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmResult {
    Yes,
    No,
    Timeout,
}

/// Outcome of a help request, reported back to ROS 2.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HelpResult {
    Answered(String),
    Cancelled,
    Timeout,
}

/// Removes every entity carrying marker `M` (and its children).
/// Used as the `OnExit` system of each screen.
pub fn despawn_all<M: Component>(mut commands: Commands, entities: Query<Entity, With<M>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

/// Seconds for each half of the transition (fade out, then fade in).
const FADE_SECS: f32 = 0.3;

#[derive(Default, PartialEq, Eq)]
enum FadePhase {
    #[default]
    Idle,
    Out,
    In,
}

/// Progress of the fade-through-black between screens.
#[derive(Resource, Default)]
struct Fade {
    phase: FadePhase,
    /// 0 = screen fully visible, 1 = fully black.
    amount: f32,
}

/// Full-screen black overlay used for the fade.
#[derive(Component)]
struct FadeOverlay;

pub struct ScreenPlugin;

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppScreen>()
            .init_resource::<Fade>()
            .add_systems(Startup, spawn_overlay)
            .add_systems(Update, (mirror_screen, apply_fade).chain());
    }
}

fn spawn_overlay(mut commands: Commands) {
    commands.spawn((
        FadeOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        GlobalZIndex(1000),
    ));
}

/// Bevy state follows the screen requested in the shared `AppState`
/// (which the ROS 2 thread can change). The switch happens in the middle
/// of a fade through black instead of as a hard cut.
fn mirror_screen(
    time: Res<Time>,
    state: Res<SharedAppState>,
    current: Res<State<AppScreen>>,
    mut next: ResMut<NextState<AppScreen>>,
    mut fade: ResMut<Fade>,
) {
    let step = time.delta_secs() / FADE_SECS;
    let wanted = state.0.screen();

    match fade.phase {
        FadePhase::Idle => {
            if *current.get() != wanted {
                fade.phase = FadePhase::Out;
            }
        }
        FadePhase::Out => {
            fade.amount = (fade.amount + step).min(1.0);
            if fade.amount >= 1.0 {
                // Screen is fully black: swap now, using the latest request.
                next.set(wanted);
                fade.phase = FadePhase::In;
            }
        }
        FadePhase::In => {
            fade.amount = (fade.amount - step).max(0.0);
            if fade.amount <= 0.0 {
                fade.phase = FadePhase::Idle;
            }
        }
    }
}

fn apply_fade(fade: Res<Fade>, mut overlay: Query<&mut BackgroundColor, With<FadeOverlay>>) {
    // Smoothstep easing so the fade starts and ends gently.
    let t = fade.amount;
    let alpha = t * t * (3.0 - 2.0 * t);
    for mut bg in &mut overlay {
        bg.0 = Color::srgba(0.0, 0.0, 0.0, alpha);
    }
}
