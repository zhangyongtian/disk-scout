use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use disk_scout::size::parse_size_bytes;

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

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Text => write!(f, "text"),
            OutputFormat::Json => write!(f, "json"),
        }
    }
}

#[derive(Args, Clone)]
pub struct ScanArgs {
    pub path: PathBuf,

    #[arg(long = "top-files", default_value_t = 20, value_name = "N")]
    pub top_files: usize,

    #[arg(long = "top-dirs", default_value_t = 20, value_name = "N")]
    pub top_dirs: usize,

    #[arg(
        long = "min-size",
        default_value = "0",
        value_name = "SIZE",
        value_parser = parse_size_bytes
    )]
    pub min_size: u64,

    #[arg(long, default_value_t = OutputFormat::Text, value_name = "FORMAT")]
    pub format: OutputFormat,

    #[arg(long, value_name = "PATTERN")]
    pub ignore: Vec<String>,

    #[arg(long = "ignore-file", value_name = "PATH")]
    pub ignore_file: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use disk_scout::size::parse_size_bytes;

    #[test]
    fn parses_plain_bytes() {
        assert_eq!(parse_size_bytes("0").unwrap(), 0);
        assert_eq!(parse_size_bytes("42").unwrap(), 42);
        assert_eq!(parse_size_bytes("42 b").unwrap(), 42);
    }

    #[test]
    fn parses_binary_units() {
        assert_eq!(parse_size_bytes("1KiB").unwrap(), 1024);
        assert_eq!(parse_size_bytes("1 MiB").unwrap(), 1024 * 1024);
    }

    #[test]
    fn parses_decimal_units() {
        assert_eq!(parse_size_bytes("1KB").unwrap(), 1000);
        assert_eq!(parse_size_bytes("2 mb").unwrap(), 2_000_000);
    }
}
