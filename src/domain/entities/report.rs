use crate::domain::entities::github_activity::GitHubActivity;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Represents an annual report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)] // Temporarily allowed during TDD implementation
pub struct Report {
    year: u32,
    department_name: String,
    period_from: NaiveDate,
    period_to: NaiveDate,
    github_activity: GitHubActivity,
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
    ) -> Self {
        Self {
            year,
            department_name,
            period_from,
            period_to,
            github_activity,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn レポートを作成できる() {
        let activity = GitHubActivity::new(100, 20, 15, 30);
        let from = NaiveDate::from_ymd_opt(2024, 4, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2025, 3, 31).expect("Invalid date");

        let report = Report::new(2024, "個人".to_string(), from, to, activity.clone());

        assert_eq!(report.year(), 2024);
        assert_eq!(report.department_name(), "個人");
        assert_eq!(report.period_from(), from);
        assert_eq!(report.period_to(), to);
        assert_eq!(report.github_activity(), &activity);
    }
}
