//! Persistent lifecycle state for a citizen widget.

use egui_mobius_reactive::Dynamic;

/// Tracks the lifecycle state of a single citizen across frames.
///
/// Each field is a reactive `Dynamic<T>` so dependents (plots, panels, etc.)
/// can observe changes without polling.
#[derive(Clone)]
pub struct CitizenState {
    /// Whether this citizen is the "active" one in its group (e.g., selected tab).
    pub active: Dynamic<bool>,

    /// True during the frame the citizen was clicked.
    pub clicked: Dynamic<bool>,

    /// Persistent selection state (toggle on/off).
    pub selected: Dynamic<bool>,

    /// True if the citizen has been moved (e.g., docked to a new location).
    pub moved: Dynamic<bool>,

    /// Current position in the UI, if applicable.
    pub location: Dynamic<[f32; 2]>,

    /// Whether the citizen is currently visible / rendered.
    pub visible: Dynamic<bool>,
}

impl CitizenState {
    pub fn new() -> Self {
        Self {
            active: Dynamic::new(false),
            clicked: Dynamic::new(false),
            selected: Dynamic::new(false),
            moved: Dynamic::new(false),
            location: Dynamic::new([0.0, 0.0]),
            visible: Dynamic::new(false),
        }
    }
}

impl Default for CitizenState {
    fn default() -> Self {
        Self::new()
    }
}
