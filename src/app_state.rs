use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::emotion::Emotion;
use crate::screen::{AppScreen, ConfirmResult, HelpResult};

const MAX_MESSAGES: usize = 1000;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<State>>,
}

struct State {
    next_id: u64,
    messages: VecDeque<(u64, String)>,
    emotion: Emotion,
    screen: AppScreen,
    /// Help results waiting to be published to ROS 2.
    results: Vec<HelpResult>,
    /// Text shown above the keyboard on the help screen.
    help_title: String,
    /// Question shown on the confirmation screen.
    confirm_question: String,
    /// Confirmation answers waiting to be published to ROS 2.
    confirm_results: Vec<ConfirmResult>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(State {
                next_id: 0,
                messages: VecDeque::new(),
                emotion: Emotion::default(),
                screen: AppScreen::default(),
                results: Vec::new(),
                help_title: String::new(),
                confirm_question: String::new(),
                confirm_results: Vec::new(),
            })),
        }
    }

    /// Entry point for the task manager's yes/no question.
    /// Shows `question` with Yes / No buttons unless another screen is
    /// already showing. Returns true if the screen was switched.
    pub fn request_confirm(&self, question: impl Into<String>) -> bool {
        let mut state = self.inner.lock().unwrap();
        if state.screen == AppScreen::Face {
            state.confirm_question = question.into();
            state.screen = AppScreen::Confirm;
            true
        } else {
            false
        }
    }

    pub fn confirm_question(&self) -> String {
        self.inner.lock().unwrap().confirm_question.clone()
    }

    /// Queue the answer to a confirmation question for publishing to ROS 2.
    pub fn report_confirm(&self, result: ConfirmResult) {
        self.inner.lock().unwrap().confirm_results.push(result);
    }

    /// Take all pending confirmation answers (called by the ROS 2 thread).
    #[allow(dead_code)] // only used on Linux, where ROS 2 is available
    pub fn take_confirm_results(&self) -> Vec<ConfirmResult> {
        std::mem::take(&mut self.inner.lock().unwrap().confirm_results)
    }

    pub fn screen(&self) -> AppScreen {
        self.inner.lock().unwrap().screen
    }

    pub fn set_screen(&self, screen: AppScreen) {
        self.inner.lock().unwrap().screen = screen;
    }

    /// Back to the eyes, e.g. when the task manager cancels its goal.
    /// Results the UI reported in the meantime are dropped.
    #[allow(dead_code)] // only used on Linux, where ROS 2 is available
    pub fn close_screen(&self) {
        let mut state = self.inner.lock().unwrap();
        state.screen = AppScreen::Face;
        state.results.clear();
        state.confirm_results.clear();
    }

    /// Entry point for every trigger source (ROS 2 topic, debug key, ...).
    /// Switches to the help screen unless it is already showing.
    /// Returns true if the screen was switched.
    pub fn request_help(&self, title: impl Into<String>) -> bool {
        let mut state = self.inner.lock().unwrap();
        if state.screen == AppScreen::Face {
            state.help_title = title.into();
            state.screen = AppScreen::DrinkRequest;
            true
        } else {
            false
        }
    }

    /// Text shown on the help screen (set by the last accepted request).
    pub fn help_title(&self) -> String {
        self.inner.lock().unwrap().help_title.clone()
    }

    /// Queue the result of a help request for publishing to ROS 2.
    pub fn report_help(&self, result: HelpResult) {
        self.inner.lock().unwrap().results.push(result);
    }

    /// Take all pending help results (called by the ROS 2 thread).
    #[allow(dead_code)] // only used on Linux, where ROS 2 is available
    pub fn take_help_results(&self) -> Vec<HelpResult> {
        std::mem::take(&mut self.inner.lock().unwrap().results)
    }

    /// Set the emotion the face should show (latest value wins).
    pub fn set_emotion(&self, emotion: Emotion) {
        self.inner.lock().unwrap().emotion = emotion;
    }

    /// Currently requested emotion.
    pub fn emotion(&self) -> Emotion {
        self.inner.lock().unwrap().emotion
    }

    /// Add an output message from any application component.
    pub fn push_output(&self, message: impl Into<String>) {
        let mut state = self.inner.lock().unwrap();

        let id = state.next_id;
        state.next_id += 1;

        state.messages.push_back((id, message.into()));

        while state.messages.len() > MAX_MESSAGES {
            state.messages.pop_front();
        }
    }

    /// Return messages newer than the given ID (for a debug overlay).
    #[allow(dead_code)]
    pub fn messages_after(&self, last_id: u64) -> Vec<(u64, String)> {
        let state = self.inner.lock().unwrap();

        state
            .messages
            .iter()
            .filter(|(id, _)| *id > last_id)
            .cloned()
            .collect()
    }
}