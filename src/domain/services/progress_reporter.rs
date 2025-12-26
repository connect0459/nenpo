/// Trait for reporting progress during long-running operations
pub trait ProgressReporter {
    /// Reports the start of fetching commits for an organization/user
    fn start_fetching_commits(&self, org_or_user: &str);

    /// Reports progress of commit fetching
    ///
    /// # Arguments
    ///
    /// * `org_or_user` - GitHub organization or user name
    /// * `fetched_count` - Number of commits fetched so far
    fn report_commits_progress(&self, org_or_user: &str, fetched_count: usize);

    /// Reports completion of commit fetching
    ///
    /// # Arguments
    ///
    /// * `org_or_user` - GitHub organization or user name
    /// * `total_count` - Total number of commits fetched
    fn finish_fetching_commits(&self, org_or_user: &str, total_count: usize);

    /// Reports an error during operations
    #[allow(dead_code)]
    fn report_error(&self, error: &str);
}

/// Progress reporter that outputs to stdout
pub struct StdoutProgressReporter;

impl StdoutProgressReporter {
    pub fn new() -> Self {
        Self
    }
}

impl ProgressReporter for StdoutProgressReporter {
    fn start_fetching_commits(&self, org_or_user: &str) {
        eprintln!("Fetching commits for {}...", org_or_user);
    }

    fn report_commits_progress(&self, org_or_user: &str, fetched_count: usize) {
        eprintln!(
            "  {} commits fetched from {}...",
            fetched_count, org_or_user
        );
    }

    fn finish_fetching_commits(&self, org_or_user: &str, total_count: usize) {
        eprintln!(
            "✓ Finished fetching {} commits from {}",
            total_count, org_or_user
        );
    }

    fn report_error(&self, error: &str) {
        eprintln!("✗ Error: {}", error);
    }
}

/// No-op progress reporter for testing or when progress reporting is not needed
#[allow(dead_code)]
pub struct NoOpProgressReporter;

#[allow(dead_code)]
impl NoOpProgressReporter {
    pub fn new() -> Self {
        Self
    }
}

impl ProgressReporter for NoOpProgressReporter {
    fn start_fetching_commits(&self, _org_or_user: &str) {}
    fn report_commits_progress(&self, _org_or_user: &str, _fetched_count: usize) {}
    fn finish_fetching_commits(&self, _org_or_user: &str, _total_count: usize) {}
    fn report_error(&self, _error: &str) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn NoOpProgressReporterは何も出力しない() {
        let reporter = NoOpProgressReporter::new();

        // These should not panic and do nothing
        reporter.start_fetching_commits("test-org");
        reporter.report_commits_progress("test-org", 50);
        reporter.finish_fetching_commits("test-org", 100);
        reporter.report_error("test error");
    }

    #[test]
    #[allow(non_snake_case)]
    fn StdoutProgressReporterを作成できる() {
        let reporter = StdoutProgressReporter::new();

        // Basic smoke test - these will output to stderr
        reporter.start_fetching_commits("test-org");
        reporter.report_commits_progress("test-org", 50);
        reporter.finish_fetching_commits("test-org", 100);
    }
}
