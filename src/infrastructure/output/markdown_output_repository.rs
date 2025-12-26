use crate::domain::entities::report::Report;
use crate::domain::repositories::output_repository::OutputRepository;
use anyhow::Result;
use std::path::Path;

/// Markdown output repository
#[allow(dead_code)] // Temporarily allowed during TDD implementation
pub struct MarkdownOutputRepository;

impl MarkdownOutputRepository {
    /// Creates a new MarkdownOutputRepository instance
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn new() -> Self {
        Self
    }
}

impl OutputRepository for MarkdownOutputRepository {
    fn output(&self, report: &Report, path: &Path) -> Result<()> {
        let content = format!(
            r#"# Annual Report {}

## {}

### Period
- From: {}
- To: {}

### GitHub Activity
- Commits: {}
- Pull Requests: {}
- Issues: {}
- Reviews: {}
"#,
            report.year(),
            report.department_name(),
            report.period_from(),
            report.period_to(),
            report.github_activity().commits(),
            report.github_activity().pull_requests(),
            report.github_activity().issues(),
            report.github_activity().reviews(),
        );

        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::github_activity::GitHubActivity;
    use chrono::NaiveDate;

    #[test]
    #[allow(non_snake_case)]
    fn Markdownファイルにレポートを出力できる() {
        let activity = GitHubActivity::new(100, 20, 15, 30);
        let from = NaiveDate::from_ymd_opt(2024, 4, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2025, 3, 31).expect("Invalid date");
        let report = Report::new(2024, "個人".to_string(), from, to, activity);

        let repository = MarkdownOutputRepository::new();
        let output_path = Path::new("/tmp/test_report.md");

        repository
            .output(&report, output_path)
            .expect("Failed to output report");

        let content = std::fs::read_to_string(output_path).expect("Failed to read output file");
        assert!(content.contains("# Annual Report 2024"));
        assert!(content.contains("## 個人"));
        assert!(content.contains("Commits: 100"));

        std::fs::remove_file(output_path).expect("Failed to remove test file");
    }
}
