use crate::domain::entities::report::Report;
use anyhow::Result;
use std::path::Path;

/// Repository trait for outputting reports
#[allow(dead_code)] // Temporarily allowed during TDD implementation
pub trait OutputRepository {
    /// Outputs a report to the specified path
    fn output(&self, report: &Report, path: &Path) -> Result<()>;
}
