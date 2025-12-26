# ADR-002: Feature to Fetch Commits by Specific User

## Status

- [ ] Proposed
- [x] Accepted
- [x] Implemented (2025-12-26)
- [ ] Deprecated

## Context

### Background

In annual reports, we want to aggregate activities of specific users across multiple organizations (personal account, STUDY FOR TWO, CARTA HOLDINGS, etc.).

### Problem

In the initial Phase 2 implementation, we attempted to fetch all commits from all repositories of an organization. This caused the following issues:

1. **Data Explosion**: Large organizations like voyagegroup have thousands to tens of thousands of commits
2. **API Limits**: Risk of hitting GitHub GraphQL API rate limits
3. **Parse Errors**: Response size too large, causing JSON parsing failures
4. **Processing Time**: Slow due to fetching unnecessary commits
5. **Inaccurate Aggregation**: Wanted only a specific user's activities but fetched entire organization's commits

### Actual Error

```text
Fetching commits for voyagegroup...
  2779 commits fetched from voyagegroup...
  4227 commits fetched from voyagegroup...
Error: Failed to generate report: Failed to parse commits GraphQL response
```

### Requirements

From the comment in `nenpo-config.toml` "Aggregate connect0459's activities across 3 organizations":

- Aggregate only commits created by the connect0459 user in each organization
- Commits by other users in the organization are unnecessary
- Fetch data efficiently (avoid API limits)

## Decision

### 1. Adopt GitHub GraphQL API's author Filter

**Rationale**:

- GitHub GraphQL API's `history` query has an `author` parameter
- Can filter commits by specific user on the server side
- More lenient rate limits than GitHub Search API
- contributionsCollection API cannot fetch commit messages

**Format**:

```graphql
history(first: 100, since: "2025-01-01T00:00:00Z", until: "2025-12-31T23:59:59Z", author: {id: "MDQ6VXNlcjEyMzQ1Njc="}) {
  nodes {
    oid
    message
    author {
      name
    }
    committedDate
  }
}
```

### 2. Add target_github_user to Configuration File

For organizations, enable fetching only commits by specific user:

```toml
[global]
# Target GitHub user for all organizations (optional)
target_github_user = "connect0459"

[[departments]]
name = "CARTA HOLDINGS"
github_organizations = ["voyagegroup"]
# This field can override target_github_user (future extension)
# target_github_user = "other_user"
```

### 3. Implementation Strategy

#### 3.1 Domain Layer

**Config entity**:

```rust
pub struct Config {
    target_github_user: Option<String>,  // new field
    default_fiscal_year_start_month: u32,
    default_output_format: OutputFormat,
    output_directory: String,
    departments: Vec<Department>,
}
```

**GitHubRepository trait**:

```rust
pub trait GitHubRepository {
    fn fetch_commits(
        &self,
        org_or_user: &str,
        from: NaiveDate,
        to: NaiveDate,
        author: Option<&str>,  // new parameter
    ) -> Result<Vec<Commit>>;
}
```

#### 3.2 Infrastructure Layer

**Fetch User ID**:

```rust
fn fetch_user_id(&self, login: &str) -> Result<String> {
    let query = format!(r#"
        query {{
            user(login: "{}") {{
                id
            }}
        }}
    "#, login);
    // ...
}
```

**GraphQL Query Modification**:

```rust
fn build_commits_query(
    org_or_user: &str,
    from: NaiveDate,
    to: NaiveDate,
    after_cursor: Option<&str>,
    author_id: Option<&str>,  // new parameter
) -> String {
    let author_param = author_id
        .map(|id| format!(", author: {{id: \"{}\"}}", id))
        .unwrap_or_default();

    format!(r#"
        history(first: 100, since: "{}", until: "{}"{}}) {{
            nodes {{ ... }}
        }}
    "#, since, until, author_param)
}
```

#### 3.3 Application Layer

**ReportGenerator**:

```rust
let author = self.config_repository.get()?.target_github_user();
self.github_repository.fetch_commits(org, from, to, author.as_deref())?;
```

### 4. Cache Strategy

Include author information in cache file name:

```text
~/.cache/nenpo/voyagegroup_20250101_20251231_connect0459_commits.json
```

**Rationale**:

- Different data for same organization with different users
- Ensure cache uniqueness

## Consequences

### Effects

1. **Data Reduction**: voyagegroup from tens of thousands → hundreds of commits (connect0459 only)
2. **API Limit Avoidance**: Server-side filtering drastically reduces transfer data
3. **Processing Speed**: Faster by not fetching unnecessary data
4. **Accurate Aggregation**: Aggregates only specific user's activities as intended
5. **Error Resolution**: GraphQL response parse errors resolved

### Results

Commit counts for connect0459 user (Jan 1, 2025 - Dec 31, 2025):

- Personal account (connect0459): 842 commits
- STUDY FOR TWO (study-for-two): 437 commits
- CARTA HOLDINGS (voyagegroup): 329 commits (FY2025: Apr 1, 2025 - Mar 31, 2026)

Total: ~1,608 commits (tens of thousands in entire organizations)

**Impact Measurement**:

- voyagegroup organization: 4,227 → 329 commits (~92% reduction)
- Data fetch success rate: 100% (previously had parse errors with large data)

### Limitations

1. **GitHub user ID fetch**: Requires additional GraphQL query on first run
2. **Cache invalidation**: Need to clear cache if author changes
3. **Multiple users**: Current implementation supports only 1 user (future extension possible for multiple)

### Implementation Issues and Solutions

#### Issue: TomlConfigRepository Was Not Reading target_github_user

**Discovery**:

After implementation completion, during verification, large amounts of commits (2892, 4888) were still being fetched from voyagegroup organization, indicating author filter was not being applied. Investigation revealed that `TomlConfig` struct did not have `target_github_user` field and was not reading from config file.

**Symptoms**:

```text
Fetching commits for voyagegroup...
  2892 commits fetched from voyagegroup...  # author filter not working
  4888 commits fetched from voyagegroup...
Error: Failed to parse commits GraphQL response
```

**Cause**:

```rust
// Before fix: TomlConfig struct missing target_github_user field
struct TomlConfig {
    default_fiscal_year_start_month: u32,
    default_output_format: String,
    output_directory: String,
    departments: Vec<TomlDepartment>,
}

// Using Config::new() (doesn't accept target_github_user)
Ok(Config::new(
    toml_config.default_fiscal_year_start_month,
    output_format,
    toml_config.output_directory,
    departments,
))
```

**Solution**:

1. Add `target_github_user` field to `TomlConfig` struct:

```rust
struct TomlConfig {
    #[serde(default)]  // optional field
    target_github_user: Option<String>,
    default_fiscal_year_start_month: u32,
    default_output_format: String,
    output_directory: String,
    departments: Vec<TomlDepartment>,
}
```

2. Change to use `Config::with_target_user()`:

```rust
Ok(Config::with_target_user(
    toml_config.target_github_user,  // added
    toml_config.default_fiscal_year_start_month,
    output_format,
    toml_config.output_directory,
    departments,
))
```

3. Add test:

```rust
#[test]
fn can_load_config_with_target_github_user() {
    let toml_content = r#"
target_github_user = "connect0459"
default_fiscal_year_start_month = 1
default_output_format = "markdown"
output_directory = "./reports"
...
"#;
    let config = repository.load(Path::new(temp_file))?;
    assert_eq!(config.target_github_user(), Some("connect0459"));
}
```

**Result**: After fix, author filter was properly applied, successfully fetching only 329 commits (connect0459 only) from voyagegroup organization.

## References

- [GitHub GraphQL API - CommitHistoryConnection](https://docs.github.com/en/graphql/reference/objects#commithistoryconnection)
- [GitHub GraphQL API - CommitAuthor](https://docs.github.com/en/graphql/reference/input-objects#commitauthor)
- [ADR-001: Theme-Based Commit Aggregation Using Conventional Commits](./adr-001-aggregate-by-commits.md)

## Related File Paths

### Initial Implementation (2025-12-26)

#### Domain Layer

- `src/domain/entities/config.rs` (added target_github_user field)
- `src/domain/repositories/github_repository.rs` (added author argument to fetch_commits)

#### Infrastructure Layer

- `src/infrastructure/github/gh_command_repository.rs` (modified)
  - Added fetch_user_id() method
  - Added author_id parameter to build_commits_query()
  - Modified fetch_commits() implementation
- `src/infrastructure/cache/commit_cache.rs` (modified)
  - Include author information in cache file name

#### Application Layer

- `src/application/services/report_generator.rs` (modified)
  - Pass author when calling fetch_commits()

#### Configuration

- `src/infrastructure/config/toml_config_repository.rs` (modified)
  - Added target_github_user field to TomlConfig struct
  - Changed to use Config::with_target_user()
  - Added test to load config with target_github_user

#### Tests

- `src/domain/entities/config.rs` (tests module)
  - Can create config with target_github_user
  - Case when target_github_user is None
- `src/infrastructure/config/toml_config_repository.rs` (tests module)
  - Can load config with target_github_user
  - Can load config without target_github_user (backward compatibility)
- `src/infrastructure/github/gh_command_repository.rs` (tests module)
  - Can fetch user ID
  - Can fetch commits by specific user only
  - Can fetch commits without author (backward compatibility)

#### Documentation

- `nenpo-config.toml.example` (added target_github_user example)
- `README.md` (added target_github_user description)

## Future Extensibility

### 1. Per-Department target_user Setting

```toml
[[departments]]
name = "Team A"
github_organizations = ["org-a"]
target_github_user = "user_a"  # different user per department

[[departments]]
name = "Team B"
github_organizations = ["org-b"]
target_github_user = "user_b"
```

### 2. Multiple User Support

```toml
[global]
target_github_users = ["connect0459", "other_user"]
```

### 3. Issue and Draft PR Filtering

Currently commits only, but Issues and PRs can be similarly filtered by author:

```graphql
issues(first: 100, filterBy: {createdBy: "connect0459"}) {
  nodes { ... }
}
```
