use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// Represents the output format for reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)] // Temporarily allowed during TDD implementation
pub enum OutputFormat {
    Markdown,
    Json,
    Html,
}

impl OutputFormat {
    /// Parses a string into an OutputFormat
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "markdown" => Ok(OutputFormat::Markdown),
            "json" => Ok(OutputFormat::Json),
            "html" => Ok(OutputFormat::Html),
            _ => Err(anyhow!("Invalid output format: {}", s)),
        }
    }

    /// Converts the OutputFormat to a string
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn as_str(&self) -> &str {
        match self {
            OutputFormat::Markdown => "markdown",
            OutputFormat::Json => "json",
            OutputFormat::Html => "html",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn 文字列からOutputFormatに変換できる() {
        assert_eq!(
            OutputFormat::from_str("markdown").expect("Failed to parse markdown"),
            OutputFormat::Markdown
        );
        assert_eq!(
            OutputFormat::from_str("json").expect("Failed to parse json"),
            OutputFormat::Json
        );
        assert_eq!(
            OutputFormat::from_str("html").expect("Failed to parse html"),
            OutputFormat::Html
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn 無効な文字列からの変換はエラーになる() {
        assert!(OutputFormat::from_str("invalid").is_err());
        assert!(OutputFormat::from_str("pdf").is_err());
    }

    #[test]
    #[allow(non_snake_case)]
    fn OutputFormatを文字列に変換できる() {
        assert_eq!(OutputFormat::Markdown.as_str(), "markdown");
        assert_eq!(OutputFormat::Json.as_str(), "json");
        assert_eq!(OutputFormat::Html.as_str(), "html");
    }
}
