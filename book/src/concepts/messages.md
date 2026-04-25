# CitizenMessage — the backend bridge

> **Stub.** Source: `crates/egui_citizen/src/message.rs`.

The six variants: `Activated`, `Deactivated`, `Clicked`, `Selected`,
`Moved`, `VisibilityChanged`. Cover what each one is emitted for and
what fields it carries.

Forwarding pattern: drain messages once per frame after
`DockArea::show()`, send them onto a `crossbeam_channel` to a backend
thread. Worked example in `lib.rs:115-138`.

Extension pattern: wrap `CitizenMessage` in your own app-level enum
(e.g. `AppMessage::Citizen(CitizenMessage)`) when your app has its own
message vocabulary. CopperForge uses this exact shape.
