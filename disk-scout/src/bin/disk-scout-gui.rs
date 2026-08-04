#[cfg(not(windows))]
fn main() {
    eprintln!("disk-scout-gui is only supported on Windows");
    std::process::exit(2);
}

#[cfg(windows)]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "disk-scout",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

#[cfg(windows)]
struct App {
    root_path: String,
    top_files: usize,
    top_dirs: usize,
    min_size: String,
    ignore_patterns: String,
    ignore_file: String,
    status: String,
}

#[cfg(windows)]
impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            root_path: String::new(),
            top_files: 20,
            top_dirs: 20,
            min_size: "0".to_string(),
            ignore_patterns: String::new(),
            ignore_file: String::new(),
            status: "idle".to_string(),
        }
    }

    fn choose_root_path(&mut self) {
        let Some(path) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        self.root_path = path.display().to_string();
    }

    fn choose_ignore_file(&mut self) {
        let Some(path) = rfd::FileDialog::new().pick_file() else {
            return;
        };
        self.ignore_file = path.display().to_string();
    }
}

#[cfg(windows)]
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("disk-scout");
            ui.add_space(8.0);

            ui.label("Root path");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.root_path);
                if ui.button("Choose...").clicked() {
                    self.choose_root_path();
                }
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("Top files");
                ui.add(egui::DragValue::new(&mut self.top_files).clamp_range(0..=10000));
                ui.label("Top dirs");
                ui.add(egui::DragValue::new(&mut self.top_dirs).clamp_range(0..=10000));
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("Min size");
                ui.text_edit_singleline(&mut self.min_size);
            });

            ui.add_space(8.0);
            ui.label("Ignore patterns (one per line)");
            ui.add(
                egui::TextEdit::multiline(&mut self.ignore_patterns)
                    .desired_rows(4)
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(8.0);
            ui.label("Ignore file");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.ignore_file);
                if ui.button("Choose...").clicked() {
                    self.choose_ignore_file();
                }
            });

            ui.add_space(12.0);
            if ui.button("Start scan").clicked() {
                self.status = "scan not implemented yet".to_string();
            }

            ui.add_space(12.0);
            ui.label(format!("Status: {}", self.status));
        });
    }
}
