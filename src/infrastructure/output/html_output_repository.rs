use crate::domain::entities::report::Report;
use crate::domain::repositories::output_repository::OutputRepository;
use anyhow::Result;
use std::path::Path;

/// HTML output repository
#[allow(dead_code)]
pub struct HtmlOutputRepository;

impl HtmlOutputRepository {
    /// Creates a new HtmlOutputRepository instance
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
}

impl OutputRepository for HtmlOutputRepository {
    fn output(&self, report: &Report, path: &Path) -> Result<()> {
        let mut content = format!(
            r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Annual Report {}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            background-color: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            border-bottom: 2px solid #007bff;
            padding-bottom: 10px;
        }}
        h2 {{
            color: #555;
            margin-top: 30px;
        }}
        h3 {{
            color: #666;
            margin-top: 20px;
        }}
        ul {{
            list-style-type: none;
            padding-left: 0;
        }}
        li {{
            padding: 8px 0;
            border-bottom: 1px solid #eee;
        }}
        .stat {{
            font-weight: bold;
            color: #007bff;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Annual Report {}</h1>
        <h2>{}</h2>

        <h3>Period</h3>
        <ul>
            <li>From: {}</li>
            <li>To: {}</li>
        </ul>

        <h3>GitHub Activity</h3>
        <ul>
            <li>Commits: <span class="stat">{}</span></li>
            <li>Pull Requests: <span class="stat">{}</span></li>
            <li>Issues: <span class="stat">{}</span></li>
            <li>Reviews: <span class="stat">{}</span></li>
        </ul>

        <h3>Local Documents</h3>
"#,
            report.year(),
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
            content.push_str("        <p>(No documents)</p>\n");
        } else {
            content.push_str("        <ul>\n");
            for doc in report.documents() {
                content.push_str(&format!("            <li>{}</li>\n", doc.file_path()));
            }
            content.push_str("        </ul>\n");
        }

        // Theme Summary (Conventional Commits)
        if !report.theme_summary().is_empty() {
            content.push_str("\n        <h3>Commit Themes</h3>\n");
            content.push_str("        <ul>\n");
            let mut themes: Vec<_> = report.theme_summary().iter().collect();
            themes.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending

            for (theme, count) in themes {
                content.push_str(&format!(
                    "            <li>{}: <span class=\"stat\">{}</span></li>\n",
                    theme.display_name(),
                    count
                ));
            }
            content.push_str("        </ul>\n");
        }

        content.push_str(
            r#"    </div>
</body>
</html>
"#,
        );

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
    fn HTMLファイルにレポートを出力できる() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("test_report.html");

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

        let repository = HtmlOutputRepository::new();

        repository
            .output(&report, &output_path)
            .expect("Failed to output report");

        let content = std::fs::read_to_string(&output_path).expect("Failed to read output file");

        // Verify HTML structure
        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains("<title>Annual Report 2024</title>"));
        assert!(content.contains("<h1>Annual Report 2024</h1>"));
        assert!(content.contains("<h2>個人</h2>"));
        assert!(content.contains("Commits: <span class=\"stat\">100</span>"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn ドキュメント付きのHTMLレポートを出力できる() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("test_report_with_docs.html");

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

        let repository = HtmlOutputRepository::new();

        repository
            .output(&report, &output_path)
            .expect("Failed to output report");

        let content = std::fs::read_to_string(&output_path).expect("Failed to read output file");

        assert!(content.contains("<h3>Local Documents</h3>"));
        assert!(content.contains("<li>doc1.md</li>"));
        assert!(content.contains("<li>doc2.md</li>"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn ドキュメントがない場合は該当なしと表示する() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("test_report_no_docs.html");

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

        let repository = HtmlOutputRepository::new();

        repository
            .output(&report, &output_path)
            .expect("Failed to output report");

        let content = std::fs::read_to_string(&output_path).expect("Failed to read output file");

        assert!(content.contains("<h3>Local Documents</h3>"));
        assert!(content.contains("<p>(No documents)</p>"));
    }
}
