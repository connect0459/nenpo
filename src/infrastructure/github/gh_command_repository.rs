use crate::domain::entities::github_activity::GitHubActivity;
use crate::domain::repositories::github_repository::GitHubRepository;
use anyhow::Result;
use chrono::NaiveDate;

/// GitHub repository implementation using gh command
#[allow(dead_code)] // Temporarily allowed during TDD implementation
pub struct GhCommandRepository;

impl GhCommandRepository {
    /// Creates a new GhCommandRepository instance
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn new() -> Self {
        Self
    }
}

impl GitHubRepository for GhCommandRepository {
    fn fetch_activity(
        &self,
        _org_or_user: &str,
        _from: NaiveDate,
        _to: NaiveDate,
    ) -> Result<GitHubActivity> {
        // Phase 1 MVP: Returns dummy data
        // TODO: Implement actual gh command execution in future iterations
        Ok(GitHubActivity::new(0, 0, 0, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    #[allow(non_snake_case)]
    fn GhCommandRepositoryを作成できる() {
        let _repository = GhCommandRepository::new();
        // Successfully created
    }

    #[test]
    #[allow(non_snake_case)]
    fn GitHub活動データを取得できる() {
        let repository = GhCommandRepository::new();
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2024, 12, 31).expect("Invalid date");

        // Phase 1 MVP: Returns dummy data for now
        // Actual gh command implementation will be added later
        let activity = repository
            .fetch_activity("test-org", from, to)
            .expect("Failed to fetch activity");

        // Verify that we got a valid GitHubActivity instance
        // Phase 1 MVP returns dummy data (all zeros)
        assert_eq!(activity.commits(), 0);
        assert_eq!(activity.pull_requests(), 0);
        assert_eq!(activity.issues(), 0);
        assert_eq!(activity.reviews(), 0);
    }
}
