use serde::{Deserialize, Serialize};

/// Represents GitHub activity statistics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)] // Temporarily allowed during TDD implementation
pub struct GitHubActivity {
    commits: u32,
    pull_requests: u32,
    issues: u32,
    reviews: u32,
}

impl GitHubActivity {
    /// Creates a new GitHubActivity instance
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn new(commits: u32, pull_requests: u32, issues: u32, reviews: u32) -> Self {
        Self {
            commits,
            pull_requests,
            issues,
            reviews,
        }
    }

    /// Returns the number of commits
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn commits(&self) -> u32 {
        self.commits
    }

    /// Returns the number of pull requests
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn pull_requests(&self) -> u32 {
        self.pull_requests
    }

    /// Returns the number of issues
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn issues(&self) -> u32 {
        self.issues
    }

    /// Returns the number of reviews
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn reviews(&self) -> u32 {
        self.reviews
    }

    /// Adds another GitHubActivity to this one and returns the result
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn add(&self, other: &GitHubActivity) -> GitHubActivity {
        GitHubActivity {
            commits: self.commits + other.commits,
            pull_requests: self.pull_requests + other.pull_requests,
            issues: self.issues + other.issues,
            reviews: self.reviews + other.reviews,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_github_activity() {
        let activity = GitHubActivity::new(100, 20, 15, 30);

        assert_eq!(activity.commits(), 100);
        assert_eq!(activity.pull_requests(), 20);
        assert_eq!(activity.issues(), 15);
        assert_eq!(activity.reviews(), 30);
    }

    #[test]
    fn creates_github_activity_with_zeros() {
        let activity = GitHubActivity::new(0, 0, 0, 0);

        assert_eq!(activity.commits(), 0);
        assert_eq!(activity.pull_requests(), 0);
        assert_eq!(activity.issues(), 0);
        assert_eq!(activity.reviews(), 0);
    }

    #[test]
    fn adds_activities() {
        let activity1 = GitHubActivity::new(100, 20, 15, 30);
        let activity2 = GitHubActivity::new(50, 10, 5, 15);

        let total = activity1.add(&activity2);

        assert_eq!(total.commits(), 150);
        assert_eq!(total.pull_requests(), 30);
        assert_eq!(total.issues(), 20);
        assert_eq!(total.reviews(), 45);
    }
}
