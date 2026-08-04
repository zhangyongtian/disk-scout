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
enum ScanStatus {
    Idle,
    Scanning,
    Done,
    Failed(String),
}

#[cfg(windows)]
struct ScanOutput {
    result: disk_scout::scanner::ScanResult,
}

#[cfg(windows)]
struct ScanConfig {
    root_path: std::path::PathBuf,
    top_files: usize,
    top_dirs: usize,
    min_size: u64,
    ignore_patterns: Vec<String>,
    ignore_file: Option<std::path::PathBuf>,
}

#[cfg(windows)]
struct App {
    root_path: String,
    top_files: usize,
    top_dirs: usize,
    min_size: String,
    ignore_patterns: String,
    ignore_file: String,
    status: ScanStatus,
    receiver: Option<std::sync::mpsc::Receiver<Result<ScanOutput, String>>>,
    result: Option<disk_scout::scanner::ScanResult>,
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
            status: ScanStatus::Idle,
            receiver: None,
            result: None,
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

    fn build_scan_config(&self) -> Result<ScanConfig, String> {
        let root_path = self.root_path.trim();
        if root_path.is_empty() {
            return Err("root path is empty".to_string());
        }

        let root_path = std::path::PathBuf::from(root_path);

        let ignore_file = {
            let p = self.ignore_file.trim();
            if p.is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(p))
            }
        };

        let ignore_patterns = self
            .ignore_patterns
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect::<Vec<_>>();

        let min_size = disk_scout::size::parse_size_bytes(&self.min_size)?;

        Ok(ScanConfig {
            root_path,
            top_files: self.top_files,
            top_dirs: self.top_dirs,
            min_size,
            ignore_patterns,
            ignore_file,
        })
    }

    fn start_scan(&mut self) {
        let config = match self.build_scan_config() {
            Ok(c) => c,
            Err(e) => {
                self.status = ScanStatus::Failed(e);
                return;
            }
        };

        let (tx, rx) = std::sync::mpsc::channel();
        self.receiver = Some(rx);
        self.status = ScanStatus::Scanning;
        self.result = None;

        std::thread::spawn(move || {
            let plan = disk_scout::plan::ScanPlan {
                root: config.root_path,
                top_files: config.top_files,
                top_dirs: config.top_dirs,
                min_size: config.min_size,
                ignore: disk_scout::ignore::IgnoreConfig {
                    patterns: config.ignore_patterns,
                    ignore_file: config.ignore_file,
                },
            };

            let r = disk_scout::scanner::scan(&plan)
                .map(|result| ScanOutput { result })
                .map_err(|e| format!("scan failed: {}: {}", e.root.display(), e.message));

            let _ = tx.send(r);
        });
    }

    fn poll_scan_result(&mut self) {
        let Some(rx) = &self.receiver else {
            return;
        };

        match rx.try_recv() {
            Ok(Ok(out)) => {
                self.result = Some(out.result);
                self.receiver = None;
                self.status = ScanStatus::Done;
            }
            Ok(Err(e)) => {
                self.receiver = None;
                self.status = ScanStatus::Failed(e);
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.receiver = None;
                self.status = ScanStatus::Failed("scan worker disconnected".to_string());
            }
        }
    }
}

#[cfg(windows)]
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_scan_result();

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
            let scanning = matches!(self.status, ScanStatus::Scanning);
            if ui
                .add_enabled(!scanning, egui::Button::new("Start scan"))
                .clicked()
            {
                self.start_scan();
            }

            ui.add_space(12.0);
            let status_text = match &self.status {
                ScanStatus::Idle => "idle".to_string(),
                ScanStatus::Scanning => "scanning...".to_string(),
                ScanStatus::Done => "done".to_string(),
                ScanStatus::Failed(e) => format!("failed: {e}"),
            };
            ui.label(format!("Status: {status_text}"));

            if let Some(result) = &self.result {
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(format!("Root: {}", result.root.display()));
                });
                ui.label(format!(
                    "Total: {} ({})",
                    disk_scout::output::format_bytes(result.stats.bytes_total),
                    result.stats.bytes_total
                ));
                ui.label(format!(
                    "Files: {}, Dirs: {}, Errors: {}, Duration(ms): {}",
                    result.stats.files_seen,
                    result.stats.dirs_seen,
                    result.stats.errors_total,
                    result.stats.duration.as_millis()
                ));

                ui.add_space(12.0);
                ui.heading("Top files");
                for item in &result.top_files {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{} ({})",
                            disk_scout::output::format_bytes(item.size),
                            item.size
                        ));
                        ui.label(item.path.display().to_string());
                        if ui.button("Copy path").clicked() {
                            ui.output_mut(|o| o.copied_text = item.path.display().to_string());
                        }
                    });
                }

                ui.add_space(12.0);
                ui.heading("Top dirs");
                for item in &result.top_dirs {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{} ({})",
                            disk_scout::output::format_bytes(item.size),
                            item.size
                        ));
                        ui.label(item.path.display().to_string());
                    });
                }
            }
        });
    }
}
