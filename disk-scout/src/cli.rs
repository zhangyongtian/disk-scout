use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::output::OutputFormat;

#[derive(Parser)]
#[command(name = "disk-scout")]
#[command(about = "Scan disks and report top files/dirs by size", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Scan(ScanArgs),
}

#[derive(Args, Clone)]
pub struct ScanArgs {
    pub path: PathBuf,

    #[arg(long = "top-files", default_value_t = 20, value_name = "N")]
    pub top_files: usize,

    #[arg(long = "top-dirs", default_value_t = 20, value_name = "N")]
    pub top_dirs: usize,

    #[arg(long = "min-size", default_value_t = 0, value_name = "BYTES")]
    pub min_size: u64,

    #[arg(long, default_value_t = OutputFormat::Text, value_name = "FORMAT")]
    pub format: OutputFormat,

    #[arg(long, value_name = "PATTERN")]
    pub ignore: Vec<String>,

    #[arg(long = "ignore-file", value_name = "PATH")]
    pub ignore_file: Option<PathBuf>,
}
