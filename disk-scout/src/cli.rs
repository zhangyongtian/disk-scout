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

    #[arg(
        long = "min-size",
        default_value = "0",
        value_name = "SIZE",
        value_parser = parse_size
    )]
    pub min_size: u64,

    #[arg(long, default_value_t = OutputFormat::Text, value_name = "FORMAT")]
    pub format: OutputFormat,

    #[arg(long, value_name = "PATTERN")]
    pub ignore: Vec<String>,

    #[arg(long = "ignore-file", value_name = "PATH")]
    pub ignore_file: Option<PathBuf>,
}

fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("size is empty".to_string());
    }

    let mut split = s.split_whitespace();
    let first = split.next().unwrap_or("");
    let second = split.next();
    if split.next().is_some() {
        return Err("invalid size".to_string());
    }

    let (num_str, unit_str) = if let Some(unit) = second {
        (first, unit)
    } else {
        let mut idx = first.len();
        for (i, c) in first.char_indices() {
            if !(c.is_ascii_digit() || c == '.') {
                idx = i;
                break;
            }
        }

        let (n, u) = first.split_at(idx);
        (n, u)
    };

    let num: f64 = num_str
        .parse()
        .map_err(|_| format!("invalid number: {num_str}"))?;
    if !num.is_finite() || num < 0.0 {
        return Err("invalid number".to_string());
    }

    let unit = unit_str.trim().to_ascii_lowercase();
    let (base, exp) = match unit.as_str() {
        "" | "b" => (1024u64, 0u32),
        "k" | "kb" => (1000u64, 1u32),
        "m" | "mb" => (1000u64, 2u32),
        "g" | "gb" => (1000u64, 3u32),
        "t" | "tb" => (1000u64, 4u32),
        "kib" => (1024u64, 1u32),
        "mib" => (1024u64, 2u32),
        "gib" => (1024u64, 3u32),
        "tib" => (1024u64, 4u32),
        _ => return Err(format!("unknown unit: {unit_str}")),
    };

    let mul = base.saturating_pow(exp);
    let bytes = (num * mul as f64).round();
    if bytes > u64::MAX as f64 {
        return Err("size overflow".to_string());
    }

    Ok(bytes as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_bytes() {
        assert_eq!(parse_size("0").unwrap(), 0);
        assert_eq!(parse_size("42").unwrap(), 42);
        assert_eq!(parse_size("42 b").unwrap(), 42);
    }

    #[test]
    fn parses_binary_units() {
        assert_eq!(parse_size("1KiB").unwrap(), 1024);
        assert_eq!(parse_size("1 MiB").unwrap(), 1024 * 1024);
    }

    #[test]
    fn parses_decimal_units() {
        assert_eq!(parse_size("1KB").unwrap(), 1000);
        assert_eq!(parse_size("2 mb").unwrap(), 2_000_000);
    }
}
