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

// Structures for commit fetching (multi-repo query)
#[derive(Debug, Deserialize)]
struct CommitsGraphQLResponse {
    data: Option<CommitsGraphQLData>,
}

#[derive(Debug, Deserialize)]
struct CommitsGraphQLData {
    organization: Option<CommitsOrganization>,
    user: Option<CommitsUser>,
}

// Structures for repository list fetching
#[derive(Debug, Deserialize)]
struct RepositoriesGraphQLResponse {
    data: Option<RepositoriesGraphQLData>,
}

#[derive(Debug, Deserialize)]
struct RepositoriesGraphQLData {
    organization: Option<RepositoriesOrganization>,
    user: Option<RepositoriesUser>,
}

#[derive(Debug, Deserialize)]
struct RepositoriesOrganization {
    repositories: RepositoriesConnection,
}

#[derive(Debug, Deserialize)]
struct RepositoriesUser {
    repositories: RepositoriesConnection,
}

#[derive(Debug, Deserialize)]
struct RepositoriesConnection {
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
    nodes: Vec<RepositoryNode>,
}

#[derive(Debug, Deserialize)]
struct RepositoryNode {
    name: String,
}

// Structures for single repository commit fetching
#[derive(Debug, Deserialize)]
struct SingleRepoCommitsGraphQLResponse {
    data: Option<SingleRepoCommitsGraphQLData>,
}

#[derive(Debug, Deserialize)]
struct SingleRepoCommitsGraphQLData {
    organization: Option<SingleRepoOrganization>,
    user: Option<SingleRepoUser>,
}

#[derive(Debug, Deserialize)]
struct SingleRepoOrganization {
    repository: Option<SingleRepoRepository>,
}

#[derive(Debug, Deserialize)]
struct SingleRepoUser {
    repository: Option<SingleRepoRepository>,
}

#[derive(Debug, Deserialize)]
struct SingleRepoRepository {
    #[serde(rename = "defaultBranchRef")]
    default_branch_ref: Option<CommitsBranchRef>,
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
    #[allow(dead_code)] // Used by old multi-repo query, kept for backwards compatibility
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

#[derive(Debug, Deserialize, Clone)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserIdGraphQLResponse {
    data: Option<UserIdGraphQLData>,
}

#[derive(Debug, Deserialize)]
struct UserIdGraphQLData {
    user: Option<UserIdUser>,
}

#[derive(Debug, Deserialize)]
struct UserIdUser {
    id: String,
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
    pub fn without_cache(
        executor: E,
        progress_reporter: P,
    ) -> GhCommandRepository<E, P, NoOpCache> {
        GhCommandRepository {
            executor,
            progress_reporter,
            retry_config: RetryConfig::default(),
            cache: None,
        }
    }

    /// Creates a new GhCommandRepository instance with custom retry configuration
    #[allow(dead_code)]
    pub fn with_retry_config(
        executor: E,
        progress_reporter: P,
        cache: C,
        retry_config: RetryConfig,
    ) -> Self {
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

    /// Fetches GitHub user ID from login name
    fn fetch_user_id(&self, login: &str) -> Result<String> {
        let query = format!(
            r#"
            query {{
                user(login: "{}") {{
                    id
                }}
            }}
            "#,
            login
        );

        let response = with_retry(&self.retry_config, || {
            self.executor
                .execute("gh", &["api", "graphql", "-f", &format!("query={}", query)])
                .context("Failed to execute gh command for user ID")
        })?;

        let graphql_response: UserIdGraphQLResponse =
            serde_json::from_str(&response).context("Failed to parse user ID GraphQL response")?;

        let data = graphql_response
            .data
            .context("No data in user ID response")?;

        let user = data.user.context("User not found")?;

        Ok(user.id)
    }

    /// Builds a GraphQL query for fetching commits with pagination
    #[allow(dead_code)]
    fn build_commits_query(
        org_or_user: &str,
        from: NaiveDate,
        to: NaiveDate,
        after_cursor: Option<&str>,
        author_id: Option<&str>,
    ) -> String {
        let since = format!("{}T00:00:00Z", from);
        let until = format!("{}T23:59:59Z", to);
        let after_param = after_cursor
            .map(|c| format!(", after: \"{}\"", c))
            .unwrap_or_default();
        let author_param = author_id
            .map(|id| format!(", author: {{id: \"{}\"}}", id))
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
                                        history(first: 100, since: "{}", until: "{}"{}) {{
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
                                        history(first: 100, since: "{}", until: "{}"{}) {{
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
            org_or_user,
            after_param,
            since,
            until,
            author_param,
            org_or_user,
            after_param,
            since,
            until,
            author_param
        )
    }

    /// Builds a GraphQL query for fetching repository list with pagination
    /// This is used for the outer pagination loop to get all repositories
    #[allow(dead_code)]
    fn build_repositories_query(org_or_user: &str, after_cursor: Option<&str>) -> String {
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
                        }}
                    }}
                }}
            }}
            "#,
            org_or_user, after_param, org_or_user, after_param
        )
    }

    /// Builds a GraphQL query for fetching commits from a single repository with pagination
    /// This is used for the inner pagination loop to fetch all commits within a repository
    #[allow(dead_code)]
    fn build_repo_commits_query(
        org_or_user: &str,
        repo_name: &str,
        from: NaiveDate,
        to: NaiveDate,
        author_id: Option<&str>,
        after_cursor: Option<&str>,
    ) -> String {
        let since = format!("{}T00:00:00Z", from);
        let until = format!("{}T23:59:59Z", to);
        let author_param = author_id
            .map(|id| format!(", author: {{id: \"{}\"}}", id))
            .unwrap_or_default();
        let after_param = after_cursor
            .map(|c| format!(", after: \"{}\"", c))
            .unwrap_or_default();

        format!(
            r#"
            query {{
                organization(login: "{}") {{
                    repository(name: "{}") {{
                        defaultBranchRef {{
                            target {{
                                ... on Commit {{
                                    history(first: 100, since: "{}", until: "{}"{}{}) {{
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
                user(login: "{}") {{
                    repository(name: "{}") {{
                        defaultBranchRef {{
                            target {{
                                ... on Commit {{
                                    history(first: 100, since: "{}", until: "{}"{}{}) {{
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
            "#,
            org_or_user,
            repo_name,
            since,
            until,
            author_param,
            after_param,
            org_or_user,
            repo_name,
            since,
            until,
            author_param,
            after_param
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
                        commit_node
                            .author
                            .name
                            .unwrap_or_else(|| "Unknown".to_string()),
                        commit_node.committed_date,
                        format!("{}/{}", org_or_user, repo_name),
                    );
                    commits.push(commit);
                }
            }
        }

        Ok(commits)
    }

    /// Parses repositories GraphQL response
    /// Returns repository names and pagination info
    #[allow(dead_code)]
    fn parse_repositories_response(response: &str) -> Result<(Vec<String>, PageInfo)> {
        let graphql_response: RepositoriesGraphQLResponse = serde_json::from_str(response)
            .context("Failed to parse repositories GraphQL response")?;

        let data = graphql_response
            .data
            .context("No data in repositories GraphQL response")?;

        let repositories = if let Some(org) = data.organization {
            org.repositories
        } else if let Some(user) = data.user {
            user.repositories
        } else {
            anyhow::bail!("Neither organization nor user found in repositories response");
        };

        let repo_names: Vec<String> = repositories
            .nodes
            .into_iter()
            .map(|node| node.name)
            .collect();

        Ok((repo_names, repositories.page_info))
    }

    /// Parses single repository commits GraphQL response
    /// Returns commits and pagination info
    #[allow(dead_code)]
    fn parse_repo_commits_response(
        response: &str,
        org_or_user: &str,
        repo_name: &str,
    ) -> Result<(Vec<Commit>, PageInfo)> {
        let graphql_response: SingleRepoCommitsGraphQLResponse = serde_json::from_str(response)
            .context("Failed to parse single repository commits GraphQL response")?;

        let data = graphql_response
            .data
            .context("No data in single repository commits GraphQL response")?;

        let repository = if let Some(org) = data.organization {
            org.repository
        } else if let Some(user) = data.user {
            user.repository
        } else {
            anyhow::bail!(
                "Neither organization nor user found in single repository commits response"
            );
        };

        let repository = repository.context(format!(
            "Repository {} not found for {}",
            repo_name, org_or_user
        ))?;

        // If there's no default branch, return empty commits (e.g., empty repository)
        let Some(branch_ref) = repository.default_branch_ref else {
            eprintln!(
                "⚠ Skipping {}/{}: No default branch (possibly empty repository)",
                org_or_user, repo_name
            );
            return Ok((
                Vec::new(),
                PageInfo {
                    has_next_page: false,
                    end_cursor: None,
                },
            ));
        };

        let history = branch_ref.target.history;
        let page_info = history.page_info.clone();

        let commits: Vec<Commit> = history
            .nodes
            .into_iter()
            .map(|commit_node| {
                Commit::new(
                    commit_node.oid,
                    commit_node.message,
                    commit_node
                        .author
                        .name
                        .unwrap_or_else(|| "Unknown".to_string()),
                    commit_node.committed_date,
                    format!("{}/{}", org_or_user, repo_name),
                )
            })
            .collect();

        Ok((commits, page_info))
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

impl<E: CommandExecutor, P: ProgressReporter, C: CommitCache> GitHubRepository
    for GhCommandRepository<E, P, C>
{
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
        author: Option<&str>,
    ) -> Result<Vec<Commit>> {
        // Fetch author ID if author is specified
        let author_id = if let Some(author_login) = author {
            Some(self.fetch_user_id(author_login)?)
        } else {
            None
        };

        // Check cache first
        if let Some(ref cache) = self.cache {
            if let Some(cached_commits) = cache.get(org_or_user, from, to, author)? {
                eprintln!(
                    "✓ Using cached commits for {} ({} commits)",
                    org_or_user,
                    cached_commits.len()
                );
                return Ok(cached_commits);
            }
        }

        self.progress_reporter.start_fetching_commits(org_or_user);

        let mut all_commits = Vec::new();
        let mut repo_cursor: Option<String> = None;

        // Outer loop: Repository pagination
        loop {
            let repos_query = Self::build_repositories_query(org_or_user, repo_cursor.as_deref());

            // Execute with retry
            let repos_response = with_retry(&self.retry_config, || {
                self.executor
                    .execute(
                        "gh",
                        &["api", "graphql", "-f", &format!("query={}", repos_query)],
                    )
                    .context("Failed to execute gh command for repositories")
            })?;

            let (repo_names, repos_page_info) = Self::parse_repositories_response(&repos_response)?;

            // Inner loop: Fetch commits for each repository
            for repo_name in repo_names {
                let mut commit_cursor: Option<String> = None;

                // Pagination within a single repository
                loop {
                    let commits_query = Self::build_repo_commits_query(
                        org_or_user,
                        &repo_name,
                        from,
                        to,
                        author_id.as_deref(),
                        commit_cursor.as_deref(),
                    );

                    // Execute with retry
                    let commits_response = with_retry(&self.retry_config, || {
                        self.executor
                            .execute(
                                "gh",
                                &["api", "graphql", "-f", &format!("query={}", commits_query)],
                            )
                            .context("Failed to execute gh command for commits")
                    })?;

                    let (commits, commits_page_info) = Self::parse_repo_commits_response(
                        &commits_response,
                        org_or_user,
                        &repo_name,
                    )?;

                    all_commits.extend(commits);

                    // Report progress
                    self.progress_reporter
                        .report_commits_progress(org_or_user, all_commits.len());

                    if commits_page_info.has_next_page {
                        commit_cursor = commits_page_info.end_cursor;
                    } else {
                        break;
                    }
                }
            }

            // Check if there's a next page of repositories
            if repos_page_info.has_next_page {
                repo_cursor = repos_page_info.end_cursor;
            } else {
                break;
            }
        }

        self.progress_reporter
            .finish_fetching_commits(org_or_user, all_commits.len());

        // Save to cache
        if let Some(ref cache) = self.cache {
            cache.set(org_or_user, from, to, author, &all_commits)?;
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
    fn parses_graphql_response() {
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
    fn fetches_github_activity_data() {
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
    fn parses_commit_data() {
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
    fn fetches_commits() {
        // First response: repository list
        let repos_response = r#"{
            "data": {
                "organization": {
                    "repositories": {
                        "pageInfo": {
                            "hasNextPage": false,
                            "endCursor": null
                        },
                        "nodes": [
                            {
                                "name": "test-repo"
                            }
                        ]
                    }
                },
                "user": null
            }
        }"#;

        // Second response: commits for test-repo
        let commits_response = r#"{
            "data": {
                "organization": {
                    "repository": {
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
                },
                "user": null
            }
        }"#;

        let mock = MockCommandExecutor::new()
            .with_response("gh api graphql -f query=", repos_response)
            .with_response("gh api graphql -f query=", commits_response);

        let repository = GhCommandRepository::new(mock, NoOpProgressReporter::new(), NoOpCache);
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2024, 12, 31).expect("Invalid date");

        let commits = repository
            .fetch_commits("test-org", from, to, None)
            .expect("Failed to fetch commits");

        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].sha(), "abc123");
        assert_eq!(commits[0].message(), "feat: add new feature");
        assert_eq!(commits[0].author(), "John Doe");
    }

    #[test]
    fn fetches_commits_with_pagination() {
        // First response: repository list (page 1)
        let repos_page1_response = r#"{
            "data": {
                "organization": {
                    "repositories": {
                        "pageInfo": {
                            "hasNextPage": true,
                            "endCursor": "cursor123"
                        },
                        "nodes": [
                            {
                                "name": "test-repo"
                            }
                        ]
                    }
                },
                "user": null
            }
        }"#;

        // Second response: commits for test-repo
        let test_repo_commits_response = r#"{
            "data": {
                "organization": {
                    "repository": {
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
                },
                "user": null
            }
        }"#;

        // Third response: repository list (page 2)
        let repos_page2_response = r#"{
            "data": {
                "organization": {
                    "repositories": {
                        "pageInfo": {
                            "hasNextPage": false,
                            "endCursor": null
                        },
                        "nodes": [
                            {
                                "name": "test-repo-2"
                            }
                        ]
                    }
                },
                "user": null
            }
        }"#;

        // Fourth response: commits for test-repo-2
        let test_repo_2_commits_response = r#"{
            "data": {
                "organization": {
                    "repository": {
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
                },
                "user": null
            }
        }"#;

        let mock = MockCommandExecutor::new()
            .with_response("gh api graphql -f query=", repos_page1_response)
            .with_response("gh api graphql -f query=", test_repo_commits_response)
            .with_response("gh api graphql -f query=", repos_page2_response)
            .with_response("gh api graphql -f query=", test_repo_2_commits_response);

        let repository = GhCommandRepository::new(mock, NoOpProgressReporter::new(), NoOpCache);
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2024, 12, 31).expect("Invalid date");

        let commits = repository
            .fetch_commits("test-org", from, to, None)
            .expect("Failed to fetch commits with pagination");

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].sha(), "abc123");
        assert_eq!(commits[0].message(), "feat: first commit");
        assert_eq!(commits[1].sha(), "def456");
        assert_eq!(commits[1].message(), "fix: second commit");
    }

    #[test]
    fn tests_pagination_for_commits_in_repository() {
        // First response: repository list
        let repos_response = r#"{
            "data": {
                "organization": {
                    "repositories": {
                        "pageInfo": {
                            "hasNextPage": false,
                            "endCursor": null
                        },
                        "nodes": [
                            {
                                "name": "large-repo"
                            }
                        ]
                    }
                },
                "user": null
            }
        }"#;

        // Second response: commits page 1 (100 commits)
        let commits_page1_response = r#"{
            "data": {
                "organization": {
                    "repository": {
                        "defaultBranchRef": {
                            "target": {
                                "history": {
                                    "pageInfo": {
                                        "hasNextPage": true,
                                        "endCursor": "commit_cursor_100"
                                    },
                                    "nodes": [
                                        {
                                            "oid": "commit001",
                                            "message": "feat: commit 1",
                                            "author": {
                                                "name": "Developer 1"
                                            },
                                            "committedDate": "2024-01-01T10:00:00Z"
                                        }
                                    ]
                                }
                            }
                        }
                    }
                },
                "user": null
            }
        }"#;

        // Third response: commits page 2 (remaining commits)
        let commits_page2_response = r#"{
            "data": {
                "organization": {
                    "repository": {
                        "defaultBranchRef": {
                            "target": {
                                "history": {
                                    "pageInfo": {
                                        "hasNextPage": false,
                                        "endCursor": null
                                    },
                                    "nodes": [
                                        {
                                            "oid": "commit101",
                                            "message": "fix: commit 101",
                                            "author": {
                                                "name": "Developer 2"
                                            },
                                            "committedDate": "2024-02-01T10:00:00Z"
                                        }
                                    ]
                                }
                            }
                        }
                    }
                },
                "user": null
            }
        }"#;

        let mock = MockCommandExecutor::new()
            .with_response("gh api graphql -f query=", repos_response)
            .with_response("gh api graphql -f query=", commits_page1_response)
            .with_response("gh api graphql -f query=", commits_page2_response);

        let repository = GhCommandRepository::new(mock, NoOpProgressReporter::new(), NoOpCache);
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2024, 12, 31).expect("Invalid date");

        let commits = repository
            .fetch_commits("test-org", from, to, None)
            .expect("Failed to fetch commits with pagination within repository");

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].sha(), "commit001");
        assert_eq!(commits[0].message(), "feat: commit 1");
        assert_eq!(commits[1].sha(), "commit101");
        assert_eq!(commits[1].message(), "fix: commit 101");
    }

    #[test]
    fn tests_nested_pagination() {
        // Test both repository pagination and commit pagination within each repository
        // First response: repository list page 1
        let repos_page1_response = r#"{
            "data": {
                "organization": {
                    "repositories": {
                        "pageInfo": {
                            "hasNextPage": true,
                            "endCursor": "repo_cursor_1"
                        },
                        "nodes": [
                            {
                                "name": "repo-1"
                            }
                        ]
                    }
                },
                "user": null
            }
        }"#;

        // Second response: repo-1 commits page 1
        let repo1_commits_page1_response = r#"{
            "data": {
                "organization": {
                    "repository": {
                        "defaultBranchRef": {
                            "target": {
                                "history": {
                                    "pageInfo": {
                                        "hasNextPage": true,
                                        "endCursor": "commit_cursor_1"
                                    },
                                    "nodes": [
                                        {
                                            "oid": "repo1_commit1",
                                            "message": "feat: repo1 first",
                                            "author": {
                                                "name": "Dev A"
                                            },
                                            "committedDate": "2024-01-01T10:00:00Z"
                                        }
                                    ]
                                }
                            }
                        }
                    }
                },
                "user": null
            }
        }"#;

        // Third response: repo-1 commits page 2
        let repo1_commits_page2_response = r#"{
            "data": {
                "organization": {
                    "repository": {
                        "defaultBranchRef": {
                            "target": {
                                "history": {
                                    "pageInfo": {
                                        "hasNextPage": false,
                                        "endCursor": null
                                    },
                                    "nodes": [
                                        {
                                            "oid": "repo1_commit2",
                                            "message": "fix: repo1 second",
                                            "author": {
                                                "name": "Dev A"
                                            },
                                            "committedDate": "2024-01-02T10:00:00Z"
                                        }
                                    ]
                                }
                            }
                        }
                    }
                },
                "user": null
            }
        }"#;

        // Fourth response: repository list page 2
        let repos_page2_response = r#"{
            "data": {
                "organization": {
                    "repositories": {
                        "pageInfo": {
                            "hasNextPage": false,
                            "endCursor": null
                        },
                        "nodes": [
                            {
                                "name": "repo-2"
                            }
                        ]
                    }
                },
                "user": null
            }
        }"#;

        // Fifth response: repo-2 commits
        let repo2_commits_response = r#"{
            "data": {
                "organization": {
                    "repository": {
                        "defaultBranchRef": {
                            "target": {
                                "history": {
                                    "pageInfo": {
                                        "hasNextPage": false,
                                        "endCursor": null
                                    },
                                    "nodes": [
                                        {
                                            "oid": "repo2_commit1",
                                            "message": "chore: repo2 commit",
                                            "author": {
                                                "name": "Dev B"
                                            },
                                            "committedDate": "2024-01-03T10:00:00Z"
                                        }
                                    ]
                                }
                            }
                        }
                    }
                },
                "user": null
            }
        }"#;

        let mock = MockCommandExecutor::new()
            .with_response("gh api graphql -f query=", repos_page1_response)
            .with_response("gh api graphql -f query=", repo1_commits_page1_response)
            .with_response("gh api graphql -f query=", repo1_commits_page2_response)
            .with_response("gh api graphql -f query=", repos_page2_response)
            .with_response("gh api graphql -f query=", repo2_commits_response);

        let repository = GhCommandRepository::new(mock, NoOpProgressReporter::new(), NoOpCache);
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Invalid date");
        let to = NaiveDate::from_ymd_opt(2024, 12, 31).expect("Invalid date");

        let commits = repository
            .fetch_commits("test-org", from, to, None)
            .expect("Failed to fetch commits with nested pagination");

        assert_eq!(commits.len(), 3);
        assert_eq!(commits[0].sha(), "repo1_commit1");
        assert_eq!(commits[0].repository(), "test-org/repo-1");
        assert_eq!(commits[1].sha(), "repo1_commit2");
        assert_eq!(commits[1].repository(), "test-org/repo-1");
        assert_eq!(commits[2].sha(), "repo2_commit1");
        assert_eq!(commits[2].repository(), "test-org/repo-2");
    }
}
