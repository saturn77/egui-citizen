//! Citizen Fetch — demonstrates backend threading with egui_citizen.
//!
//! A Fetch panel (citizen) sends a URL to a background thread.
//! The thread performs an HTTP GET and sends the response back.
//! The Response panel (citizen) displays the result.
//!
//! This shows the full citizen lifecycle:
//!   UI click → dispatcher.activate() → drain_messages()
//!   → forward to backend thread via channel
//!   → thread does work → sends result back via channel
//!   → UI reads result and displays it
//!
//! Run: cargo run -p citizen_fetch

use eframe::egui;
use egui::Color32;
use egui_dock::{DockArea, DockState, NodeIndex};
use egui_citizen::{CitizenMessage, Dispatcher};
use egui_citizen::message::CitizenId;
use crossbeam_channel::{unbounded, Receiver, Sender};

// ── Colors (Tokyo Night subset) ─────────────────────────────────────────

const BG: Color32 = Color32::from_rgb(0x24, 0x28, 0x3b);
const FG: Color32 = Color32::from_rgb(0xc0, 0xca, 0xf5);
const CYAN: Color32 = Color32::from_rgb(0x7d, 0xcf, 0xff);
const GREEN: Color32 = Color32::from_rgb(0x9e, 0xce, 0x6a);
const RED: Color32 = Color32::from_rgb(0xf7, 0x76, 0x8e);
const COMMENT: Color32 = Color32::from_rgb(0x56, 0x5f, 0x89);

// ── Backend messages ────────────────────────────────────────────────────

/// Request sent from UI to backend thread.
enum FetchRequest {
    Get(String),
}

/// Response sent from backend thread back to UI.
enum FetchResponse {
    Success { url: String, body: String, status: u16 },
    Error { url: String, error: String },
}

// ── Tabs ────────────────────────────────────────────────────────────────

#[derive(Clone)]
enum TabKind { Fetch, Response, Logger }

#[derive(Clone)]
struct Tab { kind: TabKind }

impl Tab {
    fn title(&self) -> &str {
        match self.kind {
            TabKind::Fetch => "Fetch",
            TabKind::Response => "Response",
            TabKind::Logger => "Logger",
        }
    }

    fn citizen_id(&self) -> Option<CitizenId> {
        match self.kind {
            TabKind::Fetch => Some(CitizenId::new("fetch")),
            TabKind::Response => Some(CitizenId::new("response")),
            TabKind::Logger => None,
        }
    }
}

// ── Tab viewer ──────────────────────────────────────────────────────────

struct TabViewer<'a> {
    dispatcher: &'a mut Dispatcher,
    url: &'a mut String,
    request_tx: &'a Sender<FetchRequest>,
    response_body: &'a str,
    response_status: &'a str,
    is_fetching: &'a mut bool,
    log: &'a mut Vec<String>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText { tab.title().into() }

    fn on_tab_button(&mut self, tab: &mut Tab, response: &egui::Response) {
        if response.clicked() {
            if let Some(id) = tab.citizen_id() {
                self.dispatcher.activate(&id);
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
        match tab.kind {
            TabKind::Fetch => self.render_fetch(ui),
            TabKind::Response => self.render_response(ui),
            TabKind::Logger => self.render_logger(ui),
        }
    }
}

impl TabViewer<'_> {
    fn render_fetch(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new().fill(BG).inner_margin(12.0).show(ui, |ui| {
            ui.heading(egui::RichText::new("HTTP Fetch").color(CYAN));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.text_edit_singleline(self.url);
            });

            ui.add_space(8.0);

            let button_text = if *self.is_fetching { "Fetching..." } else { "Fetch" };
            if ui.add_enabled(!*self.is_fetching, egui::Button::new(button_text)).clicked() {
                let url = self.url.clone();
                self.log.push(format!("[FETCH] Requesting: {}", url));
                let _ = self.request_tx.send(FetchRequest::Get(url));
                *self.is_fetching = true;
                self.log.push("[INFO] Request sent to backend thread".to_string());
            }

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(
                    "Enter a URL and click Fetch. The request runs on a \
                     background thread — the UI stays responsive."
                ).color(COMMENT)
            );
        });
    }

    fn render_response(&self, ui: &mut egui::Ui) {
        egui::Frame::new().fill(BG).inner_margin(12.0).show(ui, |ui| {
            ui.heading(egui::RichText::new("Response").color(GREEN));
            ui.add_space(4.0);

            if !self.response_status.is_empty() {
                ui.label(egui::RichText::new(self.response_status).color(CYAN).monospace());
                ui.separator();
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.response_body.is_empty() {
                    ui.label(egui::RichText::new("No response yet.").color(COMMENT));
                } else {
                    ui.label(egui::RichText::new(self.response_body).color(FG).monospace());
                }
            });
        });
    }

    fn render_logger(&self, ui: &mut egui::Ui) {
        egui::Frame::new().fill(BG).inner_margin(8.0).show(ui, |ui| {
            ui.heading(egui::RichText::new("Messages").color(CYAN));
            ui.add_space(4.0);

            egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                for line in self.log.iter() {
                    let color = if line.contains("[CITIZEN]") {
                        GREEN
                    } else if line.contains("[ERROR]") {
                        RED
                    } else {
                        COMMENT
                    };
                    ui.label(egui::RichText::new(line).color(color).monospace());
                }
            });
        });
    }
}

// ── App ─────────────────────────────────────────────────────────────────

struct FetchApp {
    dock_state: DockState<Tab>,
    dispatcher: Dispatcher,
    url: String,
    response_body: String,
    response_status: String,
    is_fetching: bool,
    log: Vec<String>,

    // Channels: UI → backend thread → UI
    request_tx: Sender<FetchRequest>,
    response_rx: Receiver<FetchResponse>,
}

impl FetchApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Register citizens
        let mut dispatcher = Dispatcher::new();
        dispatcher.register(CitizenId::new("fetch"));
        dispatcher.register(CitizenId::new("response"));
        dispatcher.activate(&CitizenId::new("fetch"));
        let _ = dispatcher.drain_messages();

        // Dock layout:
        // ┌──────────┬───────────┐
        // │  Fetch   │ Response  │
        // ├──────────┴───────────┤
        // │       Logger         │
        // └──────────────────────┘
        let mut dock_state = DockState::new(vec![Tab { kind: TabKind::Response }]);
        let [left, _right] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(), 0.35,
            vec![Tab { kind: TabKind::Fetch }],
        );
        dock_state.main_surface_mut().split_below(
            left, 0.75,
            vec![Tab { kind: TabKind::Logger }],
        );

        // Backend thread channels
        let (request_tx, request_rx) = unbounded::<FetchRequest>();
        let (response_tx, response_rx) = unbounded::<FetchResponse>();

        // Spawn backend thread
        std::thread::spawn(move || {
            for req in request_rx {
                match req {
                    FetchRequest::Get(url) => {
                        match ureq::get(&url).call() {
                            Ok(resp) => {
                                let status = resp.status();
                                let body = resp.into_string()
                                    .unwrap_or_else(|e| format!("(read error: {})", e));
                                // Truncate large responses for display
                                let body = if body.len() > 4000 {
                                    format!("{}...\n\n[truncated, {} bytes total]",
                                        &body[..4000], body.len())
                                } else {
                                    body
                                };
                                let _ = response_tx.send(FetchResponse::Success {
                                    url, body, status,
                                });
                            }
                            Err(e) => {
                                let _ = response_tx.send(FetchResponse::Error {
                                    url, error: e.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        });

        Self {
            dock_state,
            dispatcher,
            url: "https://httpbin.org/get".to_string(),
            response_body: String::new(),
            response_status: String::new(),
            is_fetching: false,
            log: vec!["[INFO] Citizen Fetch example started".to_string()],
            request_tx,
            response_rx,
        }
    }
}

impl eframe::App for FetchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for responses from backend thread (non-blocking)
        while let Ok(response) = self.response_rx.try_recv() {
            self.is_fetching = false;
            match response {
                FetchResponse::Success { url, body, status } => {
                    self.log.push(format!("[FETCH] {} → {} ({} bytes)", url, status, body.len()));
                    self.response_status = format!("HTTP {} — {}", status, url);
                    self.response_body = body;
                }
                FetchResponse::Error { url, error } => {
                    self.log.push(format!("[ERROR] {} → {}", url, error));
                    self.response_status = format!("Error — {}", url);
                    self.response_body = error;
                }
            }
        }

        // Render dock area
        let mut dock_state = self.dock_state.clone();
        let mut dispatcher = std::mem::take(&mut self.dispatcher);
        {
            let mut viewer = TabViewer {
                dispatcher: &mut dispatcher,
                url: &mut self.url,
                request_tx: &self.request_tx,
                response_body: &self.response_body,
                response_status: &self.response_status,
                is_fetching: &mut self.is_fetching,
                log: &mut self.log,
            };
            DockArea::new(&mut dock_state).show(ctx, &mut viewer);
        }

        // Drain citizen messages
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

        // Keep repainting while fetching so we pick up the response
        if self.is_fetching {
            ctx.request_repaint();
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Citizen Fetch",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([800.0, 500.0])
                .with_min_inner_size([500.0, 300.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(FetchApp::new(cc)))),
    )
}
