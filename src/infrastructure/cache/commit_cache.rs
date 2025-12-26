use crate::domain::entities::commit::Commit;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Trait for caching commits
pub trait CommitCache {
    /// Gets cached commits for the specified parameters
    ///
    /// # Returns
    ///
    /// `Some(commits)` if cache hit, `None` if cache miss
    fn get(&self, org_or_user: &str, from: NaiveDate, to: NaiveDate)
        -> Result<Option<Vec<Commit>>>;

    /// Sets commits in cache
    fn set(
        &self,
        org_or_user: &str,
        from: NaiveDate,
        to: NaiveDate,
        commits: &[Commit],
    ) -> Result<()>;

    /// Clears all cached data
    fn clear(&self) -> Result<()>;
}

/// No-op cache implementation (does not cache anything)
pub struct NoOpCache;

impl CommitCache for NoOpCache {
    fn get(
        &self,
        _org_or_user: &str,
        _from: NaiveDate,
        _to: NaiveDate,
    ) -> Result<Option<Vec<Commit>>> {
        Ok(None)
    }

    fn set(
        &self,
        _org_or_user: &str,
        _from: NaiveDate,
        _to: NaiveDate,
        _commits: &[Commit],
    ) -> Result<()> {
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        Ok(())
    }
}

/// File-based cache implementation
pub struct FileCache {
    cache_dir: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    org_or_user: String,
    from: NaiveDate,
    to: NaiveDate,
    commits: Vec<Commit>,
}

impl FileCache {
    /// Creates a new FileCache instance
    ///
    /// Cache directory defaults to `~/.cache/nenpo/`
    pub fn new() -> Result<Self> {
        let cache_dir = Self::default_cache_dir()?;
        Self::with_cache_dir(cache_dir)
    }

    /// Creates a new FileCache instance with a custom cache directory
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;
        }

        Ok(Self { cache_dir })
    }

    /// Returns the default cache directory (`~/.cache/nenpo/`)
    fn default_cache_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        Ok(home.join(".cache").join("nenpo"))
    }

    /// Generates a cache file path for the given parameters
    fn cache_file_path(&self, org_or_user: &str, from: NaiveDate, to: NaiveDate) -> PathBuf {
        let filename = format!(
            "{}_{}_{}_{}",
            org_or_user,
            from.format("%Y%m%d"),
            to.format("%Y%m%d"),
            "commits.json"
        );
        self.cache_dir.join(filename)
    }
}

impl CommitCache for FileCache {
    fn get(
        &self,
        org_or_user: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Option<Vec<Commit>>> {
        let cache_file = self.cache_file_path(org_or_user, from, to);

        if !cache_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&cache_file).context("Failed to read cache file")?;

        let entry: CacheEntry =
            serde_json::from_str(&content).context("Failed to deserialize cache entry")?;

        Ok(Some(entry.commits))
    }

    fn set(
        &self,
        org_or_user: &str,
        from: NaiveDate,
        to: NaiveDate,
        commits: &[Commit],
    ) -> Result<()> {
        let cache_file = self.cache_file_path(org_or_user, from, to);

        let entry = CacheEntry {
            org_or_user: org_or_user.to_string(),
            from,
            to,
            commits: commits.to_vec(),
        };

        let json =
            serde_json::to_string_pretty(&entry).context("Failed to serialize cache entry")?;

        fs::write(&cache_file, json).context("Failed to write cache file")?;

        Ok(())
    }

    fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    fs::remove_file(path)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use tempfile::TempDir;

    #[test]
    #[allow(non_snake_case)]
    fn キャッシュが存在しない場合はNoneを返す() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache = FileCache::with_cache_dir(temp_dir.path().to_path_buf())
            .expect("Failed to create cache");

        let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let result = cache
            .get("test-org", from, to)
            .expect("Failed to get cache");
        assert!(result.is_none());
    }

    #[test]
    #[allow(non_snake_case)]
    fn コミットをキャッシュに保存して取得できる() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache = FileCache::with_cache_dir(temp_dir.path().to_path_buf())
            .expect("Failed to create cache");

        let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let commits = vec![
            Commit::new(
                "abc123".to_string(),
                "feat: add feature".to_string(),
                "John Doe".to_string(),
                Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
                "test-org/repo1".to_string(),
            ),
            Commit::new(
                "def456".to_string(),
                "fix: resolve bug".to_string(),
                "Jane Smith".to_string(),
                Utc.with_ymd_and_hms(2024, 1, 16, 14, 20, 0).unwrap(),
                "test-org/repo2".to_string(),
            ),
        ];

        cache
            .set("test-org", from, to, &commits)
            .expect("Failed to set cache");

        let cached = cache
            .get("test-org", from, to)
            .expect("Failed to get cache")
            .expect("Cache should exist");

        assert_eq!(cached.len(), 2);
        assert_eq!(cached[0].sha(), "abc123");
        assert_eq!(cached[1].sha(), "def456");
    }

    #[test]
    #[allow(non_snake_case)]
    fn キャッシュをクリアできる() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache = FileCache::with_cache_dir(temp_dir.path().to_path_buf())
            .expect("Failed to create cache");

        let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let commits = vec![Commit::new(
            "abc123".to_string(),
            "feat: add feature".to_string(),
            "John Doe".to_string(),
            Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            "test-org/repo1".to_string(),
        )];

        cache
            .set("test-org", from, to, &commits)
            .expect("Failed to set cache");

        cache.clear().expect("Failed to clear cache");

        let result = cache
            .get("test-org", from, to)
            .expect("Failed to get cache");
        assert!(result.is_none());
    }

    #[test]
    #[allow(non_snake_case)]
    fn 異なるパラメータで別々にキャッシュされる() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache = FileCache::with_cache_dir(temp_dir.path().to_path_buf())
            .expect("Failed to create cache");

        let from1 = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let to1 = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();

        let from2 = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();
        let to2 = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let commits1 = vec![Commit::new(
            "abc123".to_string(),
            "feat: Q1-Q2".to_string(),
            "John Doe".to_string(),
            Utc.with_ymd_and_hms(2024, 3, 15, 10, 30, 0).unwrap(),
            "test-org/repo1".to_string(),
        )];

        let commits2 = vec![Commit::new(
            "def456".to_string(),
            "feat: Q3-Q4".to_string(),
            "Jane Smith".to_string(),
            Utc.with_ymd_and_hms(2024, 9, 16, 14, 20, 0).unwrap(),
            "test-org/repo2".to_string(),
        )];

        cache
            .set("test-org", from1, to1, &commits1)
            .expect("Failed to set cache 1");
        cache
            .set("test-org", from2, to2, &commits2)
            .expect("Failed to set cache 2");

        let cached1 = cache
            .get("test-org", from1, to1)
            .expect("Failed to get cache 1")
            .expect("Cache 1 should exist");
        let cached2 = cache
            .get("test-org", from2, to2)
            .expect("Failed to get cache 2")
            .expect("Cache 2 should exist");

        assert_eq!(cached1[0].message(), "feat: Q1-Q2");
        assert_eq!(cached2[0].message(), "feat: Q3-Q4");
    }
}
