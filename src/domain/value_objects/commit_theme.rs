use serde::{Deserialize, Serialize};

/// Represents a commit theme based on Conventional Commits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommitTheme {
    /// New features
    Feat,
    /// Bug fixes
    Fix,
    /// Documentation changes
    Docs,
    /// Code refactoring
    Refactor,
    /// Tests
    Test,
    /// Build system or dependencies
    Build,
    /// CI/CD changes
    Ci,
    /// Performance improvements
    Perf,
    /// Code style changes
    Style,
    /// Chores (maintenance)
    Chore,
    /// Other/unknown commits
    Other,
}

impl CommitTheme {
    /// Parses a commit message and extracts the theme
    #[allow(dead_code)] // Will be used when implementing commit message fetching
    pub fn from_commit_message(message: &str) -> Self {
        let message_lower = message.to_lowercase();
        let prefix = message_lower.split(':').next().unwrap_or("");

        match prefix.trim() {
            "feat" => CommitTheme::Feat,
            "fix" => CommitTheme::Fix,
            "docs" => CommitTheme::Docs,
            "refactor" => CommitTheme::Refactor,
            "test" => CommitTheme::Test,
            "build" => CommitTheme::Build,
            "ci" => CommitTheme::Ci,
            "perf" => CommitTheme::Perf,
            "style" => CommitTheme::Style,
            "chore" => CommitTheme::Chore,
            _ => CommitTheme::Other,
        }
    }

    /// Returns the display name of the theme
    pub fn display_name(&self) -> &str {
        match self {
            CommitTheme::Feat => "New Features",
            CommitTheme::Fix => "Bug Fixes",
            CommitTheme::Docs => "Documentation",
            CommitTheme::Refactor => "Refactoring",
            CommitTheme::Test => "Tests",
            CommitTheme::Build => "Build System",
            CommitTheme::Ci => "CI/CD",
            CommitTheme::Perf => "Performance",
            CommitTheme::Style => "Code Style",
            CommitTheme::Chore => "Chores",
            CommitTheme::Other => "Other",
        }
    }

    /// Returns a short name for the theme
    #[allow(dead_code)] // Will be used when implementing commit message fetching
    pub fn short_name(&self) -> &str {
        match self {
            CommitTheme::Feat => "feat",
            CommitTheme::Fix => "fix",
            CommitTheme::Docs => "docs",
            CommitTheme::Refactor => "refactor",
            CommitTheme::Test => "test",
            CommitTheme::Build => "build",
            CommitTheme::Ci => "ci",
            CommitTheme::Perf => "perf",
            CommitTheme::Style => "style",
            CommitTheme::Chore => "chore",
            CommitTheme::Other => "other",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn コミットメッセージからテーマを抽出できる() {
        assert_eq!(
            CommitTheme::from_commit_message("feat: add new feature"),
            CommitTheme::Feat
        );
        assert_eq!(
            CommitTheme::from_commit_message("fix: resolve bug"),
            CommitTheme::Fix
        );
        assert_eq!(
            CommitTheme::from_commit_message("docs: update README"),
            CommitTheme::Docs
        );
        assert_eq!(
            CommitTheme::from_commit_message("refactor: improve code structure"),
            CommitTheme::Refactor
        );
        assert_eq!(
            CommitTheme::from_commit_message("test: add unit tests"),
            CommitTheme::Test
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn 大文字小文字を区別しない() {
        assert_eq!(
            CommitTheme::from_commit_message("FEAT: Add New Feature"),
            CommitTheme::Feat
        );
        assert_eq!(
            CommitTheme::from_commit_message("Fix: Resolve Bug"),
            CommitTheme::Fix
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn 形式に従わないコミットはOtherになる() {
        assert_eq!(
            CommitTheme::from_commit_message("add new feature"),
            CommitTheme::Other
        );
        assert_eq!(
            CommitTheme::from_commit_message("update something"),
            CommitTheme::Other
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn 表示名を取得できる() {
        assert_eq!(CommitTheme::Feat.display_name(), "New Features");
        assert_eq!(CommitTheme::Fix.display_name(), "Bug Fixes");
        assert_eq!(CommitTheme::Docs.display_name(), "Documentation");
    }

    #[test]
    #[allow(non_snake_case)]
    fn 短縮名を取得できる() {
        assert_eq!(CommitTheme::Feat.short_name(), "feat");
        assert_eq!(CommitTheme::Fix.short_name(), "fix");
        assert_eq!(CommitTheme::Docs.short_name(), "docs");
    }
}
