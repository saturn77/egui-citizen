# The Citizen trait

> **Stub.** Source: `crates/egui_citizen/src/citizen.rs`.

What an identity is (`CitizenId`, a stable string). The three required
methods (`id()`, `state()`, `state_mut()`). The four lifecycle hooks
with default implementations (`on_activate`, `on_deactivate`,
`on_click`, plus `is_active` / `is_selected` readers).

Five-line minimum-viable impl example, drawn from `lib.rs`'s
`SettingsPanel` snippet at lines 87-108.
