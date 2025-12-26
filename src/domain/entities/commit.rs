use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a Git commit
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    sha: String,
    message: String,
    author: String,
    committed_date: DateTime<Utc>,
    repository: String,
}

impl Commit {
    /// Creates a new Commit instance
    pub fn new(
        sha: String,
        message: String,
        author: String,
        committed_date: DateTime<Utc>,
        repository: String,
    ) -> Self {
        Self {
            sha,
            message,
            author,
            committed_date,
            repository,
        }
    }

    /// Returns the SHA
    #[allow(dead_code)]
    pub fn sha(&self) -> &str {
        &self.sha
    }

    /// Returns the commit message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the author name
    #[allow(dead_code)]
    pub fn author(&self) -> &str {
        &self.author
    }

    /// Returns the committed date
    #[allow(dead_code)]
    pub fn committed_date(&self) -> DateTime<Utc> {
        self.committed_date
    }

    /// Returns the repository name
    #[allow(dead_code)]
    pub fn repository(&self) -> &str {
        &self.repository
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    #[allow(non_snake_case)]
    fn コミットを作成できる() {
        let date = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let commit = Commit::new(
            "abc123".to_string(),
            "feat: add new feature".to_string(),
            "John Doe".to_string(),
            date,
            "test-repo".to_string(),
        );

        assert_eq!(commit.sha(), "abc123");
        assert_eq!(commit.message(), "feat: add new feature");
        assert_eq!(commit.author(), "John Doe");
        assert_eq!(commit.committed_date(), date);
        assert_eq!(commit.repository(), "test-repo");
    }

    #[test]
    #[allow(non_snake_case)]
    fn コミットをクローンできる() {
        let date = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let commit = Commit::new(
            "abc123".to_string(),
            "fix: resolve bug".to_string(),
            "Jane Smith".to_string(),
            date,
            "test-repo".to_string(),
        );

        let cloned = commit.clone();
        assert_eq!(commit, cloned);
    }

    #[test]
    #[allow(non_snake_case)]
    fn コミットをシリアライズできる() {
        let date = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        let commit = Commit::new(
            "abc123".to_string(),
            "docs: update README".to_string(),
            "Bob Johnson".to_string(),
            date,
            "test-repo".to_string(),
        );

        let json = serde_json::to_string(&commit).expect("Failed to serialize");
        let deserialized: Commit = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(commit, deserialized);
    }
}
