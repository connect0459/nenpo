use crate::domain::entities::commit::Commit;
use crate::domain::entities::github_activity::GitHubActivity;
use crate::domain::entities::report::Report;
use crate::domain::repositories::config_repository::ConfigRepository;
use crate::domain::repositories::document_repository::DocumentRepository;
use crate::domain::repositories::github_repository::GitHubRepository;
use crate::domain::repositories::output_repository::OutputRepository;
use crate::domain::value_objects::commit_theme::CommitTheme;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::collections::HashMap;
use std::path::Path;

/// Service for generating reports
#[allow(dead_code)]
pub struct ReportGenerator<C, G, D, O>
where
    C: ConfigRepository,
    G: GitHubRepository,
    D: DocumentRepository,
    O: OutputRepository,
{
    config_repository: C,
    github_repository: G,
    document_repository: D,
    output_repository: O,
}

impl<C, G, D, O> ReportGenerator<C, G, D, O>
where
    C: ConfigRepository,
    G: GitHubRepository,
    D: DocumentRepository,
    O: OutputRepository,
{
    /// Creates a new ReportGenerator instance
    #[allow(dead_code)]
    pub fn new(
        config_repository: C,
        github_repository: G,
        document_repository: D,
        output_repository: O,
    ) -> Self {
        Self {
            config_repository,
            github_repository,
            document_repository,
            output_repository,
        }
    }

    /// Generates reports for all departments or a specific department
    #[allow(dead_code)]
    pub fn generate(
        &self,
        config_path: &Path,
        year: Option<u32>,
        department_filter: Option<&str>,
        output_dir: &Path,
        file_extension: &str,
    ) -> Result<Vec<String>> {
        // Load configuration
        let config = self
            .config_repository
            .load(config_path)
            .context("Failed to load configuration")?;

        // Filter departments if specified
        let departments = if let Some(dept_name) = department_filter {
            config
                .departments()
                .iter()
                .filter(|d| d.name() == dept_name)
                .cloned()
                .collect::<Vec<_>>()
        } else {
            config.departments().to_vec()
        };

        if departments.is_empty() {
            anyhow::bail!("No departments found");
        }

        let mut generated_files = Vec::new();

        // Process each department
        for department in departments {
            let fiscal_year = year.unwrap_or(2024); // Default to 2024 if not specified
            let fiscal_start_month = department.fiscal_year_start_month();

            // Calculate period
            let (period_from, period_to) = calculate_fiscal_period(fiscal_year, fiscal_start_month);

            // Fetch GitHub activity
            let mut total_activity = GitHubActivity::new(0, 0, 0, 0);
            for org in department.github_organizations() {
                let activity =
                    self.github_repository
                        .fetch_activity(org, period_from, period_to)?;
                total_activity = total_activity.add(&activity);
            }

            // Fetch documents
            let documents = self
                .document_repository
                .fetch_documents(department.local_documents())?;

            // Fetch commits and build theme summary
            let mut all_commits = Vec::new();
            for org in department.github_organizations() {
                let commits = self
                    .github_repository
                    .fetch_commits(org, period_from, period_to)?;
                all_commits.extend(commits);
            }

            let theme_summary = Self::build_theme_summary(&all_commits);

            let report = Report::new(
                fiscal_year,
                department.name().to_string(),
                period_from,
                period_to,
                total_activity,
                documents,
                theme_summary,
            );

            // Output report
            let output_filename = format!(
                "report-{}-{}.{}",
                department.name(),
                fiscal_year,
                file_extension
            );
            let output_path = output_dir.join(&output_filename);
            self.output_repository.output(&report, &output_path)?;

            generated_files.push(output_filename);
        }

        Ok(generated_files)
    }

    /// Builds a theme summary from commit messages
    fn build_theme_summary(commits: &[Commit]) -> HashMap<CommitTheme, u32> {
        let mut theme_summary = HashMap::new();

        for commit in commits {
            let theme = CommitTheme::from_commit_message(commit.message());
            *theme_summary.entry(theme).or_insert(0) += 1;
        }

        theme_summary
    }
}

/// Calculates the fiscal period for a given year and start month
fn calculate_fiscal_period(year: u32, start_month: u32) -> (NaiveDate, NaiveDate) {
    let from = NaiveDate::from_ymd_opt(year as i32, start_month, 1).expect("Invalid date");

    let (to_year, to_month) = if start_month == 1 {
        (year as i32, 12)
    } else {
        (year as i32 + 1, start_month - 1)
    };

    // Get the last day of the to_month
    let to = if to_month == 12 {
        NaiveDate::from_ymd_opt(to_year, 12, 31).expect("Invalid date")
    } else {
        // Get the first day of the next month and subtract one day
        NaiveDate::from_ymd_opt(to_year, to_month + 1, 1)
            .expect("Invalid date")
            .pred_opt()
            .expect("Invalid date")
    };

    (from, to)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::config::Config;
    use crate::domain::entities::department::Department;
    use crate::domain::entities::document_content::DocumentContent;
    use crate::domain::value_objects::output_format::OutputFormat;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Mock repositories for testing
    struct MockConfigRepository {
        config: Config,
    }

    impl ConfigRepository for MockConfigRepository {
        fn load(&self, _path: &Path) -> Result<Config> {
            Ok(self.config.clone())
        }
    }

    struct MockGitHubRepository {
        responses: HashMap<String, GitHubActivity>,
    }

    impl GitHubRepository for MockGitHubRepository {
        fn fetch_activity(
            &self,
            org_or_user: &str,
            _from: NaiveDate,
            _to: NaiveDate,
        ) -> Result<GitHubActivity> {
            self.responses
                .get(org_or_user)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No mock data for {}", org_or_user))
        }

        fn fetch_commits(
            &self,
            _org_or_user: &str,
            _from: NaiveDate,
            _to: NaiveDate,
        ) -> Result<Vec<Commit>> {
            // Mock implementation returns empty commits for now
            Ok(Vec::new())
        }
    }

    struct MockDocumentRepository {
        documents: Vec<DocumentContent>,
    }

    impl DocumentRepository for MockDocumentRepository {
        fn fetch_documents(&self, _patterns: &[String]) -> Result<Vec<DocumentContent>> {
            Ok(self.documents.clone())
        }
    }

    struct MockOutputRepository {
        outputs: Arc<Mutex<Vec<(String, String)>>>, // (filename, department_name)
    }

    impl OutputRepository for MockOutputRepository {
        fn output(&self, report: &Report, path: &Path) -> Result<()> {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            self.outputs
                .lock()
                .unwrap()
                .push((filename, report.department_name().to_string()));
            Ok(())
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn 単一部門のレポートを生成できる() {
        let dept = Department::new("個人".to_string(), 4, vec!["test-org".to_string()], vec![]);
        let config = Config::new(
            4,
            OutputFormat::Markdown,
            "./reports".to_string(),
            vec![dept],
        );

        let mut github_responses = HashMap::new();
        github_responses.insert("test-org".to_string(), GitHubActivity::new(100, 20, 15, 30));

        let config_repo = MockConfigRepository { config };
        let github_repo = MockGitHubRepository {
            responses: github_responses,
        };
        let document_repo = MockDocumentRepository { documents: vec![] };
        let outputs = Arc::new(Mutex::new(Vec::new()));
        let output_repo = MockOutputRepository {
            outputs: outputs.clone(),
        };

        let generator = ReportGenerator::new(config_repo, github_repo, document_repo, output_repo);

        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let result = generator.generate(
            Path::new("dummy.toml"),
            Some(2024),
            None,
            temp_dir.path(),
            "md",
        );

        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], "report-個人-2024.md");

        let outputs = outputs.lock().unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].1, "個人");
    }

    #[test]
    #[allow(non_snake_case)]
    fn 複数部門のレポートを生成できる() {
        let dept1 = Department::new(
            "個人".to_string(),
            4,
            vec!["personal-org".to_string()],
            vec![],
        );
        let dept2 = Department::new(
            "企業".to_string(),
            4,
            vec!["company-org".to_string()],
            vec![],
        );
        let config = Config::new(
            4,
            OutputFormat::Markdown,
            "./reports".to_string(),
            vec![dept1, dept2],
        );

        let mut github_responses = HashMap::new();
        github_responses.insert(
            "personal-org".to_string(),
            GitHubActivity::new(100, 20, 15, 30),
        );
        github_responses.insert(
            "company-org".to_string(),
            GitHubActivity::new(50, 10, 5, 15),
        );

        let config_repo = MockConfigRepository { config };
        let github_repo = MockGitHubRepository {
            responses: github_responses,
        };
        let document_repo = MockDocumentRepository { documents: vec![] };
        let outputs = Arc::new(Mutex::new(Vec::new()));
        let output_repo = MockOutputRepository {
            outputs: outputs.clone(),
        };

        let generator = ReportGenerator::new(config_repo, github_repo, document_repo, output_repo);

        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let result = generator.generate(
            Path::new("dummy.toml"),
            Some(2024),
            None,
            temp_dir.path(),
            "md",
        );

        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"report-個人-2024.md".to_string()));
        assert!(files.contains(&"report-企業-2024.md".to_string()));

        let outputs = outputs.lock().unwrap();
        assert_eq!(outputs.len(), 2);
    }

    #[test]
    #[allow(non_snake_case)]
    fn 特定部門のみレポートを生成できる() {
        let dept1 = Department::new(
            "個人".to_string(),
            4,
            vec!["personal-org".to_string()],
            vec![],
        );
        let dept2 = Department::new(
            "企業".to_string(),
            4,
            vec!["company-org".to_string()],
            vec![],
        );
        let config = Config::new(
            4,
            OutputFormat::Markdown,
            "./reports".to_string(),
            vec![dept1, dept2],
        );

        let mut github_responses = HashMap::new();
        github_responses.insert(
            "personal-org".to_string(),
            GitHubActivity::new(100, 20, 15, 30),
        );

        let config_repo = MockConfigRepository { config };
        let github_repo = MockGitHubRepository {
            responses: github_responses,
        };
        let document_repo = MockDocumentRepository { documents: vec![] };
        let outputs = Arc::new(Mutex::new(Vec::new()));
        let output_repo = MockOutputRepository {
            outputs: outputs.clone(),
        };

        let generator = ReportGenerator::new(config_repo, github_repo, document_repo, output_repo);

        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let result = generator.generate(
            Path::new("dummy.toml"),
            Some(2024),
            Some("個人"),
            temp_dir.path(),
            "md",
        );

        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], "report-個人-2024.md");

        let outputs = outputs.lock().unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].1, "個人");
    }

    #[test]
    #[allow(non_snake_case)]
    fn 年度期間を正しく計算できる() {
        // Fiscal year starting in April
        let (from, to) = calculate_fiscal_period(2024, 4);
        assert_eq!(from, NaiveDate::from_ymd_opt(2024, 4, 1).unwrap());
        assert_eq!(to, NaiveDate::from_ymd_opt(2025, 3, 31).unwrap());

        // Calendar year
        let (from, to) = calculate_fiscal_period(2024, 1);
        assert_eq!(from, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(to, NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());
    }

    #[test]
    #[allow(non_snake_case)]
    fn コミットメッセージからテーマ別要約を構築できる() {
        use chrono::{TimeZone, Utc};

        let commits = vec![
            Commit::new(
                "abc123".to_string(),
                "feat: add new feature".to_string(),
                "John Doe".to_string(),
                Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
                "test-org/repo1".to_string(),
            ),
            Commit::new(
                "def456".to_string(),
                "fix: resolve bug".to_string(),
                "Jane Smith".to_string(),
                Utc.with_ymd_and_hms(2024, 1, 16, 14, 20, 0).unwrap(),
                "test-org/repo1".to_string(),
            ),
            Commit::new(
                "ghi789".to_string(),
                "feat: add another feature".to_string(),
                "Bob Johnson".to_string(),
                Utc.with_ymd_and_hms(2024, 1, 17, 9, 15, 0).unwrap(),
                "test-org/repo2".to_string(),
            ),
            Commit::new(
                "jkl012".to_string(),
                "docs: update README".to_string(),
                "Alice Williams".to_string(),
                Utc.with_ymd_and_hms(2024, 1, 18, 11, 45, 0).unwrap(),
                "test-org/repo2".to_string(),
            ),
        ];

        let theme_summary = ReportGenerator::<
            MockConfigRepository,
            MockGitHubRepository,
            MockDocumentRepository,
            MockOutputRepository,
        >::build_theme_summary(&commits);

        assert_eq!(theme_summary.get(&CommitTheme::Feat), Some(&2));
        assert_eq!(theme_summary.get(&CommitTheme::Fix), Some(&1));
        assert_eq!(theme_summary.get(&CommitTheme::Docs), Some(&1));
        assert_eq!(theme_summary.get(&CommitTheme::Refactor), None);
    }
}
