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
        // Calculate total commits from theme summary
        let your_commits_count: usize = report.theme_summary().values().map(|&v| v as usize).sum();

        let mut content = format!(
            r#"# Annual Report {}

## {}

### Period

- From: {}
- To: {}

### Organization Activity Summary

- Total Commits: {}
- Total Pull Requests: {}
- Total Issues: {}
- Total Reviews: {}

### Your Activity

- Your Commits: {}
"#,
            report.year(),
            report.department_name(),
            report.period_from(),
            report.period_to(),
            report.github_activity().commits(),
            report.github_activity().pull_requests(),
            report.github_activity().issues(),
            report.github_activity().reviews(),
            your_commits_count,
        );

        // Theme Summary (Conventional Commits)
        if !report.theme_summary().is_empty() {
            content.push_str("\n#### Commit Themes\n\n");
            let mut themes: Vec<_> = report.theme_summary().iter().collect();
            themes.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending

            for (theme, count) in themes {
                content.push_str(&format!("- {}: {}\n", theme.display_name(), count));
            }
        }

        // Local Documents (only show if there are documents)
        if !report.documents().is_empty() {
            content.push_str("\n### Local Documents\n\n");
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
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    #[allow(non_snake_case)]
    fn Markdownファイルにレポートを出力できる() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("test_report.md");

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

        let repository = MarkdownOutputRepository::new();

        repository
            .output(&report, &output_path)
            .expect("Failed to output report");

        let content = std::fs::read_to_string(&output_path).expect("Failed to read output file");
        assert!(content.contains("# Annual Report 2024"));
        assert!(content.contains("## 個人"));
        assert!(content.contains("### Organization Activity Summary"));
        assert!(content.contains("Total Commits: 100"));
        assert!(content.contains("### Your Activity"));
        assert!(content.contains("Your Commits: 0")); // No theme summary, so 0
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

        let report = Report::new(
            2024,
            "個人".to_string(),
            from,
            to,
            activity,
            documents,
            HashMap::new(),
        );

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
    fn ドキュメントがない場合はLocal_Documentsセクションを表示しない() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("test_report_no_docs.md");

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

        let repository = MarkdownOutputRepository::new();

        repository
            .output(&report, &output_path)
            .expect("Failed to output report");

        let content = std::fs::read_to_string(&output_path).expect("Failed to read output file");
        assert!(!content.contains("### Local Documents")); // Should not contain Local Documents section
    }
}
