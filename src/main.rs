mod application;
mod domain;
mod infrastructure;
mod presentation;

use application::services::report_generator::ReportGenerator;
use clap::Parser;
use domain::services::progress_reporter::StdoutProgressReporter;
use domain::value_objects::output_format::OutputFormat;
use infrastructure::cache::FileCache;
use infrastructure::config::toml_config_repository::TomlConfigRepository;
use infrastructure::document::local_file_document_repository::LocalFileDocumentRepository;
use infrastructure::github::{GhCommandExecutor, GhCommandRepository};
use infrastructure::output::html_output_repository::HtmlOutputRepository;
use infrastructure::output::json_output_repository::JsonOutputRepository;
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

            // Parse output format
            let output_format = format
                .as_deref()
                .and_then(|f| OutputFormat::from_str(f).ok())
                .unwrap_or(OutputFormat::Markdown);

            // Determine output directory
            let output_dir = Path::new("./reports");
            if !output_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(output_dir) {
                    eprintln!("Error: Failed to create output directory: {}", e);
                    process::exit(1);
                }
            }

            // Create shared repository instances
            let config_repo = TomlConfigRepository::new();
            let cache = FileCache::new().unwrap_or_else(|e| {
                eprintln!("Warning: Failed to create cache: {}. Proceeding without cache.", e);
                std::process::exit(1);
            });
            let github_repo = GhCommandRepository::new(
                GhCommandExecutor::new(),
                StdoutProgressReporter::new(),
                cache,
            );
            let document_repo = LocalFileDocumentRepository::new();

            // Generate reports based on format
            let result = match output_format {
                OutputFormat::Markdown => {
                    let output_repo = MarkdownOutputRepository::new();
                    let generator =
                        ReportGenerator::new(config_repo, github_repo, document_repo, output_repo);
                    generator.generate(
                        Path::new(&config),
                        year,
                        department.as_deref(),
                        output_dir,
                        "md",
                    )
                }
                OutputFormat::Json => {
                    let output_repo = JsonOutputRepository::new();
                    let generator =
                        ReportGenerator::new(config_repo, github_repo, document_repo, output_repo);
                    generator.generate(
                        Path::new(&config),
                        year,
                        department.as_deref(),
                        output_dir,
                        "json",
                    )
                }
                OutputFormat::Html => {
                    let output_repo = HtmlOutputRepository::new();
                    let generator =
                        ReportGenerator::new(config_repo, github_repo, document_repo, output_repo);
                    generator.generate(
                        Path::new(&config),
                        year,
                        department.as_deref(),
                        output_dir,
                        "html",
                    )
                }
            };

            match result {
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
