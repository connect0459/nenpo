use crate::domain::entities::commit::Commit;
use crate::domain::entities::github_activity::GitHubActivity;
use crate::domain::repositories::github_repository::GitHubRepository;
use crate::domain::services::progress_reporter::ProgressReporter;
use crate::infrastructure::cache::{CommitCache, NoOpCache};
use crate::infrastructure::github::retry_handler::{with_retry, RetryConfig};
use crate::infrastructure::github::CommandExecutor;
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GraphQLResponse {
    data: Option<GraphQLData>,
}

#[derive(Debug, Deserialize)]
struct GraphQLData {
    organization: Option<Organization>,
    user: Option<User>,
}

#[derive(Debug, Deserialize)]
struct Organization {
    repositories: RepositoryConnection,
}

#[derive(Debug, Deserialize)]
struct User {
    repositories: RepositoryConnection,
}

#[derive(Debug, Deserialize)]
struct RepositoryConnection {
    nodes: Vec<Repository>,
}

#[derive(Debug, Deserialize)]
struct Repository {
    #[serde(rename = "defaultBranchRef")]
    default_branch_ref: Option<BranchRef>,
    #[serde(rename = "pullRequests")]
    pull_requests: PullRequestConnection,
    issues: IssueConnection,
}

#[derive(Debug, Deserialize)]
struct BranchRef {
    target: Target,
}

#[derive(Debug, Deserialize)]
struct Target {
    history: CommitHistoryConnection,
}

#[derive(Debug, Deserialize)]
struct CommitHistoryConnection {
    #[serde(rename = "totalCount")]
    total_count: u32,
}

#[derive(Debug, Deserialize)]
struct PullRequestConnection {
    #[serde(rename = "totalCount")]
    total_count: u32,
}

#[derive(Debug, Deserialize)]
struct IssueConnection {
    #[serde(rename = "totalCount")]
    total_count: u32,
}

// Structures for commit fetching
#[derive(Debug, Deserialize)]
struct CommitsGraphQLResponse {
    data: Option<CommitsGraphQLData>,
}

#[derive(Debug, Deserialize)]
struct CommitsGraphQLData {
    organization: Option<CommitsOrganization>,
    user: Option<CommitsUser>,
}

#[derive(Debug, Deserialize)]
struct CommitsOrganization {
    repositories: CommitsRepositoryConnection,
}

#[derive(Debug, Deserialize)]
struct CommitsUser {
    repositories: CommitsRepositoryConnection,
}

#[derive(Debug, Deserialize)]
struct CommitsRepositoryConnection {
    nodes: Vec<CommitsRepository>,
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
struct CommitsRepository {
    name: String,
    #[serde(rename = "defaultBranchRef")]
    default_branch_ref: Option<CommitsBranchRef>,
}

#[derive(Debug, Deserialize)]
struct CommitsBranchRef {
    target: CommitsTarget,
}

#[derive(Debug, Deserialize)]
struct CommitsTarget {
    history: CommitHistoryConnectionDetailed,
}

#[derive(Debug, Deserialize)]
struct CommitHistoryConnectionDetailed {
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
    nodes: Vec<CommitNode>,
}

#[derive(Debug, Deserialize)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CommitNode {
    oid: String,
    message: String,
    author: CommitAuthor,
    #[serde(rename = "committedDate")]
    committed_date: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CommitAuthor {
    name: Option<String>,
}

/// GitHub repository implementation using gh command
#[allow(dead_code)] // Phase 2: Will be used when integrated into main application
pub struct GhCommandRepository<E: CommandExecutor, P: ProgressReporter, C: CommitCache> {
    executor: E,
    progress_reporter: P,
    retry_config: RetryConfig,
    cache: Option<C>,
}

impl<E: CommandExecutor, P: ProgressReporter, C: CommitCache> GhCommandRepository<E, P, C> {
    /// Creates a new GhCommandRepository instance with default retry configuration and cache
    #[allow(dead_code)] // Phase 2: Will be used when integrated into main application
    pub fn new(executor: E, progress_reporter: P, cache: C) -> Self {
        Self {
            executor,
            progress_reporter,
            retry_config: RetryConfig::default(),
            cache: Some(cache),
        }
    }

    /// Creates a new GhCommandRepository instance without cache
    #[allow(dead_code)]
    pub fn without_cache(executor: E, progress_reporter: P) -> GhCommandRepository<E, P, NoOpCache> {
        GhCommandRepository {
            executor,
            progress_reporter,
            retry_config: RetryConfig::default(),
            cache: None,
        }
    }

    /// Creates a new GhCommandRepository instance with custom retry configuration
    #[allow(dead_code)]
    pub fn with_retry_config(executor: E, progress_reporter: P, cache: C, retry_config: RetryConfig) -> Self {
        Self {
            executor,
            progress_reporter,
            retry_config,
            cache: Some(cache),
        }
    }

    #[allow(dead_code)] // Phase 2: Will be used when integrated into main application
    fn build_graphql_query(org_or_user: &str, from: NaiveDate, to: NaiveDate) -> String {
        let since = format!("{}T00:00:00Z", from);
        let until = format!("{}T23:59:59Z", to);

        format!(
            r#"
            query {{
                organization(login: "{}") {{
                    repositories(first: 100) {{
                        nodes {{
                            defaultBranchRef {{
                                target {{
                                    ... on Commit {{
                                        history(since: "{}", until: "{}") {{
                                            totalCount
                                        }}
                                    }}
                                }}
                            }}
                            pullRequests(states: [OPEN, CLOSED, MERGED]) {{
                                totalCount
                            }}
                            issues(states: [OPEN, CLOSED]) {{
                                totalCount
                            }}
                        }}
                    }}
                }}
                user(login: "{}") {{
                    repositories(first: 100, ownerAffiliations: OWNER) {{
                        nodes {{
                            defaultBranchRef {{
                                target {{
                                    ... on Commit {{
                                        history(since: "{}", until: "{}") {{
                                            totalCount
                                        }}
                                    }}
                                }}
                            }}
                            pullRequests(states: [OPEN, CLOSED, MERGED]) {{
                                totalCount
                            }}
                            issues(states: [OPEN, CLOSED]) {{
                                totalCount
                            }}
                        }}
                    }}
                }}
            }}
            "#,
            org_or_user, since, until, org_or_user, since, until
        )
    }

    /// Builds a GraphQL query for fetching commits with pagination
    #[allow(dead_code)]
    fn build_commits_query(
        org_or_user: &str,
        from: NaiveDate,
        to: NaiveDate,
        after_cursor: Option<&str>,
    ) -> String {
        let since = format!("{}T00:00:00Z", from);
        let until = format!("{}T23:59:59Z", to);
        let after_param = after_cursor
            .map(|c| format!(", after: \"{}\"", c))
            .unwrap_or_default();

        format!(
            r#"
            query {{
                organization(login: "{}") {{
                    repositories(first: 100{}) {{
                        pageInfo {{
                            hasNextPage
                            endCursor
                        }}
                        nodes {{
                            name
                            defaultBranchRef {{
                                target {{
                                    ... on Commit {{
                                        history(first: 100, since: "{}", until: "{}") {{
                                            pageInfo {{
                                                hasNextPage
                                                endCursor
                                            }}
                                            nodes {{
                                                oid
                                                message
                                                author {{
                                                    name
                                                }}
                                                committedDate
                                            }}
                                        }}
                                    }}
                                }}
                            }}
                        }}
                    }}
                }}
                user(login: "{}") {{
                    repositories(first: 100, ownerAffiliations: OWNER{}) {{
                        pageInfo {{
                            hasNextPage
                            endCursor
                        }}
                        nodes {{
                            name
                            defaultBranchRef {{
                                target {{
                                    ... on Commit {{
                                        history(first: 100, since: "{}", until: "{}") {{
                                            pageInfo {{
                                                hasNextPage
                                                endCursor
                                            }}
                                            nodes {{
                                                oid
                                                message
                                                author {{
                                                    name
                                                }}
                                                committedDate
                                            }}
                                        }}
                                    }}
                                }}
                            }}
                        }}
                    }}
                }}
            }}
            "#,
            org_or_user, after_param, since, until, org_or_user, after_param, since, until
        )
    }

    /// Parses commits GraphQL response
    #[allow(dead_code)]
    fn parse_commits_response(response: &str, org_or_user: &str) -> Result<Vec<Commit>> {
        let graphql_response: CommitsGraphQLResponse =
            serde_json::from_str(response).context("Failed to parse commits GraphQL response")?;

        let data = graphql_response
            .data
            .context("No data in commits GraphQL response")?;

        let repositories = if let Some(org) = data.organization {
            org.repositories.nodes
        } else if let Some(user) = data.user {
            user.repositories.nodes
        } else {
            anyhow::bail!("Neither organization nor user found in commits response");
        };

        let mut commits = Vec::new();

        for repo in repositories {
            let repo_name = repo.name;
            if let Some(branch_ref) = repo.default_branch_ref {
                for commit_node in branch_ref.target.history.nodes {
                    let commit = Commit::new(
                        commit_node.oid,
                        commit_node.message,
                        commit_node.author.name.unwrap_or_else(|| "Unknown".to_string()),
                        commit_node.committed_date,
                        format!("{}/{}", org_or_user, repo_name),
                    );
                    commits.push(commit);
                }
            }
        }

        Ok(commits)
    }

    #[allow(dead_code)] // Used in tests
    fn parse_response(response: &str) -> Result<GitHubActivity> {
        let graphql_response: GraphQLResponse =
            serde_json::from_str(response).context("Failed to parse GraphQL response")?;

        let data = graphql_response
            .data
            .context("No data in GraphQL response")?;

        let repositories = if let Some(org) = data.organization {
            org.repositories.nodes
        } else if let Some(user) = data.user {
            user.repositories.nodes
        } else {
            anyhow::bail!("Neither organization nor user found in response");
        };

        let mut total_commits = 0;
        let mut total_prs = 0;
        let mut total_issues = 0;

        for repo in repositories {
            if let Some(branch_ref) = repo.default_branch_ref {
                total_commits += branch_ref.target.history.total_count;
            }
            total_prs += repo.pull_requests.total_count;
            total_issues += repo.issues.total_count;
        }

        // Phase 2: Reviews count is not yet implemented
        // TODO: Add reviews count in future iteration
        Ok(GitHubActivity::new(
            total_commits,
            total_prs,
            total_issues,
            0,
        ))
    }
}

impl<E: CommandExecutor, P: ProgressReporter, C: CommitCache> GitHubRepository for GhCommandRepository<E, P, C> {
    fn fetch_activity(
        &self,
        org_or_user: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<GitHubActivity> {
        let query = Self::build_graphql_query(org_or_user, from, to);
        let response = self
            .executor
            .execute("gh", &["api", "graphql", "-f", &format!("query={}", query)])
            .context("Failed to execute gh command")?;

        Self::parse_response(&response)
    }

    fn fetch_commits(
        &self,
        org_or_user: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Commit>> {
        // Check cache first
        if let Some(ref cache) = self.cache {
            if let Some(cached_commits) = cache.get(org_or_user, from, to)? {
                eprintln!("✓ Using cached commits for {} ({} commits)", org_or_user, cached_commits.len());
                return Ok(cached_commits);
            }
        }

        self.progress_reporter.start_fetching_commits(org_or_user);

        let mut all_commits = Vec::new();
        let mut after_cursor: Option<String> = None;

        // Pagination loop: fetch all commits by following hasNextPage
        loop {
            let query = Self::build_commits_query(
                org_or_user,
                from,
                to,
                after_cursor.as_deref(),
            );

            // Execute with retry
            let response = with_retry(&self.retry_config, || {
                self.executor
                    .execute("gh", &["api", "graphql", "-f", &format!("query={}", query)])
                    .context("Failed to execute gh command for commits")
            })?;

            let commits = Self::parse_commits_response(&response, org_or_user)?;
            all_commits.extend(commits);

            // Report progress
            self.progress_reporter
                .report_commits_progress(org_or_user, all_commits.len());

            // Check if there's a next page
            let graphql_response: CommitsGraphQLResponse =
                serde_json::from_str(&response).context("Failed to parse pagination info")?;

            let data = graphql_response
                .data
                .context("No data in pagination response")?;

            let page_info = if let Some(org) = data.organization {
                org.repositories.page_info
            } else if let Some(user) = data.user {
                user.repositories.page_info
            } else {
                anyhow::bail!("Neither organization nor user found in pagination response");
            };

            if page_info.has_next_page {
                after_cursor = page_info.end_cursor;
            } else {
                break;
            }
        }

        self.progress_reporter
            .finish_fetching_commits(org_or_user, all_commits.len());

        // Save to cache
        if let Some(ref cache) = self.cache {
            cache.set(org_or_user, from, to, &all_commits)?;
        }

        Ok(all_commits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::services::progress_reporter::NoOpProgressReporter;
    use crate::infrastructure::cache::NoOpCache;
    use crate::infrastructure::github::command_executor::MockCommandExecutor;
    use chrono::NaiveDate;

    #[test]
    #[allow(non_snake_case)]
    fn GraphQLレスポンスをパースできる() {
        let response = r#"{
            "data": {
                "organization": {
                    "repositories": {
                        "nodes": [
                            {
                                "defaultBranchRef": {
                                    "target": {
                                        "history": {
                                            "totalCount": 100
                                        }
                                    }
                                },
                                "pullRequests": {
                                    "totalCount": 20
                                },
                                "issues": {
                                    "totalCount": 15
                                }
                            },
                            {
                                "defaultBranchRef": {
                                    "target": {
                                        "history": {
                                            "totalCount": 50
                                        }
                                    }
                                },
                                "pullRequests": {
                                    "totalCount": 10
                                },
                                "issues": {
                                    "totalCount": 5
                                }
                            }
                        ]
                    }
                },
                "user": null
            }
        }"#;

        let activity = GhCommandRepository::<MockCommandExecutor, NoOpProgressReporter, NoOpCache>::parse_response(response)
            .expect("Failed to parse");

        assert_eq!(activity.commits(), 150);
        assert_eq!(activity.pull_requests(), 30);
        assert_eq!(activity.issues(), 20);
        assert_eq!(activity.reviews(), 0); // Not yet implemented
    }

    #[test]
    #[allow(non_snake_case)]
    fn GitHub活動データを取得できる() {
        let mock_response = r#"{
            "data": {
                "organization": {
                    "repositories": {
                        "nodes": [
                            {
                                "defaultBranchRef": {
                                    "target": {
                                        "history": {
                                            "totalCount": 100
                                        }
                                    }
                                },
                                "pullRequests": {
                                    "totalCount": 20
                                },
                                "issues": {
                                    "totalCount": 15
                                }
                            }
                        ]
                    }
                },
                "user": null
            }
        }"#;

        let mock =
            MockCommandExecutor::new().with_response("gh api graphql -f query=", mock_response);

        let repository = GhCommandRepository::new(mock, NoOpProgressReporter::new(), NoOpCache);
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2024, 12, 31).expect("Invalid date");

        let activity = repository
            .fetch_activity("test-org", from, to)
            .expect("Failed to fetch activity");

        assert_eq!(activity.commits(), 100);
        assert_eq!(activity.pull_requests(), 20);
        assert_eq!(activity.issues(), 15);
        assert_eq!(activity.reviews(), 0);
    }

    #[test]
    #[allow(non_snake_case)]
    fn コミットデータをパースできる() {
        let response = r#"{
            "data": {
                "organization": {
                    "repositories": {
                        "pageInfo": {
                            "hasNextPage": false,
                            "endCursor": null
                        },
                        "nodes": [
                            {
                                "name": "test-repo",
                                "defaultBranchRef": {
                                    "target": {
                                        "history": {
                                            "pageInfo": {
                                                "hasNextPage": false,
                                                "endCursor": null
                                            },
                                            "nodes": [
                                                {
                                                    "oid": "abc123",
                                                    "message": "feat: add new feature",
                                                    "author": {
                                                        "name": "John Doe"
                                                    },
                                                    "committedDate": "2024-01-15T10:30:00Z"
                                                },
                                                {
                                                    "oid": "def456",
                                                    "message": "fix: resolve bug",
                                                    "author": {
                                                        "name": "Jane Smith"
                                                    },
                                                    "committedDate": "2024-01-16T14:20:00Z"
                                                }
                                            ]
                                        }
                                    }
                                }
                            }
                        ]
                    }
                },
                "user": null
            }
        }"#;

        let commits = GhCommandRepository::<MockCommandExecutor, NoOpProgressReporter, NoOpCache>::parse_commits_response(response, "test-org")
            .expect("Failed to parse commits");

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].sha(), "abc123");
        assert_eq!(commits[0].message(), "feat: add new feature");
        assert_eq!(commits[0].author(), "John Doe");
        assert_eq!(commits[0].repository(), "test-org/test-repo");

        assert_eq!(commits[1].sha(), "def456");
        assert_eq!(commits[1].message(), "fix: resolve bug");
        assert_eq!(commits[1].author(), "Jane Smith");
    }

    #[test]
    #[allow(non_snake_case)]
    fn コミットを取得できる() {
        let mock_response = r#"{
            "data": {
                "organization": {
                    "repositories": {
                        "pageInfo": {
                            "hasNextPage": false,
                            "endCursor": null
                        },
                        "nodes": [
                            {
                                "name": "test-repo",
                                "defaultBranchRef": {
                                    "target": {
                                        "history": {
                                            "pageInfo": {
                                                "hasNextPage": false,
                                                "endCursor": null
                                            },
                                            "nodes": [
                                                {
                                                    "oid": "abc123",
                                                    "message": "feat: add new feature",
                                                    "author": {
                                                        "name": "John Doe"
                                                    },
                                                    "committedDate": "2024-01-15T10:30:00Z"
                                                }
                                            ]
                                        }
                                    }
                                }
                            }
                        ]
                    }
                },
                "user": null
            }
        }"#;

        let mock = MockCommandExecutor::new()
            .with_response("gh api graphql -f query=", mock_response);

        let repository = GhCommandRepository::new(mock, NoOpProgressReporter::new(), NoOpCache);
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2024, 12, 31).expect("Invalid date");

        let commits = repository
            .fetch_commits("test-org", from, to)
            .expect("Failed to fetch commits");

        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].sha(), "abc123");
        assert_eq!(commits[0].message(), "feat: add new feature");
        assert_eq!(commits[0].author(), "John Doe");
    }

    #[test]
    #[allow(non_snake_case)]
    fn ページネーションでコミットを取得できる() {
        let first_response = r#"{
            "data": {
                "organization": {
                    "repositories": {
                        "pageInfo": {
                            "hasNextPage": true,
                            "endCursor": "cursor123"
                        },
                        "nodes": [
                            {
                                "name": "test-repo",
                                "defaultBranchRef": {
                                    "target": {
                                        "history": {
                                            "pageInfo": {
                                                "hasNextPage": false,
                                                "endCursor": null
                                            },
                                            "nodes": [
                                                {
                                                    "oid": "abc123",
                                                    "message": "feat: first commit",
                                                    "author": {
                                                        "name": "John Doe"
                                                    },
                                                    "committedDate": "2024-01-15T10:30:00Z"
                                                }
                                            ]
                                        }
                                    }
                                }
                            }
                        ]
                    }
                },
                "user": null
            }
        }"#;

        let second_response = r#"{
            "data": {
                "organization": {
                    "repositories": {
                        "pageInfo": {
                            "hasNextPage": false,
                            "endCursor": null
                        },
                        "nodes": [
                            {
                                "name": "test-repo-2",
                                "defaultBranchRef": {
                                    "target": {
                                        "history": {
                                            "pageInfo": {
                                                "hasNextPage": false,
                                                "endCursor": null
                                            },
                                            "nodes": [
                                                {
                                                    "oid": "def456",
                                                    "message": "fix: second commit",
                                                    "author": {
                                                        "name": "Jane Smith"
                                                    },
                                                    "committedDate": "2024-01-16T14:20:00Z"
                                                }
                                            ]
                                        }
                                    }
                                }
                            }
                        ]
                    }
                },
                "user": null
            }
        }"#;

        let mock = MockCommandExecutor::new()
            .with_response("gh api graphql -f query=", first_response)
            .with_response("gh api graphql -f query=", second_response);

        let repository = GhCommandRepository::new(mock, NoOpProgressReporter::new(), NoOpCache);
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2024, 12, 31).expect("Invalid date");

        let commits = repository
            .fetch_commits("test-org", from, to)
            .expect("Failed to fetch commits with pagination");

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].sha(), "abc123");
        assert_eq!(commits[0].message(), "feat: first commit");
        assert_eq!(commits[1].sha(), "def456");
        assert_eq!(commits[1].message(), "fix: second commit");
    }
}
