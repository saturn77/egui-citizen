<div align="center">

# egui-citizen
*Every panel is a citizen. Every citizen has a voice.*

[![egui_version](https://img.shields.io/badge/egui-0.33-blue)](https://github.com/emilk/egui)
[![egui_dock](https://img.shields.io/badge/egui__dock-0.18-purple)](https://github.com/Adanos020/egui_dock)
![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)
[![Latest Version](https://img.shields.io/badge/version-0.1.0-green.svg)](https://crates.io/crates/egui_citizen)
![Rust 2024](https://img.shields.io/badge/rust-2024-blue.svg)

</div>

Modern desktop applications depend primarily on only two functional features to make them enterprise or polished - dockable panels and threading. `egui_citizen` was created to provides these features for egui, evolving from `egui_mobius`, and providing panel lifecycle, threading, and message dispatch for dockable egui applications.

## What it does

`egui_citizen` gives each dock panel a persistent identity, lifecycle state, and
message channel. Panels become **citizens** — they know when they're active, clicked,
selected, or moved, and they communicate through a central dispatcher.

This solves a fundamental problem in `egui_dock`: when multiple panels are visible
across dock nodes, there is no built-in way to track which one the user last
interacted with. Without citizen, panels fight over shared state every frame.
With citizen, state transitions happen exactly once, on click, dispatched as messages.

`egui_citizen` is an evoluation of `egui_mobius` -- which added reactive state to egui panels -- with a focus on signals, slots, lifecycle and message dispatch. Reactive state is still supported, but it's orthogonal to the core citizen pattern.

## The two things that matter

Production egui applications need two things:

1. **Dockable panels** — users rearrange, undock, and re-dock panels freely. This is what gives an application flexibility and longevity.
   The application must handle any layout without state corruption.

2. **Threading** — real time simulation & computation, networking, serial ports, modbus connections, and file I/O all must
   run on background threads. Panels must coordinate with these threads safely.

Everything else — theming, plotting, reactive state — layers on top.
`egui_citizen` is the missing layer between `egui_dock` (docking) and your
threaded backend (I/O), providing the lifecycle and message dispatch that
connects them.

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
│              CitizenRegistry                            │
│          activate() / drain_messages()                  │
│                       │                                 │
│           ┌───────────┴───────────┐                     │
│           ▼                       ▼                     │
│   Other citizens           Backend threads              │
│   (reactive update)    (serial, modbus, compute)        │
└─────────────────────────────────────────────────────────┘
```

## Core types

| Type | Role |
|------|------|
| `Citizen` | Trait. Each dock panel implements this. Identity + lifecycle hooks. |
| `CitizenState` | Per-panel state: active, clicked, selected, moved, location, visible. All reactive (`Dynamic<T>`). |
| `CitizenMessage` | Events: Activated, Deactivated, Clicked, Selected, Moved, VisibilityChanged. |
| `CitizenRegistry` | Manages all citizens. `activate(id)` is a flip-flop — one active, rest off. `drain_messages()` for dispatch. |
| `CitizenId` | String identifier. Panels are addressed by name. |

## Message dispatch

Citizens emit messages. Messages have two consumer types:

1. **Another citizen** — a plot panel observes that an algo panel was activated
   and switches its display. No shared mutable state, no frame-order dependency.
2. **Backend dispatcher** — a service thread receives the activation message and
   starts a computation, opens a serial port, or writes a modbus register. The contents of the message dictate whether the backend thread should spawn, update, or kill a task.

```rust
// In TabViewer::on_tab_button — fires once on click, not every frame
if response.clicked() {
    registry.activate(&id);
}

// After DockArea::show — process messages
for msg in registry.drain_messages() {
    match msg {
        CitizenMessage::Activated { id } => { /* route to panel or backend */ }
        CitizenMessage::Deactivated { id } => { /* cleanup */ }
        _ => {}
    }
}
```

## Real-world use

This architecture was extracted from production engineering tools:

- **saturn-grid-sim** — IEEE 1547 grid support algorithm simulator with freq-watt,
  volt-watt, volt-var, and watt-var panels, live serial telemetry plotting,
  and modbus TCP register access to embedded FPGA hardware.
- **quarri** — Quartus FPGA toolchain launcher with dark theme injection,
  multi-installation management, and system shell integration.

Both applications use `egui_dock` with threaded serial and modbus backends.
The citizen pattern was born from a bug where undocked algo panels raced to
set shared state — the last panel rendered each frame won.

## Workspace structure

```
egui-citizen/
├── crates/
│   └── egui_citizen/        # core crate
├── examples/
│   ├── citizen_dock/        # basic: three tabs + plot + logger
│   └── serial_plotter/      # real-time serial plotter (RP2350)
└── develop/
    ├── task_plan.md
    ├── task_progress.md
    └── task_findings.md
```

## License

MIT
