<div align="center">

# egui-citizen

Panel lifecycle and message dispatch for dockable egui applications.

[![egui](https://img.shields.io/badge/egui-0.33-blue)](https://github.com/emilk/egui)
[![egui_dock](https://img.shields.io/badge/egui__dock-0.18-purple)](https://github.com/Adanos020/egui_dock)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
![Rust 2024](https://img.shields.io/badge/rust-2024-blue.svg)

</div>

## The problem

In `egui_dock`, when multiple panels are visible across dock nodes, there is no built-in way to track which panel the user last interacted with. Panels fight over shared state every frame — whichever renders last wins. This is a per-frame race condition that gets worse as you add panels.

## What egui-citizen does

Each dock panel gets a persistent identity (`CitizenId`), lifecycle state (`CitizenState`), and participates in message dispatch through a central `Dispatcher`. State transitions happen exactly once, on click — not every frame.

```rust
// Register panels at startup
let mut dispatcher = Dispatcher::new();
dispatcher.register(CitizenId::new("freq_watt"));
dispatcher.register(CitizenId::new("volt_watt"));
dispatcher.register(CitizenId::new("plot"));

// In TabViewer::on_tab_button — fires once on click
if response.clicked() {
    dispatcher.activate(&id);  // flip-flop: one active, rest off
}

// After DockArea::show — process messages
for msg in dispatcher.drain_messages() {
    match msg {
        CitizenMessage::Activated { id } => { /* route to panel or backend */ }
        CitizenMessage::Deactivated { id } => { /* cleanup */ }
        _ => {}
    }
}
```

## Core types

| Type | Purpose |
|------|---------|
| `Citizen` | Trait each dock panel implements. Identity + lifecycle hooks. |
| `CitizenState` | Per-panel reactive state: active, clicked, selected, moved, visible. |
| `CitizenMessage` | Lifecycle events dispatched through the message queue. |
| `Dispatcher` | Manages citizens. `activate()` is a flip-flop. `drain_messages()` for Elm-style dispatch. |
| `CitizenId` | String identifier. Panels are addressed by name. |

## Two consumer paths

Citizens emit messages. There are two consumers:

1. **Other panels** — a plot panel reads another citizen's `CitizenState` directly via reactive `Dynamic<T>`. No polling, no shared mutable state.

2. **Backend threads** — a service thread receives `CitizenMessage` via `drain_messages()` and spawns computation, opens a serial port, or writes a modbus register.

```
Tab click → dispatcher.activate("alpha")
  ├─ Path 1: alpha.state.active = true   (reactive, immediate)
  │           beta.state.active = false
  └─ Path 2: queue ← [Activated{alpha}, Deactivated{beta}]
             drain_messages() → route to backends
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    egui_dock layout                     │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐            │
│  │ Panel A   │  │ Panel B   │  │ Panel C   │   ...      │
│  │ (citizen) │  │ (citizen) │  │ (citizen) │            │
│  └─────┬─────┘  └─────┬─────┘  └─────┬─────┘            │
│        │              │              │                  │
│        └──────────────┼──────────────┘                  │
│                       ▼                                 │
│                  Dispatcher                             │
│          activate() / drain_messages()                  │
│                       │                                 │
│           ┌───────────┴───────────┐                     │
│           ▼                       ▼                     │
│   Other citizens           Backend threads              │
│   (reactive state)     (serial, modbus, compute)        │
└─────────────────────────────────────────────────────────┘
```

## Origin

The citizen pattern was extracted from production engineering tools where
undocked algo panels raced to set shared state — the last panel rendered
each frame won. The fix: persistent panel identity with flip-flop activation
and message dispatch.

## Applications built with egui-citizen

- **[CopperForge](https://github.com/Atlantix-EDA/CopperForge)** — KiCad companion tool for project management, gerber viewing, and fabrication output. 12 citizen panels, LayerStore-based rendering, Tokyo Night Storm theme.

- **saturn-grid-sim** — IEEE 1547 grid support algorithm simulator with freq-watt, volt-watt, volt-var, and watt-var panels, live serial telemetry, and modbus TCP register access to embedded FPGA hardware.

- **quarri** — Quartus FPGA toolchain launcher with dark theme injection and multi-installation management.

## Example

The `citizen_dock` example demonstrates the basic pattern: three algorithm tabs, a reactive plot panel, and a message logger.

```bash
cargo run --example citizen_dock
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| egui | UI framework |
| egui_mobius_reactive | `Dynamic<T>` reactive state for CitizenState fields |

The core crate has no dependency on `egui_dock`, `eframe`, or any rendering backend. It provides the trait and dispatch — you wire it into your dock layout.

## License

MIT
