use crate::domain::entities::document_content::DocumentContent;
use crate::domain::entities::github_activity::GitHubActivity;
use crate::domain::value_objects::commit_theme::CommitTheme;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an annual report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)] // Temporarily allowed during TDD implementation
pub struct Report {
    year: u32,
    department_name: String,
    period_from: NaiveDate,
    period_to: NaiveDate,
    github_activity: GitHubActivity,
    documents: Vec<DocumentContent>,
    theme_summary: HashMap<CommitTheme, u32>,
}

impl Report {
    /// Creates a new Report instance
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn new(
        year: u32,
        department_name: String,
        period_from: NaiveDate,
        period_to: NaiveDate,
        github_activity: GitHubActivity,
        documents: Vec<DocumentContent>,
        theme_summary: HashMap<CommitTheme, u32>,
    ) -> Self {
        Self {
            year,
            department_name,
            period_from,
            period_to,
            github_activity,
            documents,
            theme_summary,
        }
    }

    /// Returns the year
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn year(&self) -> u32 {
        self.year
    }

    /// Returns the department name
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn department_name(&self) -> &str {
        &self.department_name
    }

    /// Returns the period start date
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn period_from(&self) -> NaiveDate {
        self.period_from
    }

    /// Returns the period end date
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn period_to(&self) -> NaiveDate {
        self.period_to
    }

    /// Returns the GitHub activity
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn github_activity(&self) -> &GitHubActivity {
        &self.github_activity
    }

    /// Returns the documents
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn documents(&self) -> &[DocumentContent] {
        &self.documents
    }

    /// Returns the theme summary
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn theme_summary(&self) -> &HashMap<CommitTheme, u32> {
        &self.theme_summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_report() {
        let activity = GitHubActivity::new(100, 20, 15, 30);
        let from = NaiveDate::from_ymd_opt(2024, 4, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2025, 3, 31).expect("Invalid date");

        let report = Report::new(
            2024,
            "Personal".to_string(),
            from,
            to,
            activity.clone(),
            vec![],
            HashMap::new(),
        );

        assert_eq!(report.year(), 2024);
        assert_eq!(report.department_name(), "Personal");
        assert_eq!(report.period_from(), from);
        assert_eq!(report.period_to(), to);
        assert_eq!(report.github_activity(), &activity);
        assert_eq!(report.documents().len(), 0);
    }

    #[test]
    fn creates_report_with_documents() {
        let activity = GitHubActivity::new(100, 20, 15, 30);
        let from = NaiveDate::from_ymd_opt(2024, 4, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2025, 3, 31).expect("Invalid date");

        let documents = vec![
            DocumentContent::new("doc1.md".to_string(), "Content 1".to_string()),
            DocumentContent::new("doc2.md".to_string(), "Content 2".to_string()),
        ];

        let report = Report::new(
            2024,
            "Personal".to_string(),
            from,
            to,
            activity,
            documents.clone(),
            HashMap::new(),
        );

        assert_eq!(report.documents().len(), 2);
        assert_eq!(report.documents()[0].file_path(), "doc1.md");
        assert_eq!(report.documents()[1].file_path(), "doc2.md");
    }
}
