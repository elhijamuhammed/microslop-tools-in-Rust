use eframe::egui;

struct App {
    selected_tool: Tool,
}

#[derive(PartialEq)]
enum Tool {
    Processes,
    Services,
    Logs,
}

impl Default for App {
    fn default() -> Self {
        Self {
            selected_tool: Tool::Processes,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Left sidebar (tool selector)
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

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.selected_tool {
                Tool::Processes => {
                    ui.heading("Processes");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.button("Refresh");
                        ui.button("Kill");
                        ui.button("Details");
                    });

                    ui.label("Process list will go here.");
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
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Microslop Tools (Rust)")
            .with_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Microslop Tools",
        options,
        Box::new(|_| Ok(Box::<App>::default())),
    )
}
