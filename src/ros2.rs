use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use futures::{
    executor::LocalPool,
    future::{self, Either},
    stream::StreamExt,
    task::LocalSpawnExt,
};
use r2r::{
    QosProfile,
    pib_face_interfaces::action::{AskText, Confirm},
};

use crate::app_state::AppState;
use crate::confirm_ui::DEFAULT_QUESTION;
use crate::emotion::Emotion;
use crate::help_ui::DEFAULT_TITLE;
use crate::screen::{ConfirmResult, HelpResult};

/// Topic carrying the emotion name as `std_msgs/String`.
pub const EMOTION_TOPIC: &str = "/face/emotion";
/// Action: show a yes/no question and return the answer.
pub const CONFIRM_ACTION: &str = "/face/confirm";
/// Action: show the keyboard and return the typed text.
pub const ASK_TEXT_ACTION: &str = "/face/ask_text";

// Values of `Confirm.action` result.answer.
const ANSWER_NO: u8 = 0;
const ANSWER_YES: u8 = 1;
const ANSWER_TIMEOUT: u8 = 2;

type Str = r2r::std_msgs::msg::String;

/// Poll `poll` every 50 ms until it returns a value.
/// The UI (running in Bevy) reports its result through `AppState`.
async fn wait_for_result<T>(
    node: Arc<Mutex<r2r::Node>>,
    mut poll: impl FnMut() -> Option<T>,
) -> T {
    let mut timer = node
        .lock()
        .unwrap()
        .create_wall_timer(Duration::from_millis(50))
        .expect("could not create timer");

    loop {
        if let Some(value) = poll() {
            return value;
        }
        let _ = timer.tick().await;
    }
}

pub fn run(state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = r2r::Context::create()?;
    let node = Arc::new(Mutex::new(r2r::Node::create(ctx, "pib_face", "")?));

    let mut emotion_sub = node
        .lock()
        .unwrap()
        .subscribe::<Str>(EMOTION_TOPIC, QosProfile::default())?;
    let mut confirm_requests = node
        .lock()
        .unwrap()
        .create_action_server::<Confirm::Action>(CONFIRM_ACTION)?;
    let mut ask_requests = node
        .lock()
        .unwrap()
        .create_action_server::<AskText::Action>(ASK_TEXT_ACTION)?;

    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    // Emotion topic: sets the eyes.
    let emotion_state = state.clone();
    spawner.spawn_local(async move {
        while let Some(msg) = emotion_sub.next().await {
            match msg.data.parse::<Emotion>() {
                Ok(emotion) => {
                    emotion_state.set_emotion(emotion);
                    emotion_state.push_output(format!("emotion: {emotion:?}"));
                }
                Err(err) => {
                    emotion_state.set_emotion(Emotion::Idle);
                    emotion_state.push_output(format!("{err}, falling back to Idle"));
                }
            }
        }
    })?;

    // Action /face/confirm: yes/no question. An empty question uses the
    // default text. Only one screen can be open at a time, so a goal that
    // arrives while another screen is showing is rejected.
    let confirm_state = state.clone();
    let confirm_node = node.clone();
    let confirm_spawner = spawner.clone();
    spawner.spawn_local(async move {
        while let Some(req) = confirm_requests.next().await {
            let question = if req.goal.question.trim().is_empty() {
                DEFAULT_QUESTION.to_string()
            } else {
                req.goal.question.clone()
            };

            if !confirm_state.request_confirm(question.clone()) {
                let _ = req.reject();
                continue;
            }
            confirm_state.push_output(format!("confirm goal: '{question}'"));

            let (mut goal, mut cancel) = match req.accept() {
                Ok(accepted) => accepted,
                Err(_) => {
                    confirm_state.close_screen();
                    continue;
                }
            };

            let state = confirm_state.clone();
            let node = confirm_node.clone();
            let handler = confirm_spawner.spawn_local(async move {
                let _ = goal.publish_feedback(Confirm::Feedback { screen_visible: true });

                let wait = wait_for_result(node, || state.take_confirm_results().into_iter().next());
                futures::pin_mut!(wait);

                match future::select(wait, cancel.next()).await {
                    Either::Left((result, _)) => match result {
                        ConfirmResult::Yes => {
                            let _ = goal.succeed(Confirm::Result { answer: ANSWER_YES });
                        }
                        ConfirmResult::No => {
                            let _ = goal.succeed(Confirm::Result { answer: ANSWER_NO });
                        }
                        ConfirmResult::Timeout => {
                            let _ = goal.abort(Confirm::Result { answer: ANSWER_TIMEOUT });
                        }
                    },
                    Either::Right((request, _)) => {
                        if let Some(request) = request {
                            request.accept();
                        }
                        state.close_screen();
                        let _ = goal.cancel(Confirm::Result { answer: ANSWER_TIMEOUT });
                    }
                }
            });
            if handler.is_err() {
                confirm_state.close_screen();
            }
        }
    })?;

    // Action /face/ask_text: keyboard. An empty title uses the default text.
    let ask_state = state.clone();
    let ask_node = node.clone();
    let ask_spawner = spawner.clone();
    spawner.spawn_local(async move {
        while let Some(req) = ask_requests.next().await {
            let title = if req.goal.title.trim().is_empty() {
                DEFAULT_TITLE.to_string()
            } else {
                req.goal.title.clone()
            };

            if !ask_state.request_help(title.clone()) {
                let _ = req.reject();
                continue;
            }
            ask_state.push_output(format!("ask_text goal: '{title}'"));

            let (mut goal, mut cancel) = match req.accept() {
                Ok(accepted) => accepted,
                Err(_) => {
                    ask_state.close_screen();
                    continue;
                }
            };

            let state = ask_state.clone();
            let node = ask_node.clone();
            let handler = ask_spawner.spawn_local(async move {
                let _ = goal.publish_feedback(AskText::Feedback { screen_visible: true });

                let wait = wait_for_result(node, || state.take_help_results().into_iter().next());
                futures::pin_mut!(wait);

                match future::select(wait, cancel.next()).await {
                    Either::Left((result, _)) => match result {
                        HelpResult::Answered(text) => {
                            let _ = goal.succeed(AskText::Result { success: true, text });
                        }
                        HelpResult::Cancelled | HelpResult::Timeout => {
                            let _ = goal.abort(AskText::Result {
                                success: false,
                                text: String::new(),
                            });
                        }
                    },
                    Either::Right((request, _)) => {
                        if let Some(request) = request {
                            request.accept();
                        }
                        state.close_screen();
                        let _ = goal.cancel(AskText::Result {
                            success: false,
                            text: String::new(),
                        });
                    }
                }
            });
            if handler.is_err() {
                ask_state.close_screen();
            }
        }
    })?;

    // ROS 2 event loop.
    loop {
        node.lock()
            .unwrap()
            .spin_once(Duration::from_millis(10));
        pool.run_until_stalled();
    }
}
