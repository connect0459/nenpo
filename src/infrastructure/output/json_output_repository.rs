use crate::domain::entities::report::Report;
use crate::domain::repositories::output_repository::OutputRepository;
use anyhow::Result;
use std::path::Path;

/// JSON output repository
#[allow(dead_code)]
pub struct JsonOutputRepository;

impl JsonOutputRepository {
    /// Creates a new JsonOutputRepository instance
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
}

impl OutputRepository for JsonOutputRepository {
    fn output(&self, report: &Report, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(report)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::github_activity::GitHubActivity;
    use chrono::NaiveDate;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn outputs_report_to_json_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("test_report.json");

        let activity = GitHubActivity::new(100, 20, 15, 30);
        let from = NaiveDate::from_ymd_opt(2024, 4, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2025, 3, 31).expect("Invalid date");
        let report = Report::new(
            2024,
            "個人".to_string(),
            from,
            to,
            activity,
            vec![],
            HashMap::new(),
        );

        let repository = JsonOutputRepository::new();

        repository
            .output(&report, &output_path)
            .expect("Failed to output report");

        let content = std::fs::read_to_string(&output_path).expect("Failed to read output file");

        // Verify JSON structure
        assert!(content.contains("\"year\": 2024"));
        assert!(content.contains("\"department_name\": \"個人\""));
        assert!(content.contains("\"github_activity\""));
        assert!(content.contains("\"commits\": 100"));
    }

    #[test]
    fn loads_output_json() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("test_report_roundtrip.json");

        let activity = GitHubActivity::new(100, 20, 15, 30);
        let from = NaiveDate::from_ymd_opt(2024, 4, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2025, 3, 31).expect("Invalid date");
        let original_report = Report::new(
            2024,
            "個人".to_string(),
            from,
            to,
            activity,
            vec![],
            HashMap::new(),
        );

        let repository = JsonOutputRepository::new();

        repository
            .output(&original_report, &output_path)
            .expect("Failed to output report");

        // Read and deserialize
        let content = std::fs::read_to_string(&output_path).expect("Failed to read output file");
        let deserialized_report: Report =
            serde_json::from_str(&content).expect("Failed to deserialize JSON");

        assert_eq!(deserialized_report.year(), 2024);
        assert_eq!(deserialized_report.department_name(), "個人");
        assert_eq!(deserialized_report.github_activity().commits(), 100);
    }
}
