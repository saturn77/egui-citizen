//! Central dispatcher for citizen lifecycle management and message routing.
//!
//! The dispatcher is the hub between panels (shared state) and backend
//! threads (signals/slots). Panels read state directly. Threads receive
//! signals and send responses back on slots.

use std::collections::HashMap;

use crate::message::{CitizenId, CitizenMessage};
use crate::state::CitizenState;

/// Central dispatcher that manages citizen state and routes messages.
///
/// - **Panel side**: panels read `CitizenState` directly (shared, reactive).
/// - **Backend side**: threads receive `CitizenMessage` signals and post
///   responses back via `send()`.
///
/// `activate()` is the core operation — a set/reset flip-flop that sets
/// one citizen active, deactivates the rest, and emits messages for both sides.
pub struct Dispatcher {
    citizens: HashMap<CitizenId, CitizenState>,
    message_queue: Vec<CitizenMessage>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            citizens: HashMap::new(),
            message_queue: Vec::new(),
        }
    }

    /// Register a citizen with the dispatcher. Returns its state handle.
    pub fn register(&mut self, id: CitizenId) -> CitizenState {
        let state = CitizenState::new();
        self.citizens.insert(id, state.clone());
        state
    }

    /// Get the state of a registered citizen.
    pub fn get(&self, id: &CitizenId) -> Option<&CitizenState> {
        self.citizens.get(id)
    }

    /// Push a message onto the queue for processing.
    pub fn send(&mut self, message: CitizenMessage) {
        self.message_queue.push(message);
    }

    /// Activate a citizen by ID, deactivating all others.
    ///
    /// Set/reset flip-flop — exactly one citizen is active at a time.
    /// Emits Activated and Deactivated messages for dispatch.
    pub fn activate(&mut self, id: &CitizenId) {
        for (cid, state) in &self.citizens {
            if cid == id {
                state.active.set(true);
                self.message_queue.push(CitizenMessage::Activated { id: cid.clone() });
            } else if state.active.get() {
                state.active.set(false);
                self.message_queue.push(CitizenMessage::Deactivated { id: cid.clone() });
            }
        }
    }

    /// Drain all pending messages, returning them for processing.
    pub fn drain_messages(&mut self) -> Vec<CitizenMessage> {
        std::mem::take(&mut self.message_queue)
    }

    /// Number of registered citizens.
    pub fn len(&self) -> usize {
        self.citizens.len()
    }

    /// Whether the dispatcher has no citizens.
    pub fn is_empty(&self) -> bool {
        self.citizens.is_empty()
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}
