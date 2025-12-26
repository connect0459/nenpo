use crate::domain::entities::config::Config;
use anyhow::Result;
use std::path::Path;

/// Repository trait for loading configuration
#[allow(dead_code)] // Temporarily allowed during TDD implementation
pub trait ConfigRepository {
    /// Loads configuration from the specified path
    fn load(&self, path: &Path) -> Result<Config>;
}
