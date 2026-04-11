//! The core Citizen trait.

use crate::message::CitizenId;
use crate::state::CitizenState;

/// A UI element with persistent identity and lifecycle state.
///
/// Any widget, panel, or dockable that needs to track its own
/// lifecycle across frames implements `Citizen`. The registry
/// uses this trait to manage state and dispatch messages.
///
/// # Example
///
/// ```ignore
/// struct FreqWattTab {
///     citizen_id: CitizenId,
///     citizen_state: CitizenState,
/// }
///
/// impl Citizen for FreqWattTab {
///     fn id(&self) -> &CitizenId { &self.citizen_id }
///     fn state(&self) -> &CitizenState { &self.citizen_state }
///     fn state_mut(&mut self) -> &mut CitizenState { &mut self.citizen_state }
/// }
/// ```
pub trait Citizen {
    /// Unique identifier for this citizen.
    fn id(&self) -> &CitizenId;

    /// Immutable access to lifecycle state.
    fn state(&self) -> &CitizenState;

    /// Mutable access to lifecycle state.
    fn state_mut(&mut self) -> &mut CitizenState;

    /// Called when this citizen becomes active in its group.
    /// Default implementation sets `state.active` to true.
    fn on_activate(&mut self) {
        self.state_mut().active.set(true);
    }

    /// Called when this citizen is deactivated.
    /// Default implementation sets `state.active` to false.
    fn on_deactivate(&mut self) {
        self.state_mut().active.set(false);
    }

    /// Called when this citizen is clicked.
    /// Default implementation sets `state.clicked` to true for one frame.
    fn on_click(&mut self) {
        self.state_mut().clicked.set(true);
    }

    /// Whether this citizen is currently active.
    fn is_active(&self) -> bool {
        self.state().active.get()
    }

    /// Whether this citizen is currently selected.
    fn is_selected(&self) -> bool {
        self.state().selected.get()
    }
}
