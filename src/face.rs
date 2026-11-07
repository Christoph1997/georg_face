use bevy::{camera::ScalingMode, prelude::*};

use crate::app_state::AppState;
use crate::emotion::Emotion;
use crate::screen::{AppScreen, despawn_all};

// Geometry taken from animations/pib-eyes-animated.gif (1000x750 canvas).
const VIEW_W: f32 = 1000.0;
const VIEW_H: f32 = 750.0;
const EYE_OFFSET_X: f32 = 289.0; // eye centres at 211 / 789 of 1000
const OUTER_HALF_W: f32 = 161.0;
const OUTER_HALF_H: f32 = 235.0;
const INNER_HALF_W: f32 = 106.0;
const INNER_HALF_H: f32 = 180.0;
const GAP_H: f32 = 45.0;
pub const EYE_COLOR: Color = Color::srgb(0.118, 0.608, 0.859); // #1E9BDB

/// Smoothing speed (1/s) for pose transitions.
const POSE_SPEED: f32 = 8.0;

#[derive(Resource, Clone)]
pub struct SharedAppState(pub AppState);

/// Shape parameters of both eyes.
#[derive(Clone, Copy)]
struct Pose {
    /// Vertical scale of the eye (1 = fully open, small = closed).
    open: f32,
    /// Horizontal scale of the eye.
    width: f32,
    /// Scale of the gap between upper and lower half.
    gap: f32,
}

impl Pose {
    fn lerp(self, target: Pose, t: f32) -> Pose {
        Pose {
            open: self.open + (target.open - self.open) * t,
            width: self.width + (target.width - self.width) * t,
            gap: self.gap + (target.gap - self.gap) * t,
        }
    }
}

fn pose_for(emotion: Emotion) -> Pose {
    match emotion {
        Emotion::Idle => Pose { open: 1.0, width: 1.0, gap: 1.0 },
        Emotion::Happy => Pose { open: 0.85, width: 1.05, gap: 1.4 },
        Emotion::Sleepy => Pose { open: 0.35, width: 1.0, gap: 0.6 },
        Emotion::Surprised => Pose { open: 1.15, width: 1.15, gap: 0.4 },
    }
}

#[derive(Resource)]
struct EyeAnimation {
    pose: Pose,
    emotion: Emotion,
    /// Seconds until the next blink starts.
    next_blink: f32,
    /// Seconds since the current blink started (None = not blinking).
    blink_t: Option<f32>,
}

impl Default for EyeAnimation {
    fn default() -> Self {
        Self {
            pose: pose_for(Emotion::Idle),
            emotion: Emotion::Idle,
            next_blink: 2.5,
            blink_t: None,
        }
    }
}

#[derive(Component)]
struct Eye;

#[derive(Component)]
struct GapBar;

const BLINK_DURATION: f32 = 0.18;

pub struct FacePlugin;

impl Plugin for FacePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .init_resource::<EyeAnimation>()
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(AppScreen::Face), spawn_eyes)
            .add_systems(OnExit(AppScreen::Face), despawn_all::<Eye>)
            .add_systems(
                Update,
                (debug_keys, sync_emotion, animate_eyes)
                    .chain()
                    .run_if(in_state(AppScreen::Face)),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    // Fixed virtual canvas that scales with the window.
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: VIEW_W,
                min_height: VIEW_H,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn spawn_eyes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {

    let blue = materials.add(EYE_COLOR);
    let black = materials.add(Color::BLACK);
    let outer = meshes.add(Ellipse::new(OUTER_HALF_W, OUTER_HALF_H));
    let inner = meshes.add(Ellipse::new(INNER_HALF_W, INNER_HALF_H));
    let bar = meshes.add(Rectangle::new(OUTER_HALF_W * 2.0 + 10.0, GAP_H));

    for side in [-1.0_f32, 1.0] {
        commands
            .spawn((
                Eye,
                Transform::from_xyz(side * EYE_OFFSET_X, 0.0, 0.0),
                Visibility::default(),
            ))
            .with_children(|eye| {
                eye.spawn((
                    Mesh2d(outer.clone()),
                    MeshMaterial2d(blue.clone()),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                ));
                eye.spawn((
                    Mesh2d(inner.clone()),
                    MeshMaterial2d(black.clone()),
                    Transform::from_xyz(0.0, 0.0, 1.0),
                ));
                eye.spawn((
                    GapBar,
                    Mesh2d(bar.clone()),
                    MeshMaterial2d(black.clone()),
                    Transform::from_xyz(0.0, 0.0, 2.0),
                ));
            });
    }
}

/// Windows/dev helper: keys 1-4 select an emotion without ROS 2.
fn debug_keys(keys: Res<ButtonInput<KeyCode>>, state: Res<SharedAppState>) {
    let map = [
        (KeyCode::Digit1, Emotion::ALL[0]),
        (KeyCode::Digit2, Emotion::ALL[1]),
        (KeyCode::Digit3, Emotion::ALL[2]),
        (KeyCode::Digit4, Emotion::ALL[3]),
    ];
    for (key, emotion) in map {
        if keys.just_pressed(key) {
            state.0.set_emotion(emotion);
        }
    }
}

/// Happy is a short reaction (e.g. after the help answer), then back to idle.
const HAPPY_SECS: f32 = 4.0;

fn sync_emotion(
    time: Res<Time>,
    state: Res<SharedAppState>,
    mut anim: ResMut<EyeAnimation>,
    mut happy_for: Local<f32>,
) {
    let wanted = state.0.emotion();
    if wanted != anim.emotion {
        anim.emotion = wanted;
        *happy_for = 0.0;
    }
    if wanted == Emotion::Happy {
        *happy_for += time.delta_secs();
        if *happy_for > HAPPY_SECS {
            state.0.set_emotion(Emotion::Idle);
        }
    }
}

fn animate_eyes(
    time: Res<Time>,
    mut anim: ResMut<EyeAnimation>,
    mut eyes: Query<(&mut Transform, &Children), (With<Eye>, Without<GapBar>)>,
    mut bars: Query<&mut Transform, (With<GapBar>, Without<Eye>)>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    // Smooth transition toward the target pose.
    let target = pose_for(anim.emotion);
    let k = 1.0 - (-POSE_SPEED * dt).exp();
    anim.pose = anim.pose.lerp(target, k);

    // Blink scheduling: short, irregular blinks.
    match anim.blink_t {
        Some(bt) => {
            let bt = bt + dt;
            anim.blink_t = if bt >= BLINK_DURATION { None } else { Some(bt) };
        }
        None => {
            anim.next_blink -= dt;
            if anim.next_blink <= 0.0 {
                anim.blink_t = Some(0.0);
                // Pseudo-random 2.5 - 5.5 s from the clock, no rand dependency.
                anim.next_blink = 2.5 + ((t * 12.9898).sin() * 43758.5453).fract().abs() * 3.0;
            }
        }
    }

    // Triangle wave: 1 -> 0 -> 1 over the blink.
    let blink = match anim.blink_t {
        Some(bt) => (2.0 * bt / BLINK_DURATION - 1.0).abs(),
        None => 1.0,
    };

    let pose = anim.pose;
    let open = (pose.open * (0.1 + 0.9 * blink)).max(0.05);

    // Slight gaze drift.
    let gaze = Vec2::new((t * 0.7).sin() * 12.0, (t * 0.5).cos() * 6.0);

    // Gap disappears as the eye closes.
    let gap = pose.gap * ((open - 0.1) / 0.9).clamp(0.0, 1.0);

    for (mut tf, children) in &mut eyes {
        tf.scale = Vec3::new(pose.width, open, 1.0);
        tf.translation.y = gaze.y;
        // Preserve the horizontal side offset, add gaze on top.
        let side = tf.translation.x.signum();
        tf.translation.x = side * EYE_OFFSET_X + gaze.x;

        for child in children.iter() {
            if let Ok(mut bar) = bars.get_mut(child) {
                bar.scale.y = gap;
            }
        }
    }
}
