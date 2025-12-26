use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "nenpo")]
#[command(about = "Annual report generator from GitHub and local documents", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate annual report
    Generate {
        /// Configuration file path
        #[arg(long, default_value = "./nenpou.toml")]
        config: String,

        /// Target year
        #[arg(long)]
        year: Option<u32>,

        /// Specific department name
        #[arg(long)]
        department: Option<String>,

        /// Output format (markdown, json, html)
        #[arg(long)]
        format: Option<String>,
    },
}
