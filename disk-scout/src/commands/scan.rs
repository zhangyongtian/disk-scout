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
    println!("root: {}", plan.root.display());
    println!(
        "bytes_total: {} ({})",
        format_bytes(result.stats.bytes_total),
        result.stats.bytes_total
    );
    println!("files_seen: {}", result.stats.files_seen);
    println!("dirs_seen: {}", result.stats.dirs_seen);
    println!("errors_total: {}", result.stats.errors_total);
    println!("duration_ms: {}", result.stats.duration.as_millis());

    println!();
    println!("top_files:");
    for item in &result.top_files {
        println!(
            "  {} ({}) {}",
            format_bytes(item.size),
            item.size,
            item.path.display()
        );
    }

    println!();
    println!("top_dirs:");
    for item in &result.top_dirs {
        println!(
            "  {} ({}) {}",
            format_bytes(item.size),
            item.size,
            item.path.display()
        );
    }
}

fn print_result_json(plan: &ScanPlan, result: &scanner::ScanResult) {
    println!("{{");
    println!("  \"meta\": {{");
    println!(
        "    \"scan_root\": \"{}\",",
        escape_json_string(&plan.root.display().to_string())
    );
    println!("    \"stats\": {{");
    println!("      \"bytes_total\": {},", result.stats.bytes_total);
    println!("      \"files_seen\": {},", result.stats.files_seen);
    println!("      \"dirs_seen\": {},", result.stats.dirs_seen);
    println!("      \"errors_total\": {},", result.stats.errors_total);
    println!("      \"duration_ms\": {}", result.stats.duration.as_millis());
    println!("    }}");
    println!("  }},");
    println!("  \"top_files\": [");
    for (idx, item) in result.top_files.iter().enumerate() {
        let comma = if idx + 1 == result.top_files.len() { "" } else { "," };
        println!(
            "    {{ \"size_bytes\": {}, \"path\": \"{}\" }}{}",
            item.size,
            escape_json_string(&item.path.display().to_string()),
            comma
        );
    }
    println!("  ],");
    println!("  \"top_dirs\": [");
    for (idx, item) in result.top_dirs.iter().enumerate() {
        let comma = if idx + 1 == result.top_dirs.len() { "" } else { "," };
        println!(
            "    {{ \"size_bytes\": {}, \"path\": \"{}\" }}{}",
            item.size,
            escape_json_string(&item.path.display().to_string()),
            comma
        );
    }
    println!("  ],");
    println!("  \"errors\": {{");
    println!("    \"total\": {},", result.errors.total);
    println!("    \"samples\": [");
    for (idx, sample) in result.errors.samples.iter().enumerate() {
        let comma = if idx + 1 == result.errors.samples.len() {
            ""
        } else {
            ","
        };
        println!(
            "      {{ \"path\": \"{}\", \"message\": \"{}\" }}{}",
            escape_json_string(&sample.path.display().to_string()),
            escape_json_string(&sample.message),
            comma
        );
    }
    println!("    ]");
    println!("  }}");
    println!("}}");
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
