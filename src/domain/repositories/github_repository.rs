use crate::domain::entities::commit::Commit;
use crate::domain::entities::github_activity::GitHubActivity;
use anyhow::Result;
use chrono::NaiveDate;

/// Repository trait for fetching GitHub data
#[allow(dead_code)] // Temporarily allowed during TDD implementation
pub trait GitHubRepository {
    /// Fetches GitHub activity for the specified organization/user within the given period
    ///
    /// # Arguments
    ///
    /// * `org_or_user` - GitHub organization or user name
    /// * `from` - Start date (inclusive)
    /// * `to` - End date (inclusive)
    fn fetch_activity(
        &self,
        org_or_user: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<GitHubActivity>;

    /// Fetches all commits for the specified organization/user within the given period
    ///
    /// # Arguments
    ///
    /// * `org_or_user` - GitHub organization or user name
    /// * `from` - Start date (inclusive)
    /// * `to` - End date (inclusive)
    ///
    /// # Returns
    ///
    /// A vector of all commits within the period, using pagination to fetch all results
    fn fetch_commits(
        &self,
        org_or_user: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Commit>>;
}
