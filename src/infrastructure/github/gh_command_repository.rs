use crate::domain::entities::github_activity::GitHubActivity;
use crate::domain::repositories::github_repository::GitHubRepository;
use crate::infrastructure::github::CommandExecutor;
use anyhow::{Context, Result};
use chrono::NaiveDate;
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

/// GitHub repository implementation using gh command
#[allow(dead_code)] // Phase 2: Will be used when integrated into main application
pub struct GhCommandRepository<E: CommandExecutor> {
    executor: E,
}

impl<E: CommandExecutor> GhCommandRepository<E> {
    /// Creates a new GhCommandRepository instance
    #[allow(dead_code)] // Phase 2: Will be used when integrated into main application
    pub fn new(executor: E) -> Self {
        Self { executor }
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

impl<E: CommandExecutor> GitHubRepository for GhCommandRepository<E> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let activity = GhCommandRepository::<MockCommandExecutor>::parse_response(response)
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

        let repository = GhCommandRepository::new(mock);
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
}
