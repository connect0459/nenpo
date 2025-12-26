mod application;
mod domain;
mod infrastructure;
mod presentation;

use clap::Parser;
use presentation::cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            config,
            year,
            department,
            format,
        } => {
            println!("Generating annual report...");
            println!("  Config: {}", config);
            if let Some(y) = year {
                println!("  Year: {}", y);
            }
            if let Some(d) = department {
                println!("  Department: {}", d);
            }
            if let Some(f) = format {
                println!("  Format: {}", f);
            }

            // Phase 1 MVP: Basic structure only
            // TODO: Implement actual report generation logic
            println!("\nPhase 1 MVP: Report generation structure is ready");
            println!("Actual GitHub data fetching and report generation will be implemented in future iterations");
        }
    }
}
