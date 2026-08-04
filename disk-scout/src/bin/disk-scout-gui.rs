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
use eframe::egui;

#[cfg(windows)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum MinSizeUnit {
    B,
    K,
    M,
    G,
}

#[cfg(windows)]
impl MinSizeUnit {
    fn multiplier(self) -> u64 {
        match self {
            Self::B => 1,
            Self::K => 1024,
            Self::M => 1024 * 1024,
            Self::G => 1024 * 1024 * 1024,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::B => "B",
            Self::K => "K",
            Self::M => "M",
            Self::G => "G",
        }
    }
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
struct FileEntry {
    path: std::path::PathBuf,
    size: u64,
    deleted: bool,
}

#[cfg(windows)]
struct DisplayResult {
    root: std::path::PathBuf,
    stats: disk_scout::scanner::ScanStats,
    top_files: Vec<FileEntry>,
    top_dirs: Vec<disk_scout::scanner::SizedPath>,
    errors: disk_scout::scanner::ScanErrors,
}

#[cfg(windows)]
struct App {
    root_path: String,
    top_files: usize,
    top_dirs: usize,
    min_size_value: String,
    min_size_unit: MinSizeUnit,
    ignore_patterns: String,
    ignore_file: String,
    status: ScanStatus,
    receiver: Option<std::sync::mpsc::Receiver<Result<ScanOutput, String>>>,
    result: Option<DisplayResult>,
    message: String,
    confirm_delete: Option<std::path::PathBuf>,
    confirm_delete_open: bool,
}

#[cfg(windows)]
impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            root_path: String::new(),
            top_files: 20,
            top_dirs: 20,
            min_size_value: "0".to_string(),
            min_size_unit: MinSizeUnit::B,
            ignore_patterns: String::new(),
            ignore_file: String::new(),
            status: ScanStatus::Idle,
            receiver: None,
            result: None,
            message: String::new(),
            confirm_delete: None,
            confirm_delete_open: false,
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

        let min_size_value = self.min_size_value.trim();
        if min_size_value.is_empty() {
            return Err("min size is empty".to_string());
        }
        let value = min_size_value
            .parse::<u64>()
            .map_err(|_| "min size is not a valid number".to_string())?;
        let min_size = value
            .checked_mul(self.min_size_unit.multiplier())
            .ok_or_else(|| "min size is too large".to_string())?;

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
        self.message = String::new();
        self.confirm_delete = None;
        self.confirm_delete_open = false;

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
                let top_files = out
                    .result
                    .top_files
                    .iter()
                    .map(|x| FileEntry {
                        path: x.path.clone(),
                        size: x.size,
                        deleted: false,
                    })
                    .collect::<Vec<_>>();
                self.result = Some(DisplayResult {
                    root: out.result.root,
                    stats: out.result.stats,
                    top_files,
                    top_dirs: out.result.top_dirs,
                    errors: out.result.errors,
                });
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

    fn request_delete(&mut self, path: std::path::PathBuf) {
        self.confirm_delete = Some(path);
        self.confirm_delete_open = true;
    }

    fn perform_delete(&mut self, path: &std::path::PathBuf) -> Result<(), String> {
        let Some(result) = &self.result else {
            return Err("no scan result".to_string());
        };

        let root = result
            .root
            .canonicalize()
            .map_err(|e| format!("failed to canonicalize root: {e}"))?;
        let p = path
            .canonicalize()
            .map_err(|e| format!("failed to canonicalize target: {e}"))?;

        if !p.starts_with(&root) {
            return Err("refuse to delete file outside scan root".to_string());
        }

        let meta = std::fs::metadata(&p).map_err(|e| format!("failed to stat file: {e}"))?;
        if !meta.is_file() {
            return Err("refuse to delete non-regular file".to_string());
        }

        trash::delete(&p).map_err(|e| format!("failed to move to recycle bin: {e}"))?;
        Ok(())
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
                ui.add(egui::DragValue::new(&mut self.top_files).range(0..=10000));
                ui.label("Top dirs");
                ui.add(egui::DragValue::new(&mut self.top_dirs).range(0..=10000));
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("Min size");
                ui.text_edit_singleline(&mut self.min_size_value);
                egui::ComboBox::from_id_salt("min_size_unit")
                    .selected_text(self.min_size_unit.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.min_size_unit, MinSizeUnit::B, "B");
                        ui.selectable_value(&mut self.min_size_unit, MinSizeUnit::K, "K");
                        ui.selectable_value(&mut self.min_size_unit, MinSizeUnit::M, "M");
                        ui.selectable_value(&mut self.min_size_unit, MinSizeUnit::G, "G");
                    });
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
            if !self.message.is_empty() {
                ui.label(&self.message);
            }

            let mut delete_request: Option<std::path::PathBuf> = None;

            if let Some(result) = &mut self.result {
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
                let remaining = ui.available_height();
                let list_height = ((remaining - 24.0) / 2.0).max(160.0);

                ui.group(|ui| {
                    ui.heading("Top files");
                    egui::ScrollArea::vertical()
                        .id_source("top_files_scroll")
                        .max_height(list_height)
                        .show(ui, |ui| {
                            for item in &mut result.top_files {
                                ui.horizontal(|ui| {
                                    ui.label(format!(
                                        "{} ({})",
                                        disk_scout::output::format_bytes(item.size),
                                        item.size
                                    ));

                                    let path_text = item.path.display().to_string();
                                    if item.deleted {
                                        ui.label(egui::RichText::new(&path_text).strikethrough());
                                    } else {
                                        ui.label(path_text.clone());
                                    }

                                    if ui.button("Copy path").clicked() {
                                        ui.output_mut(|o| o.copied_text = path_text.clone());
                                    }

                                    if ui
                                        .add_enabled(!item.deleted, egui::Button::new("Delete"))
                                        .clicked()
                                    {
                                        delete_request = Some(item.path.clone());
                                    }
                                });
                            }
                        });
                });

                ui.add_space(12.0);
                ui.group(|ui| {
                    ui.heading("Top dirs");
                    egui::ScrollArea::vertical()
                        .id_source("top_dirs_scroll")
                        .max_height(list_height)
                        .show(ui, |ui| {
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
                        });
                });

            }

            if let Some(path) = delete_request {
                self.request_delete(path);
            }

            if self.confirm_delete_open {
                let mut open = self.confirm_delete_open;
                let mut action: Option<bool> = None;

                egui::Window::new("Confirm delete")
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut open)
                    .show(ctx, |ui| {
                        let target = self
                            .confirm_delete
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| "-".to_string());

                        ui.label("Move file to recycle bin?");
                        ui.label(target);

                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                action = Some(false);
                            }

                            if ui.button("Delete").clicked() {
                                action = Some(true);
                            }
                        });
                    });

                if let Some(do_delete) = action {
                    if do_delete {
                        if let Some(p) = self.confirm_delete.clone() {
                            match self.perform_delete(&p) {
                                Ok(()) => {
                                    if let Some(r) = &mut self.result {
                                        for f in &mut r.top_files {
                                            if f.path == p {
                                                f.deleted = true;
                                            }
                                        }
                                    }
                                    self.message =
                                        format!("Deleted (to recycle bin): {}", p.display());
                                }
                                Err(e) => {
                                    self.message = format!("Delete failed: {e}");
                                }
                            }
                        }
                    }

                    open = false;
                }

                self.confirm_delete_open = open;
                if !open {
                    self.confirm_delete = None;
                }
            }
        });
    }
}
