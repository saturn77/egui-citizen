# egui-citizen — Task Plan

## Phase 1: Foundation (current)
- [x] Core crate: Citizen trait, CitizenState, CitizenMessage, CitizenRegistry
- [x] Basic example: citizen_dock (three algo tabs + reactive plot + logger)
- [x] Standalone workspace repo with crates/ and examples/ structure
- [ ] Decide: keep egui_mobius_reactive dependency or reimplement Dynamic<T> natively
- [ ] Root README — dense, emphasizing panels, threading, message coordination, dispatcher, real-world use
- [ ] LICENSE file
- [ ] Git init + initial commit

## Phase 2: Serial Plotter Example
- [ ] Serial plotter with citizen-based dock layout:
  - Serial Config panel (citizen) — port selection, baud rate, connect/disconnect
  - Plot panel (citizen) — real-time scrolling waveform plot via egui_plot
  - Logger panel (citizen) — raw data stream + parsed messages
  - Settings panel (citizen) — plot config, sample rate, buffer depth
- [ ] Threaded serial I/O with crossbeam channels
- [ ] RP2350 embedded firmware (TBD):
  - USB serial CDC output
  - Configurable waveform generator (sine, square, triangle, noise)
  - CSV protocol: timestamp, channel, value
- [ ] README showing how to flash RP2350 and run the plotter

## Phase 3: Reactive Layer Decision
- [ ] Evaluate: keep egui_mobius_reactive as dep vs reimplement
  - Pro keep: proven, published, maintained
  - Pro reimplement: zero external deps, citizen owns its full stack
  - Middle ground: thin reactive crate under crates/egui_citizen_reactive
- [ ] If reimplementing, bring over Dynamic<T>, Derived<T>, SignalRegistry

## Phase 4: Dispatcher
- [ ] Formal dispatcher crate (crates/egui_citizen_dispatcher or similar)
- [ ] Typed message routing: citizen-to-citizen and citizen-to-backend
- [ ] Integration with threaded backends (serial, modbus, network)

## Future
- [ ] Citizen group support (multiple independent flip-flop groups)
- [ ] Derive macro for Citizen trait boilerplate
- [ ] Persistence — save/restore citizen state and dock layout across sessions
- [ ] Template repo for cargo-generate
