# ARCHITECTURE

## Overview

This project follows the **Onion Architecture** pattern to ensure clean separation of concerns, testability, and long-term maintainability.

## Layers

### 1. Domain Layer (Core)

The innermost layer containing business logic and entities.

**Responsibilities:**

- Define domain entities and value objects
- Define repository traits (interfaces)
- Implement pure business logic
- No dependencies on outer layers

**Modules:**

- `domain::entities`: Domain entities
  - `Commit`: GitHub commit information (SHA, message, author, date, repository)
  - `Department`: Department information
  - `Report`: Annual report
  - `GitHubActivity`: GitHub activity statistics
  - `DocumentContent`: Local document content
  - `Config`: Application configuration
- `domain::value_objects`: Value objects
  - `CommitTheme`: Conventional Commits theme (feat, fix, docs, etc.)
  - `OutputFormat`: Output format (Markdown, JSON, HTML)
- `domain::repositories`: Repository trait definitions
  - `ConfigRepository`: Configuration repository
  - `GitHubRepository`: GitHub data fetching repository
  - `DocumentRepository`: Document repository
  - `OutputRepository`: Output repository
- `domain::services`: Domain services
  - `ProgressReporter`: Progress reporting abstraction

**Example:**

```rust
// domain/entities/commit.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    sha: String,
    message: String,
    author: String,
    committed_date: DateTime<Utc>,
    repository: String,
}
```

### 2. Application Layer

Orchestrates use cases and application logic.

**Responsibilities:**

- Implement use cases (e.g., "Generate Report")
- Coordinate between domain and infrastructure
- Business workflow orchestration
- Build theme summaries from commit data

**Modules:**

- `application::services`: Application services
  - `ReportGenerator`: Report generation service

**Example:**

```rust
// application/services/report_generator.rs
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
```

### 3. Infrastructure Layer

Implements interfaces to external systems.

**Responsibilities:**

- Implement repository traits defined in domain layer
- Handle external API calls (GitHub via `gh` command)
- File I/O operations
- Configuration file parsing
- Caching mechanisms
- Retry logic for API rate limits

**Modules:**

- `infrastructure::cache`: Cache implementation
  - `CommitCache` trait: Cache abstraction
  - `FileCache`: File-based cache (~/.cache/nenpo/)
  - `NoOpCache`: No-cache implementation (for testing)
- `infrastructure::config`: TOML configuration file handling
  - `TomlConfigRepository`: TOML configuration file loading
- `infrastructure::github`: GitHub integration
  - `GhCommandExecutor`: `gh` command execution
  - `GhCommandRepository`: GitHub GraphQL API calls
  - `RetryHandler`: Retry logic for API rate limits
- `infrastructure::document`: Local document loading
  - `LocalFileDocumentRepository`: File loading with glob patterns
- `infrastructure::output`: Output implementation
  - `MarkdownOutputRepository`: Markdown format output
  - `JsonOutputRepository`: JSON format output
  - `HtmlOutputRepository`: HTML format output

**Example:**

```rust
// infrastructure/github/gh_command_repository.rs
pub struct GhCommandRepository<E, P, C>
where
    E: CommandExecutor,
    P: ProgressReporter,
    C: CommitCache,
{
    executor: E,
    progress_reporter: P,
    retry_config: RetryConfig,
    cache: Option<C>,
}

impl<E, P, C> GitHubRepository for GhCommandRepository<E, P, C> {
    fn fetch_commits(&self, org_or_user: &str, from: NaiveDate, to: NaiveDate)
        -> Result<Vec<Commit>> {
        // Implementation with pagination, retry, and caching
    }
}
```

### 4. Presentation Layer (CLI)

User-facing interface.

**Responsibilities:**

- Parse command-line arguments
- Validate user input
- Display results and error messages
- Delegate to application layer

**Modules:**

- `presentation::cli`: CLI command definitions using `clap`

**Example:**

```rust
// presentation/cli/mod.rs
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Generate {
        #[arg(short, long)]
        config: String,
        #[arg(short, long)]
        year: Option<i32>,
        #[arg(short, long)]
        department: Option<String>,
        #[arg(short, long)]
        format: Option<String>,
    },
}
```

## Dependency Rule

- **Inward dependencies only**: Outer layers can depend on inner layers, but NOT vice versa
- **Dependency Inversion**: Outer layers implement interfaces defined by inner layers
- **No leakage**: Domain layer has zero knowledge of infrastructure or presentation

```text
Presentation → Application → Domain ← Infrastructure
```

## Directory Structure

```text
src/
├── main.rs                          # Entry point
├── presentation/                    # Presentation layer
│   └── cli.rs                       # CLI interface definition
├── application/                     # Application layer
│   └── services/
│       └── report_generator.rs     # Report generation service
├── domain/                          # Domain layer
│   ├── entities/                    # Entities
│   │   ├── commit.rs               # Commit information
│   │   ├── config.rs               # Configuration
│   │   ├── department.rs           # Department
│   │   ├── document_content.rs     # Document content
│   │   ├── github_activity.rs      # GitHub activity
│   │   └── report.rs               # Report
│   ├── repositories/                # Repository traits
│   │   ├── config_repository.rs    # Configuration repository
│   │   ├── document_repository.rs  # Document repository
│   │   ├── github_repository.rs    # GitHub repository
│   │   └── output_repository.rs    # Output repository
│   ├── services/                    # Domain services
│   │   └── progress_reporter.rs    # Progress reporter
│   └── value_objects/               # Value objects
│       ├── commit_theme.rs         # Commit theme
│       └── output_format.rs        # Output format
└── infrastructure/                  # Infrastructure layer
    ├── cache/                       # Cache implementation
    │   └── commit_cache.rs         # Commit cache
    ├── config/                      # Configuration implementation
    │   └── toml_config_repository.rs
    ├── document/                    # Document implementation
    │   └── local_file_document_repository.rs
    ├── github/                      # GitHub implementation
    │   ├── command_executor.rs     # Command execution
    │   ├── gh_command_repository.rs # GitHub API implementation
    │   └── retry_handler.rs        # Retry handling
    └── output/                      # Output implementation
        ├── html_output_repository.rs
        ├── json_output_repository.rs
        └── markdown_output_repository.rs
```

## Phase 2 Implementation Highlights

### Commit Entity with Rich Domain Logic

```rust
impl CommitTheme {
    pub fn from_commit_message(message: &str) -> Self {
        let lower_message = message.to_lowercase();
        if lower_message.starts_with("feat:") {
            CommitTheme::Feat
        } else if lower_message.starts_with("fix:") {
            CommitTheme::Fix
        }
        // ... other themes
        else {
            CommitTheme::Other
        }
    }
}
```

### Pagination Support

GhCommandRepository implements recursive pagination to fetch all commits:

```rust
fn fetch_commits(&self, org_or_user: &str, from: NaiveDate, to: NaiveDate)
    -> Result<Vec<Commit>> {
    let mut all_commits = Vec::new();
    let mut after_cursor: Option<String> = None;

    loop {
        let query = Self::build_commits_query(org_or_user, from, to, after_cursor.as_deref());
        let response = with_retry(&self.retry_config, || { /* ... */ })?;

        // Parse and extend commits
        all_commits.extend(commits);

        // Check pagination
        if !page_info.has_next_page {
            break;
        }
        after_cursor = page_info.end_cursor;
    }

    Ok(all_commits)
}
```

### Retry Mechanism with Exponential Backoff

```rust
pub fn with_retry<F, T>(config: &RetryConfig, mut operation: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let mut delay = config.initial_delay_ms;
    for attempt in 0..=config.max_retries {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if is_rate_limit_error(&e) => {
                thread::sleep(Duration::from_millis(delay));
                delay = (delay as f64 * config.backoff_multiplier) as u64;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Cache Implementation

```rust
pub struct FileCache {
    cache_dir: PathBuf,  // ~/.cache/nenpo/
}

impl CommitCache for FileCache {
    fn get(&self, org_or_user: &str, from: NaiveDate, to: NaiveDate)
        -> Result<Option<Vec<Commit>>> {
        let cache_file = self.cache_file_path(org_or_user, from, to);
        // Read and deserialize from JSON file
    }

    fn set(&self, org_or_user: &str, from: NaiveDate, to: NaiveDate, commits: &[Commit])
        -> Result<()> {
        // Serialize and write to JSON file
    }
}
```

### Generic Type Parameters for Flexibility

```rust
pub struct GhCommandRepository<E, P, C>
where
    E: CommandExecutor,      // gh command execution
    P: ProgressReporter,     // Progress reporting
    C: CommitCache,          // Caching mechanism
{
    executor: E,
    progress_reporter: P,
    retry_config: RetryConfig,
    cache: Option<C>,
}
```

This design allows:

- Easy testing with mock implementations
- Flexible switching of implementations
- No runtime overhead (static dispatch)

## Testing Strategy

- **Domain Layer**: Pure unit tests with real objects (Detroit School TDD)
- **Application Layer**: Use case tests with mock repositories
- **Infrastructure Layer**: Integration tests with real external systems
- **Presentation Layer**: CLI integration tests

**Coverage Goal**: 80%+ overall
**Current Coverage**: **89.51%** ✅

### Coverage Details (Phase 2)

```text
File                                    Coverage
---------------------------------------------------
domain/entities/*                       100.00%  ✅
domain/value_objects/output_format.rs   100.00%  ✅
domain/value_objects/commit_theme.rs     81.74%  ✅
domain/services/progress_reporter.rs     93.22%  ✅

infrastructure/cache/commit_cache.rs     91.80%  ✅
infrastructure/github/retry_handler.rs   96.45%  ✅
infrastructure/github/gh_command_...     88.86%  ✅
infrastructure/output/*                  88-98%   ✅

main.rs                                   0.00%  ⚠️
---------------------------------------------------
TOTAL                                    89.51%  ✅
```

## Key Principles

1. **Rich Domain Objects**: Entities contain both data and behavior
2. **No Getters/Setters**: Use methods that express intent (e.g., `name()` instead of `getName()`)
3. **TDD First**: Write tests before implementation (Red → Green → Refactor)
4. **Mock Only Boundaries**: Mock external systems only, use real objects for internal logic
5. **Evergreen Tests**: Tests should represent business requirements and remain stable
6. **Living Documentation**: Test names in Japanese describe specifications

## Design Decisions

### 1. Trait-Based Abstraction

Instead of concrete types, we use traits to define boundaries:

```rust
// Domain defines the interface
pub trait GitHubRepository {
    fn fetch_commits(&self, ...) -> Result<Vec<Commit>>;
}

// Infrastructure implements it
impl<E, P, C> GitHubRepository for GhCommandRepository<E, P, C> {
    fn fetch_commits(&self, ...) -> Result<Vec<Commit>> {
        // Implementation
    }
}
```

### 2. Dependency Injection via Generics

```rust
// ReportGenerator depends on traits, not concrete types
impl<C, G, D, O> ReportGenerator<C, G, D, O>
where
    C: ConfigRepository,
    G: GitHubRepository,
    D: DocumentRepository,
    O: OutputRepository,
{
    pub fn new(
        config_repository: C,
        github_repository: G,
        document_repository: D,
        output_repository: O,
    ) -> Self {
        // ...
    }
}
```

Benefits:

- Easy testing with mock implementations
- Flexible to change implementations
- Business logic independent of technical details

### 3. GraphQL Error Handling

Challenge: Individual users don't have an "organization", causing GraphQL errors.

Solution: Accept stdout even if exit code is non-zero, as long as there's valid JSON data:

```rust
impl CommandExecutor for GhCommandExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(program).args(args).output()?;
        let stdout = String::from_utf8(output.stdout)?;

        // Return stdout if it contains data, even if command failed
        if !output.status.success() && stdout.is_empty() {
            bail!("Command failed: {}", stderr);
        }

        Ok(stdout)
    }
}
```

This allows nenpo to gracefully handle organization-not-found errors while still processing user data.

## Data Flow

```text
CLI
 ↓
ReportGenerator
 ↓ ←──────────────┐
ConfigRepository  │
 ↓                │
GitHubRepository  │ (via traits)
 ↓                │
DocumentRepository│
 ↓                │
OutputRepository──┘
 ↓
File System
```

## Summary

The Onion Architecture enables nenpo to achieve:

- ✅ **Business Logic Independence**: Domain layer has no technical dependencies
- ✅ **Testability**: Easy to inject mocks via traits
- ✅ **Maintainability**: Clear separation of concerns
- ✅ **Extensibility**: Easy to add new output formats or data sources
- ✅ **High Coverage**: 89.51% test coverage achieved

For more details:

- [Testing Strategy](development/testing.md)
- [Phase 2 Implementation](development/phase2-implementation.md)
- [Contributing Guide](development/contributing.md)
