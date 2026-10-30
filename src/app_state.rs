use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const MAX_MESSAGES: usize = 1000;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<State>>,
}

struct State {
    next_id: u64,
    messages: VecDeque<(u64, String)>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(State {
                next_id: 0,
                messages: VecDeque::new(),
            })),
        }
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

    /// Return messages newer than the given ID.
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