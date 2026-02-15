use sysinfo::{Networks, System};
use std::{collections::HashMap, time::{Instant, Duration}};
use crate::formats;

#[derive(Clone)]
pub struct ProcRow {
    pub pid: i32,
    pub name: String,
    pub cpu: f32,
    pub memory_bytes: u64,
    pub read_bps: f64,
    pub write_bps: f64,
}

pub struct App {
    pub selected_tool: Tool,
    pub processes: Vec<ProcRow>,
    pub error: Option<String>,

    pub sys: System,
    pub nets: Networks,

    pub prev_proc_disk: HashMap<i32, (u64, u64)>,
    pub prev_net: (u64, u64),

    pub net_rx_bps: f64,
    pub net_tx_bps: f64,

    pub last_refresh: Instant,
    pub auto_refresh: bool,
    pub refresh_ms: u64,
}

#[derive(PartialEq)]
pub enum Tool {
    Processes,
    Services,
    Logs,
}

impl Default for App {
    fn default() -> Self {
        crate::collectors::init_app()
    }
}

// Real methods (so main.rs can call self.refresh_everything())
impl App {
    pub fn refresh_everything(&mut self) {
        crate::collectors::refresh_everything(self);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .min_width(150.0)
            .show(ctx, |ui| {
                ui.heading("Microslop Tools");
                ui.separator();

                if ui
                    .selectable_label(self.selected_tool == Tool::Processes, "Processes")
                    .clicked()
                {
                    self.selected_tool = Tool::Processes;
                }

                if ui
                    .selectable_label(self.selected_tool == Tool::Services, "Services")
                    .clicked()
                {
                    self.selected_tool = Tool::Services;
                }

                if ui
                    .selectable_label(self.selected_tool == Tool::Logs, "Logs")
                    .clicked()
                {
                    self.selected_tool = Tool::Logs;
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| match self.selected_tool {
            Tool::Processes => {
                ui.heading("Processes");
                ui.separator();

                if self.auto_refresh
                    && self.last_refresh.elapsed() > Duration::from_millis(self.refresh_ms)
                {
                    self.refresh_everything();
                    ctx.request_repaint();
                }

                ui.horizontal(|ui| {
                    if ui.button("Refresh").clicked() {
                        self.refresh_everything();
                    }

                    ui.checkbox(&mut self.auto_refresh, "Auto");
                    ui.add(
                        egui::DragValue::new(&mut self.refresh_ms)
                            .range(250..=5000)
                            .suffix(" ms"),
                    );

                    ui.separator();
                    ui.label(format!("Count: {}", self.processes.len()));

                    ui.separator();
                    ui.label(format!(
                        "Net: ↓ {:.2} MB/s  ↑ {:.2} MB/s",
                        formats::bps_to_mbps(self.net_rx_bps),
                        formats::bps_to_mbps(self.net_tx_bps),
                    ));

                    ui.separator();
                    ui.add_enabled(false, egui::Button::new("Kill"));
                    ui.add_enabled(false, egui::Button::new("Details"));
                });

                if let Some(err) = &self.error {
                    ui.colored_label(egui::Color32::RED, err);
                }

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("proc_grid")
                        .striped(true)
                        .min_col_width(80.0)
                        .show(ui, |ui| {
                            ui.strong("PID");
                            ui.strong("Name");
                            ui.strong("CPU");
                            ui.strong("RAM");
                            ui.strong("Disk R");
                            ui.strong("Disk W");
                            ui.end_row();

                            for p in &self.processes {
                                ui.label(p.pid.to_string());
                                ui.label(&p.name);
                                ui.label(format!("{:.1} %", p.cpu));
                                ui.label(format!("{:.1} MB", formats::bytes_to_mb(p.memory_bytes)));
                                ui.label(format!("{:.2} MB/s", formats::bps_to_mbps(p.read_bps)));
                                ui.label(format!("{:.2} MB/s", formats::bps_to_mbps(p.write_bps)));
                                ui.end_row();
                            }
                        });
                });
            }
            Tool::Services => {
                ui.heading("Services");
                ui.separator();
                ui.label("Services view (not implemented).");
            }
            Tool::Logs => {
                ui.heading("Logs");
                ui.separator();
                ui.label("Logs view (not implemented).");
            }
        });
    }
}
