//! # egui_citizen
//!
//! First-class dock panel lifecycle and state tracking for egui.
//!
//! In egui's immediate mode, dock panels have no persistent identity — they
//! exist only during the frame they are drawn. When multiple panels are visible
//! simultaneously (e.g., undocked into separate nodes), there is no built-in way
//! to know which one the user last interacted with.
//!
//! `Citizen` solves this by giving each dock panel a persistent identity and
//! lifecycle state (active, clicked, selected, moved, location) that survives
//! across frames, with state changes dispatched as messages.
//!
//! While the `Citizen` trait can be applied to any widget, it is most useful
//! when applied to **dock panels** — these are the natural unit of identity
//! in an `egui_dock` layout. A panel is the "citizen"; widgets inside it are
//! its implementation details.
//!
//! ## Core Concepts
//!
//! - **`Citizen` trait**: A dock panel implements this to declare its identity
//!   and respond to lifecycle events.
//! - **`CitizenState`**: Per-panel lifecycle state (active, clicked, selected,
//!   moved, location, visible) using reactive `Dynamic<T>`. Panels read this directly.
//! - **`CitizenMessage`**: Lifecycle events (Activated, Deactivated, Clicked, etc.)
//!   routed to backend threads as signals.
//! - **`Dispatcher`**: The hub. Panels read shared state from it. Backend threads
//!   receive signals and post responses back. `activate()` is the flip-flop.
//!
//! ## Two consumer paths
//!
//! - **Panels** read `CitizenState` directly (shared, reactive, no wiring).
//! - **Threads** receive `CitizenMessage` via `drain_messages()` (signals/slots).

mod citizen;
pub mod dispatcher;
pub mod message;
mod state;

pub use citizen::Citizen;
pub use dispatcher::Dispatcher;
pub use message::{CitizenId, CitizenMessage};
pub use state::CitizenState;
