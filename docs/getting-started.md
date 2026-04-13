# Getting Started with egui-citizen

## Why egui-citizen

Building a real egui application with dockable panels raises questions that egui and egui_dock don't answer on their own:

- How do you track which panel is active when multiple panels are visible?
- How do panels coordinate with backend threads (serial, network, compute)?
- How do you structure the app so adding panels doesn't turn into spaghetti?

egui-citizen provides the architectural layer for this. Each panel gets a persistent identity, lifecycle state, and participates in message dispatch through a central Dispatcher. The result is a consistent structure that scales from 3 panels to 12+ without the codebase degrading.

The pattern emerged from building [quarri](https://github.com/saturn77/quarri), a Quartus FPGA toolchain launcher. The clean delegation in quarri's `main.rs` — where each panel is a self-contained unit and the TabViewer just routes to them — proved that structuring panels as first-class entities made the whole app maintainable. That architecture became egui-citizen.

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

The Dispatcher tracks which citizen is active and queues lifecycle messages. `activate()` is an encoded set/reset — one active, rest off.

## Step 2: Wire on_tab_button in your TabViewer

`TabViewer` is the bridge between `egui_dock` and your application. It's the trait you implement to tell egui_dock how to render each tab panel — what title to show, what content to draw, and how to respond when a tab header is clicked. This is where citizen activation happens.

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

`on_tab_button` is an egui_dock callback that fires once when a user clicks a tab header — not every frame like `ui()`. This is the critical distinction. By putting citizen activation here instead of inside `ui()`, state transitions happen exactly once per click, eliminating per-frame race conditions between visible panels.

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

There are two approaches for backend coordination:

### Option A: Crossbeam channels (simple, explicit)

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

This is what the `citizen_fetch` example uses. Direct, visible, no magic.

### Option B: egui_mobius signals and slots (typed, reactive)

For more structured backend communication, [egui_mobius](https://github.com/saturn77/egui_mobius) provides typed `Signal<T>` / `Slot<T>` pairs that handle thread-safe message passing with automatic wiring:

```rust
use egui_mobius::factory;
use egui_mobius::signals::Signal;
use egui_mobius::slot::Slot;

// Create a typed signal/slot pair
let (signal, mut slot) = factory::create_signal_slot::<MyRequest>();

// Slot runs on a background thread
slot.start(move |request: MyRequest| {
    // handle request — runs off the UI thread
});

// Signal fires from the UI
signal.emit(MyRequest::Fetch { url });
```

This is useful when you have multiple backend services (serial, modbus, network) that each need their own typed message channel. egui-citizen provides the panel lifecycle; egui_mobius provides the backend threading primitives. They compose naturally.

The UI stays responsive with either approach — all heavy work happens off the UI thread.

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
- **[quarri](https://github.com/saturn77/quarri)** — Quartus FPGA toolchain launcher

## Complete example

The full getting started code as a single compilable file. Run it with 

```bash
cargo run -p getting_started
```

The full file is : 

```rust
use eframe::egui;
use egui::Color32;
use egui_dock::{DockArea, DockState, NodeIndex};
use egui_citizen::{Citizen, CitizenId, CitizenState, CitizenMessage, Dispatcher};

// ── Panel structs implementing Citizen ──────────────────────────────────

struct ConfigPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    value: f32,
}

impl ConfigPanel {
    fn new(state: CitizenState) -> Self {
        Self { citizen_id: CitizenId::new("config"), citizen_state: state, value: 50.0 }
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configuration");
        ui.add_space(8.0);
        ui.add(egui::Slider::new(&mut self.value, 0.0..=100.0).text("Value"));
        ui.add_space(8.0);
        if self.is_active() {
            ui.label(egui::RichText::new("This panel is active").color(Color32::from_rgb(0x9e, 0xce, 0x6a)));
        } else {
            ui.label(egui::RichText::new("Click this tab to activate").color(Color32::from_rgb(0x56, 0x5f, 0x89)));
        }
    }
}

impl Citizen for ConfigPanel {
    fn id(&self) -> &CitizenId { &self.citizen_id }
    fn state(&self) -> &CitizenState { &self.citizen_state }
    fn state_mut(&mut self) -> &mut CitizenState { &mut self.citizen_state }
}

struct DisplayPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
}

impl DisplayPanel {
    fn new(state: CitizenState) -> Self {
        Self { citizen_id: CitizenId::new("display"), citizen_state: state }
    }

    fn show(&self, ui: &mut egui::Ui) {
        ui.heading("Display");
        ui.add_space(8.0);
        if self.is_active() {
            ui.label(egui::RichText::new("This panel is active").color(Color32::from_rgb(0x9e, 0xce, 0x6a)));
        } else {
            ui.label(egui::RichText::new("Click this tab to activate").color(Color32::from_rgb(0x56, 0x5f, 0x89)));
        }
    }
}

impl Citizen for DisplayPanel {
    fn id(&self) -> &CitizenId { &self.citizen_id }
    fn state(&self) -> &CitizenState { &self.citizen_state }
    fn state_mut(&mut self) -> &mut CitizenState { &mut self.citizen_state }
}

// ── Tabs ────────────────────────────────────────────────────────────────

#[derive(Clone)]
enum TabKind { Config, Display, Logger }

#[derive(Clone)]
struct Tab { kind: TabKind }

impl Tab {
    fn title(&self) -> &str {
        match self.kind {
            TabKind::Config => "Config",
            TabKind::Display => "Display",
            TabKind::Logger => "Logger",
        }
    }

    fn citizen_id(&self) -> Option<CitizenId> {
        match self.kind {
            TabKind::Config => Some(CitizenId::new("config")),
            TabKind::Display => Some(CitizenId::new("display")),
            TabKind::Logger => None,
        }
    }
}

// ── TabViewer bridges egui_dock to the Dispatcher ───────────────────────

struct TabViewer<'a> {
    dispatcher: &'a mut Dispatcher,
    config: &'a mut ConfigPanel,
    display: &'a DisplayPanel,
    log: &'a mut Vec<String>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn on_tab_button(&mut self, tab: &mut Tab, response: &egui::Response) {
        if response.clicked() {
            if let Some(id) = tab.citizen_id() {
                self.dispatcher.activate(&id);
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
        match tab.kind {
            TabKind::Config => self.config.show(ui),
            TabKind::Display => self.display.show(ui),
            TabKind::Logger => {
                ui.heading("Messages");
                egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    for line in self.log.iter() {
                        let color = if line.contains("Activated") {
                            Color32::from_rgb(0x9e, 0xce, 0x6a)
                        } else {
                            Color32::from_rgb(0x56, 0x5f, 0x89)
                        };
                        ui.label(egui::RichText::new(line).color(color).monospace());
                    }
                });
            }
        }
    }
}

// ── App ─────────────────────────────────────────────────────────────────

struct MyApp {
    dock_state: DockState<Tab>,
    dispatcher: Dispatcher,
    config: ConfigPanel,
    display: DisplayPanel,
    log: Vec<String>,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut dispatcher = Dispatcher::new();
        let config_state = dispatcher.register(CitizenId::new("config"));
        let display_state = dispatcher.register(CitizenId::new("display"));
        dispatcher.activate(&CitizenId::new("config"));
        let _ = dispatcher.drain_messages();

        let config = ConfigPanel::new(config_state);
        let display = DisplayPanel::new(display_state);

        let mut dock_state = DockState::new(vec![Tab { kind: TabKind::Display }]);
        let [left, _right] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(), 0.35,
            vec![Tab { kind: TabKind::Config }],
        );
        dock_state.main_surface_mut().split_below(
            left, 0.65,
            vec![Tab { kind: TabKind::Logger }],
        );

        Self { dock_state, dispatcher, config, display, log: vec!["App started".to_string()] }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut dock_state = self.dock_state.clone();
        let mut dispatcher = std::mem::take(&mut self.dispatcher);
        {
            let mut viewer = TabViewer {
                dispatcher: &mut dispatcher,
                config: &mut self.config,
                display: &self.display,
                log: &mut self.log,
            };
            DockArea::new(&mut dock_state).show(ctx, &mut viewer);
        }

        for msg in dispatcher.drain_messages() {
            match &msg {
                CitizenMessage::Activated { id } => {
                    self.log.push(format!("[CITIZEN] Activated: {}", id));
                }
                CitizenMessage::Deactivated { id } => {
                    self.log.push(format!("[CITIZEN] Deactivated: {}", id));
                }
                _ => {}
            }
        }

        self.dispatcher = dispatcher;
        self.dock_state = dock_state;
    }
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Getting Started — egui-citizen",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([700.0, 450.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}
```
