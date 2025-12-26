use crate::domain::entities::config::Config;
use crate::domain::entities::department::Department;
use crate::domain::repositories::config_repository::ConfigRepository;
use crate::domain::value_objects::output_format::OutputFormat;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

/// Intermediate structure for deserializing TOML
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Used for TOML deserialization
struct TomlConfig {
    #[serde(default)]
    target_github_user: Option<String>,
    default_fiscal_year_start_month: u32,
    default_output_format: String,
    output_directory: String,
    departments: Vec<TomlDepartment>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Used for TOML deserialization
struct TomlDepartment {
    name: String,
    fiscal_year_start_month: u32,
    github_organizations: Vec<String>,
    local_documents: Vec<String>,
}

/// TOML-based configuration repository
#[allow(dead_code)] // Temporarily allowed during TDD implementation
pub struct TomlConfigRepository;

impl TomlConfigRepository {
    /// Creates a new TomlConfigRepository instance
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn new() -> Self {
        Self
    }
}

impl ConfigRepository for TomlConfigRepository {
    fn load(&self, path: &Path) -> Result<Config> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let toml_config: TomlConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML config: {:?}", path))?;

        let output_format = OutputFormat::from_str(&toml_config.default_output_format)
            .with_context(|| {
                format!(
                    "Invalid output format: {}",
                    toml_config.default_output_format
                )
            })?;

        let departments: Vec<Department> = toml_config
            .departments
            .into_iter()
            .map(|d| {
                Department::new(
                    d.name,
                    d.fiscal_year_start_month,
                    d.github_organizations,
                    d.local_documents,
                )
            })
            .collect();

        Ok(Config::with_target_user(
            toml_config.target_github_user,
            toml_config.default_fiscal_year_start_month,
            output_format,
            toml_config.output_directory,
            departments,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn loads_config_from_toml_file() {
        let toml_content = r#"
default_fiscal_year_start_month = 4
default_output_format = "markdown"
output_directory = "./reports"

[[departments]]
name = "Personal"
fiscal_year_start_month = 4
github_organizations = ["connect0459"]
local_documents = []

[[departments]]
name = "Corporate"
fiscal_year_start_month = 4
github_organizations = ["voyagegroup"]
local_documents = ["docs/**/*.md"]
"#;

        let temp_file = "/tmp/test_config.toml";
        let mut file = fs::File::create(temp_file).expect("Failed to create temp file");
        file.write_all(toml_content.as_bytes())
            .expect("Failed to write temp file");

        let repository = TomlConfigRepository::new();
        let config = repository
            .load(Path::new(temp_file))
            .expect("Failed to load config");

        assert_eq!(config.default_fiscal_year_start_month(), 4);
        assert_eq!(config.default_output_format(), OutputFormat::Markdown);
        assert_eq!(config.output_directory(), "./reports");
        assert_eq!(config.target_github_user(), None);
        assert_eq!(config.departments().len(), 2);
        assert_eq!(config.departments()[0].name(), "Personal");
        assert_eq!(config.departments()[1].name(), "Corporate");

        fs::remove_file(temp_file).expect("Failed to remove temp file");
    }

    #[test]
    fn loads_config_with_target_github_user() {
        let toml_content = r#"
target_github_user = "connect0459"
default_fiscal_year_start_month = 1
default_output_format = "markdown"
output_directory = "./reports"

[[departments]]
name = "Personal"
fiscal_year_start_month = 1
github_organizations = ["connect0459"]
local_documents = []
"#;

        let temp_file = "/tmp/test_config_with_user.toml";
        let mut file = fs::File::create(temp_file).expect("Failed to create temp file");
        file.write_all(toml_content.as_bytes())
            .expect("Failed to write temp file");

        let repository = TomlConfigRepository::new();
        let config = repository
            .load(Path::new(temp_file))
            .expect("Failed to load config");

        assert_eq!(config.target_github_user(), Some("connect0459"));
        assert_eq!(config.default_fiscal_year_start_month(), 1);
        assert_eq!(config.departments().len(), 1);

        fs::remove_file(temp_file).expect("Failed to remove temp file");
    }

    #[test]
    fn returns_error_when_loading_nonexistent_file() {
        let repository = TomlConfigRepository::new();
        let result = repository.load(Path::new("/tmp/nonexistent_config.toml"));
        assert!(result.is_err());
    }
}
