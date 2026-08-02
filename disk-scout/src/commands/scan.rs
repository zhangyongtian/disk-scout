use crate::{
    cli::ScanArgs,
    ignore::IgnoreConfig,
    output::{format_bytes, OutputFormat},
    plan::ScanPlan,
    scanner,
};

pub fn run(args: ScanArgs) -> i32 {
    let plan = ScanPlan {
        root: args.path,
        top_files: args.top_files,
        top_dirs: args.top_dirs,
        min_size: args.min_size,
        format: args.format,
        ignore: IgnoreConfig {
            patterns: args.ignore,
            ignore_file: args.ignore_file,
        },
    };

    match scanner::scan(&plan) {
        Ok(result) => {
            print_result(&plan, &result);
            0
        }
        Err(e) => {
            eprintln!("scan failed: {}: {}", e.root.display(), e.message);
            2
        }
    }
}

fn print_result(plan: &ScanPlan, result: &scanner::ScanResult) {
    match plan.format {
        OutputFormat::Text => print_result_text(plan, result),
        OutputFormat::Json => print_result_json(plan, result),
    }
}

fn print_result_text(plan: &ScanPlan, result: &scanner::ScanResult) {
    print!("{}", render_result_text(plan, result));
}

fn print_result_json(plan: &ScanPlan, result: &scanner::ScanResult) {
    print!("{}", render_result_json(plan, result));
}

fn render_result_text(plan: &ScanPlan, result: &scanner::ScanResult) -> String {
    use std::fmt::Write;

    let mut out = String::new();

    writeln!(&mut out, "root: {}", plan.root.display()).ok();
    writeln!(
        &mut out,
        "bytes_total: {} ({})",
        format_bytes(result.stats.bytes_total),
        result.stats.bytes_total
    )
    .ok();
    writeln!(&mut out, "files_seen: {}", result.stats.files_seen).ok();
    writeln!(&mut out, "dirs_seen: {}", result.stats.dirs_seen).ok();
    writeln!(&mut out, "errors_total: {}", result.stats.errors_total).ok();
    writeln!(&mut out, "duration_ms: {}", result.stats.duration.as_millis()).ok();

    writeln!(&mut out).ok();
    writeln!(&mut out, "top_files:").ok();
    for item in &result.top_files {
        writeln!(
            &mut out,
            "  {} ({}) {}",
            format_bytes(item.size),
            item.size,
            item.path.display()
        )
        .ok();
    }

    writeln!(&mut out).ok();
    writeln!(&mut out, "top_dirs:").ok();
    for item in &result.top_dirs {
        writeln!(
            &mut out,
            "  {} ({}) {}",
            format_bytes(item.size),
            item.size,
            item.path.display()
        )
        .ok();
    }

    out
}

fn render_result_json(plan: &ScanPlan, result: &scanner::ScanResult) -> String {
    use std::fmt::Write;

    let mut out = String::new();

    writeln!(&mut out, "{{").ok();
    writeln!(&mut out, "  \"meta\": {{").ok();
    writeln!(
        &mut out,
        "    \"scan_root\": \"{}\",",
        escape_json_string(&plan.root.display().to_string())
    )
    .ok();
    writeln!(&mut out, "    \"stats\": {{").ok();
    writeln!(&mut out, "      \"bytes_total\": {},", result.stats.bytes_total).ok();
    writeln!(&mut out, "      \"files_seen\": {},", result.stats.files_seen).ok();
    writeln!(&mut out, "      \"dirs_seen\": {},", result.stats.dirs_seen).ok();
    writeln!(&mut out, "      \"errors_total\": {},", result.stats.errors_total).ok();
    writeln!(
        &mut out,
        "      \"duration_ms\": {}",
        result.stats.duration.as_millis()
    )
    .ok();
    writeln!(&mut out, "    }}").ok();
    writeln!(&mut out, "  }},").ok();
    writeln!(&mut out, "  \"top_files\": [").ok();
    for (idx, item) in result.top_files.iter().enumerate() {
        let comma = if idx + 1 == result.top_files.len() { "" } else { "," };
        writeln!(
            &mut out,
            "    {{ \"size_bytes\": {}, \"path\": \"{}\" }}{}",
            item.size,
            escape_json_string(&item.path.display().to_string()),
            comma
        )
        .ok();
    }
    writeln!(&mut out, "  ],").ok();
    writeln!(&mut out, "  \"top_dirs\": [").ok();
    for (idx, item) in result.top_dirs.iter().enumerate() {
        let comma = if idx + 1 == result.top_dirs.len() { "" } else { "," };
        writeln!(
            &mut out,
            "    {{ \"size_bytes\": {}, \"path\": \"{}\" }}{}",
            item.size,
            escape_json_string(&item.path.display().to_string()),
            comma
        )
        .ok();
    }
    writeln!(&mut out, "  ],").ok();
    writeln!(&mut out, "  \"errors\": {{").ok();
    writeln!(&mut out, "    \"total\": {},", result.errors.total).ok();
    writeln!(&mut out, "    \"samples\": [").ok();
    for (idx, sample) in result.errors.samples.iter().enumerate() {
        let comma = if idx + 1 == result.errors.samples.len() {
            ""
        } else {
            ","
        };
        writeln!(
            &mut out,
            "      {{ \"path\": \"{}\", \"message\": \"{}\" }}{}",
            escape_json_string(&sample.path.display().to_string()),
            escape_json_string(&sample.message),
            comma
        )
        .ok();
    }
    writeln!(&mut out, "    ]").ok();
    writeln!(&mut out, "  }}").ok();
    writeln!(&mut out, "}}").ok();

    out
}

fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                use std::fmt::Write;
                write!(&mut out, "\\u{:04x}", c as u32).ok();
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_text_report() {
        let plan = ScanPlan {
            root: "/tmp/root".into(),
            top_files: 2,
            top_dirs: 2,
            min_size: 0,
            format: OutputFormat::Text,
            ignore: IgnoreConfig::default(),
        };

        let result = scanner::ScanResult {
            root: plan.root.clone(),
            stats: scanner::ScanStats {
                files_seen: 3,
                dirs_seen: 2,
                bytes_total: 2048,
                errors_total: 1,
                duration: std::time::Duration::from_millis(12),
            },
            top_files: vec![scanner::SizedPath {
                path: "/tmp/root/a.bin".into(),
                size: 2048,
            }],
            top_dirs: vec![scanner::SizedPath {
                path: "/tmp/root".into(),
                size: 2048,
            }],
            errors: scanner::ScanErrors {
                total: 1,
                samples: vec![scanner::ScanErrorSample {
                    path: "/tmp/root/x".into(),
                    message: "denied".to_string(),
                }],
            },
        };

        let out = render_result_text(&plan, &result);
        assert!(out.contains("root: "));
        assert!(out.contains("bytes_total: "));
        assert!(out.contains("KiB"));
        assert!(out.contains("top_files:"));
        assert!(out.contains("top_dirs:"));
    }

    #[test]
    fn renders_json_report() {
        let plan = ScanPlan {
            root: "/tmp/root".into(),
            top_files: 2,
            top_dirs: 2,
            min_size: 0,
            format: OutputFormat::Json,
            ignore: IgnoreConfig::default(),
        };

        let result = scanner::ScanResult {
            root: plan.root.clone(),
            stats: scanner::ScanStats {
                files_seen: 1,
                dirs_seen: 1,
                bytes_total: 1,
                errors_total: 0,
                duration: std::time::Duration::from_millis(1),
            },
            top_files: vec![],
            top_dirs: vec![],
            errors: scanner::ScanErrors::default(),
        };

        let out = render_result_json(&plan, &result);
        assert!(out.contains("\"meta\""));
        assert!(out.contains("\"scan_root\""));
        assert!(out.contains("\"stats\""));
        assert!(out.contains("\"top_files\""));
        assert!(out.contains("\"top_dirs\""));
        assert!(out.contains("\"errors\""));
    }
}
