//! Messages emitted by citizens when their lifecycle state changes.

/// A lifecycle event emitted by a citizen.
///
/// These are dispatched to the `CitizenRegistry` which routes them
/// to handlers — Elm-style update loop.
#[derive(Debug, Clone)]
pub enum CitizenMessage {
    /// Citizen became the active member of its group.
    Activated { id: CitizenId },

    /// Citizen was deactivated (another in the group became active).
    Deactivated { id: CitizenId },

    /// Citizen was clicked this frame.
    Clicked { id: CitizenId },

    /// Citizen selection toggled.
    Selected { id: CitizenId, selected: bool },

    /// Citizen was moved to a new location.
    Moved { id: CitizenId, location: [f32; 2] },

    /// Citizen visibility changed.
    VisibilityChanged { id: CitizenId, visible: bool },
}

/// Unique identifier for a citizen.
///
/// Wraps a string ID so citizens can be addressed by name
/// (e.g., "freq_watt", "plot", "volt_var").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CitizenId(pub String);

impl CitizenId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for CitizenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
