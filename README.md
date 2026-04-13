<div align="center">

# egui-citizen

Structuring egui applications with dockable panels as first class Citizens. 

[![egui](https://img.shields.io/badge/egui-0.33-blue)](https://github.com/emilk/egui)
[![egui_dock](https://img.shields.io/badge/egui__dock-0.18-purple)](https://github.com/Adanos020/egui_dock)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
![Rust 2024](https://img.shields.io/badge/rust-2024-blue.svg)

</div>

## Introduction

Overall how does one structure an egui application that is flexible and easy to maintain with dockable panels? This is an essential question that really rquires consideration of the whole entire gui application. 

**egui_mobius** attempted to setup a framework for messaging
and reactive updates, borrowing some ideas that have been
long established such as signals, slots, threads, and dispatching. 

This platform is really an evolution of egui_mobius. It focuses on structuring an app planning from the very beginning of the application anticipating backend processing needs and the entire architectural flow chain required to accomplish that. 

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
    dispatcher.activate(&id);  // one-hot: one active, rest off
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
| `Dispatcher` | Manages citizens. `activate()` is an encoded set/reset. `drain_messages()` for Elm-style dispatch. |
| `CitizenId` | String identifier. Panels are addressed by name. |

## Two consumer paths

Every field in `CitizenState` is a `Dynamic<T>` from [egui_mobius_reactive](https://github.com/saturn77/egui_mobius). When the Dispatcher calls `state.active.set(true)`, any panel holding a clone of that state sees the change immediately via `.get()` — no polling, no message checking, no frame delay. This is what makes two consumer paths possible:

1. **Other panels** — read `CitizenState` directly via `Dynamic<T>`. Reactive, immediate, zero wiring.

2. **Backend threads** — receive `CitizenMessage` via `drain_messages()` and route through channels to serial ports, network connections, or compute tasks.

```
Tab click → dispatcher.activate("alpha")
  ├─ Path 1: alpha.state.active = true   (reactive, immediate via Dynamic<T>)
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
each frame won. The fix: persistent panel identity with one-hot activation
and message dispatch.

## Applications built with egui-citizen

One of the focus areas for using egui-citizen is internal tooling that span out into medium sized applications that are not necessarily full enterprise apps. The egui-citizen architecture is proving itself with multiple internal tools that have been built and provides a consistent design pattern for rapidly building out such apps. 

- **[CopperForge](https://github.com/Atlantix-EDA/CopperForge)** — KiCad companion tool for project management, gerber viewing, and fabrication output. 12 citizen panels, LayerStore-based rendering, Tokyo Night Storm theme.

- **saturn-grid-sim** — A real time testing application, with communications and plotting. It is a IEEE 1547 grid support algorithm simulator with freq-watt, volt-watt, volt-var, and watt-var panels, live serial telemetry, and modbus TCP register access to embedded FPGA hardware.

- **[quarri](https://github.com/saturn77/quarri)** — Quartus FPGA toolchain launcher with dark theme injection and multi-installation management. A more basic type of application but one that has utility.

## Examples

**citizen_dock** — basic pattern: three algorithm tabs, a reactive plot panel, and a message logger.

```bash
cargo run -p citizen_dock
```

**citizen_fetch** — backend threading: HTTP fetch on a background thread, auto-refresh with random images from picsum.photos, four dockable panels showing the full citizen → backend → UI response cycle.

```bash
cargo run -p citizen_fetch
```

## Getting Started

See [docs/getting-started.md](docs/getting-started.md) for a step-by-step guide.

## Dependencies

| Crate | Purpose |
|-------|---------|
| egui | UI framework |
| egui_mobius_reactive | `Dynamic<T>` reactive state for CitizenState fields |

The core crate has no dependency on `egui_dock`, `eframe`, or any rendering backend. It provides the trait and dispatch — you wire it into your dock layout.

## License

MIT
