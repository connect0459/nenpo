mod application;
mod domain;
mod infrastructure;
mod presentation;

use application::services::report_generator::ReportGenerator;
use clap::Parser;
use infrastructure::config::toml_config_repository::TomlConfigRepository;
use infrastructure::document::local_file_document_repository::LocalFileDocumentRepository;
use infrastructure::github::{GhCommandExecutor, GhCommandRepository};
use infrastructure::output::markdown_output_repository::MarkdownOutputRepository;
use presentation::cli::{Cli, Commands};
use std::path::Path;
use std::process;

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
            if let Some(d) = &department {
                println!("  Department: {}", d);
            }
            if let Some(f) = &format {
                println!("  Format: {}", f);
            }
            println!();

            // Create repository instances
            let config_repo = TomlConfigRepository::new();
            let github_repo = GhCommandRepository::new(GhCommandExecutor::new());
            let document_repo = LocalFileDocumentRepository::new();
            let output_repo = MarkdownOutputRepository::new();

            // Create report generator
            let generator =
                ReportGenerator::new(config_repo, github_repo, document_repo, output_repo);

            // Determine output directory
            let output_dir = Path::new("./reports");
            if !output_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(output_dir) {
                    eprintln!("Error: Failed to create output directory: {}", e);
                    process::exit(1);
                }
            }

            // Generate reports
            match generator.generate(Path::new(&config), year, department.as_deref(), output_dir) {
                Ok(files) => {
                    println!("✅ Successfully generated {} report(s):", files.len());
                    for file in files {
                        println!("   - {}/{}", output_dir.display(), file);
                    }
                }
                Err(e) => {
                    eprintln!("Error: Failed to generate report: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}
