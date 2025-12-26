use crate::domain::entities::department::Department;
use crate::domain::value_objects::output_format::OutputFormat;
use serde::{Deserialize, Serialize};

/// Represents the application configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)] // Temporarily allowed during TDD implementation
pub struct Config {
    default_fiscal_year_start_month: u32,
    default_output_format: OutputFormat,
    output_directory: String,
    departments: Vec<Department>,
}

impl Config {
    /// Creates a new Config instance
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn new(
        default_fiscal_year_start_month: u32,
        default_output_format: OutputFormat,
        output_directory: String,
        departments: Vec<Department>,
    ) -> Self {
        Self {
            default_fiscal_year_start_month,
            default_output_format,
            output_directory,
            departments,
        }
    }

    /// Returns the default fiscal year start month
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn default_fiscal_year_start_month(&self) -> u32 {
        self.default_fiscal_year_start_month
    }

    /// Returns the default output format
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn default_output_format(&self) -> OutputFormat {
        self.default_output_format
    }

    /// Returns the output directory path
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn output_directory(&self) -> &str {
        &self.output_directory
    }

    /// Returns the list of departments
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn departments(&self) -> &[Department] {
        &self.departments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn 設定を作成できる() {
        let config = Config::new(4, OutputFormat::Markdown, "./reports".to_string(), vec![]);

        assert_eq!(config.default_fiscal_year_start_month(), 4);
        assert_eq!(config.default_output_format(), OutputFormat::Markdown);
        assert_eq!(config.output_directory(), "./reports");
        assert_eq!(config.departments().len(), 0);
    }

    #[test]
    #[allow(non_snake_case)]
    fn 部門を含む設定を作成できる() {
        let department = Department::new(
            "個人".to_string(),
            4,
            vec!["connect0459".to_string()],
            vec![],
        );

        let config = Config::new(
            4,
            OutputFormat::Json,
            "./output".to_string(),
            vec![department.clone()],
        );

        assert_eq!(config.departments().len(), 1);
        assert_eq!(config.departments()[0].name(), "個人");
    }
}
