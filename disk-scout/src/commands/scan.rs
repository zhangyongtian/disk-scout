use crate::{cli::ScanArgs, ignore::IgnoreConfig, plan::ScanPlan};

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

    print_plan(&plan);
    0
}

fn print_plan(plan: &ScanPlan) {
    println!("disk-scout scan plan:");
    println!("  root: {}", plan.root.display());
    println!("  top_files: {}", plan.top_files);
    println!("  top_dirs: {}", plan.top_dirs);
    println!("  min_size: {}", plan.min_size);
    println!("  format: {}", plan.format);
    println!("  ignore.patterns: {}", plan.ignore.patterns.len());
    println!(
        "  ignore.ignore_file: {}",
        plan.ignore
            .ignore_file
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<none>".to_string())
    );
}
