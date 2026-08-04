mod cli;
mod commands;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Scan(args) => {
            let exit = commands::scan::run(args);
            std::process::exit(exit);
        }
    }
}
