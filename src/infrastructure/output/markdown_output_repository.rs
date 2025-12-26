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
        let mut content = format!(
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

### Local Documents
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

        if report.documents().is_empty() {
            content.push_str("(No documents)\n");
        } else {
            for doc in report.documents() {
                content.push_str(&format!("- {}\n", doc.file_path()));
            }
        }

        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::document_content::DocumentContent;
    use crate::domain::entities::github_activity::GitHubActivity;
    use chrono::NaiveDate;
    use tempfile::TempDir;

    #[test]
    #[allow(non_snake_case)]
    fn Markdownファイルにレポートを出力できる() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("test_report.md");

        let activity = GitHubActivity::new(100, 20, 15, 30);
        let from = NaiveDate::from_ymd_opt(2024, 4, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2025, 3, 31).expect("Invalid date");
        let report = Report::new(2024, "個人".to_string(), from, to, activity, vec![]);

        let repository = MarkdownOutputRepository::new();

        repository
            .output(&report, &output_path)
            .expect("Failed to output report");

        let content = std::fs::read_to_string(&output_path).expect("Failed to read output file");
        assert!(content.contains("# Annual Report 2024"));
        assert!(content.contains("## 個人"));
        assert!(content.contains("Commits: 100"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn ドキュメント付きのレポートを出力できる() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("test_report_with_docs.md");

        let activity = GitHubActivity::new(100, 20, 15, 30);
        let from = NaiveDate::from_ymd_opt(2024, 4, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2025, 3, 31).expect("Invalid date");

        let documents = vec![
            DocumentContent::new("doc1.md".to_string(), "Content 1".to_string()),
            DocumentContent::new("doc2.md".to_string(), "Content 2".to_string()),
        ];

        let report = Report::new(2024, "個人".to_string(), from, to, activity, documents);

        let repository = MarkdownOutputRepository::new();

        repository
            .output(&report, &output_path)
            .expect("Failed to output report");

        let content = std::fs::read_to_string(&output_path).expect("Failed to read output file");
        assert!(content.contains("# Annual Report 2024"));
        assert!(content.contains("### Local Documents"));
        assert!(content.contains("- doc1.md"));
        assert!(content.contains("- doc2.md"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn ドキュメントがない場合は該当なしと表示する() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("test_report_no_docs.md");

        let activity = GitHubActivity::new(100, 20, 15, 30);
        let from = NaiveDate::from_ymd_opt(2024, 4, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2025, 3, 31).expect("Invalid date");
        let report = Report::new(2024, "個人".to_string(), from, to, activity, vec![]);

        let repository = MarkdownOutputRepository::new();

        repository
            .output(&report, &output_path)
            .expect("Failed to output report");

        let content = std::fs::read_to_string(&output_path).expect("Failed to read output file");
        assert!(content.contains("### Local Documents"));
        assert!(content.contains("(No documents)"));
    }
}
