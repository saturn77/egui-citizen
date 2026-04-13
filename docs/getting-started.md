# Getting Started with egui-citizen

## The problem

In `egui_dock`, every visible panel's `ui()` runs every frame. If two panels write to the same state, whichever renders last wins. This is a per-frame race condition — and it's why egui-citizen exists.

The fix: move state transitions from `ui()` to `on_tab_button()`, which fires once, on click, only for the clicked panel.

## Add to your project

```toml
[dependencies]
egui_citizen = { git = "https://github.com/saturn77/egui-citizen.git" }
egui = "0.33"
egui_dock = "0.18"
```

## Step 1: Create a Dispatcher and register panels

```rust
use egui_citizen::{Dispatcher, CitizenId};

let mut dispatcher = Dispatcher::new();
dispatcher.register(CitizenId::new("settings"));
dispatcher.register(CitizenId::new("plot"));
dispatcher.register(CitizenId::new("logger"));

// Activate one by default
dispatcher.activate(&CitizenId::new("plot"));
let _ = dispatcher.drain_messages(); // clear startup messages
```

The Dispatcher tracks which citizen is active and queues lifecycle messages. `activate()` is a flip-flop — one active, rest off.

## Step 2: Wire on_tab_button in your TabViewer

```rust
impl egui_dock::TabViewer for MyTabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn on_tab_button(&mut self, tab: &mut Tab, response: &egui::Response) {
        if response.clicked() {
            self.dispatcher.activate(&tab.citizen_id());
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
        tab.show(ui);
    }
}
```

This is the key — `on_tab_button` fires once on click, not every frame. That's what eliminates the race condition.

## Step 3: Drain messages after rendering

```rust
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Render the dock area
        DockArea::new(&mut self.dock_state)
            .show(ctx, &mut tab_viewer);

        // Drain citizen lifecycle messages
        for msg in self.dispatcher.drain_messages() {
            match msg {
                CitizenMessage::Activated { id } => {
                    // A panel became active — update state, notify backend
                }
                CitizenMessage::Deactivated { id } => {
                    // A panel lost focus — cleanup if needed
                }
                _ => {}
            }
        }
    }
}
```

Messages are consumed once per frame, after all panels have rendered. Order doesn't matter.

## Step 4: Implement the Citizen trait on your panels

```rust
use egui_citizen::{Citizen, CitizenId, CitizenState};

struct SettingsPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    // your panel-specific fields
}

impl SettingsPanel {
    fn new(state: CitizenState) -> Self {
        Self {
            citizen_id: CitizenId::new("settings"),
            citizen_state: state,
        }
    }
}

impl Citizen for SettingsPanel {
    fn id(&self) -> &CitizenId { &self.citizen_id }
    fn state(&self) -> &CitizenState { &self.citizen_state }
    fn state_mut(&mut self) -> &mut CitizenState { &mut self.citizen_state }
}
```

`CitizenState` fields are reactive (`Dynamic<T>`) — other panels can read them without polling.

## Step 5: Add a backend thread

Route citizen messages to a background thread via a channel:

```rust
use crossbeam_channel::unbounded;

// At startup
let (tx, rx) = unbounded();
std::thread::spawn(move || {
    for msg in rx {
        match msg {
            CitizenMessage::Activated { id } if id.0 == "fetch" => {
                // do HTTP request, serial read, computation, etc.
            }
            _ => {}
        }
    }
});

// In update loop, after drain_messages:
for msg in dispatcher.drain_messages() {
    let _ = tx.send(msg.clone());
}
```

The UI stays responsive — all heavy work happens on the backend thread.

## Working examples

**citizen_dock** — basic pattern with three algorithm tabs, a plot panel, and a logger:

```bash
cargo run -p citizen_dock
```

**citizen_fetch** — backend threading with HTTP requests and auto-refreshing random images:

```bash
cargo run -p citizen_fetch
```

## Scope

egui-citizen manages **panel lifecycle** — which panel is active, which got deactivated, which needs to notify a backend. It does not manage widgets inside a panel. Your buttons, sliders, and text fields are your business. Citizen draws the line at the panel boundary.

## Real-world usage

- **[CopperForge](https://github.com/Atlantix-EDA/CopperForge)** — 12 citizen panels, KiCad PCB companion tool
- **saturn-grid-sim** — IEEE 1547 simulator with serial telemetry and modbus
- **quarri** — Quartus FPGA toolchain launcher
