# Testing Strategy

## Testing Philosophy

nenpo is developed based on **t-wada-style TDD (Detroit School)**.

### Detroit School Principles

1. **Mock only at boundaries**: Test internal object collaboration with actual instances
2. **Test with real objects**: Use genuine domain objects for business logic
3. **Living Documentation**: Write test names in Japanese to function as specifications

## TDD Workflow

### Red → Green → Refactor

```bash
# 1. Red: Write test (fails)
cargo test

# 2. Green: Minimal implementation to pass
cargo test

# 3. Refactor: Refactoring
cargo test

# Check coverage
cargo llvm-cov --all-features --workspace --summary-only
```

### Practical Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn コミットを作成できる() {
        // Arrange
        let sha = "abc123".to_string();
        let message = "feat: add feature".to_string();
        let author = "John Doe".to_string();
        let date = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let repo = "owner/repo".to_string();

        // Act
        let commit = Commit::new(sha.clone(), message.clone(), author.clone(), date, repo.clone());

        // Assert
        assert_eq!(commit.message(), "feat: add feature");
        assert_eq!(commit.repository(), "owner/repo");
    }
}
```

## Test Coverage

### Target and Results

- **Target**: 80% or higher
- **Result**: **89.51%** ✅

### Coverage Measurement

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Measure coverage
cargo llvm-cov --all-features --workspace --lcov --output-path coverage/lcov.info

# Show summary
cargo llvm-cov --all-features --workspace --summary-only
```

### Coverage Details (Phase 2 Completion)

```text
File                                    Lines    Coverage
--------------------------------------------------------
domain/entities/commit.rs               100.00%  ✅
domain/entities/config.rs               100.00%  ✅
domain/entities/department.rs           100.00%  ✅
domain/entities/document_content.rs     100.00%  ✅
domain/entities/github_activity.rs      100.00%  ✅
domain/entities/report.rs               100.00%  ✅
domain/value_objects/output_format.rs   100.00%  ✅

domain/services/progress_reporter.rs     93.22%  ✅
domain/value_objects/commit_theme.rs     81.74%  ✅

infrastructure/cache/commit_cache.rs     91.80%  ✅
infrastructure/config/...                93.20%  ✅
infrastructure/document/...              97.27%  ✅
infrastructure/github/gh_command_...     88.86%  ✅
infrastructure/github/retry_handler.rs   96.45%  ✅
infrastructure/output/html_...           88.14%  ✅
infrastructure/output/json_...           98.32%  ✅
infrastructure/output/markdown_...       89.86%  ✅

main.rs                                   0.00%  ⚠️ (entry point)
--------------------------------------------------------
TOTAL                                    89.51%  ✅
```

## Test Structure

### 1. Unit Tests

Place `#[cfg(test)]` module in each module.

```rust
// src/domain/entities/commit.rs
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    #[allow(non_snake_case)]
    fn コミットを作成できる() {
        // Test implementation
    }
}
```

### 2. Integration Tests

Tests combining multiple components.

```rust
#[test]
#[allow(non_snake_case)]
fn 単一部門のレポートを生成できる() {
    let mock_config = MockConfigRepository::new()
        .with_config(config);
    let mock_github = MockGitHubRepository::new()
        .with_activity("connect0459", activity);
    let mock_document = MockDocumentRepository::new();
    let mock_output = MockOutputRepository::new();

    let generator = ReportGenerator::new(
        mock_config,
        mock_github,
        mock_document,
        mock_output,
    );

    let result = generator.generate(/* ... */);
    assert!(result.is_ok());
}
```

### 3. Living Documentation

Write test names in Japanese to function as specifications.

```rust
#[test]
#[allow(non_snake_case)]
fn コミットメッセージからテーマを抽出できる() { /* ... */ }

#[test]
#[allow(non_snake_case)]
fn 大文字小文字を区別しない() { /* ... */ }

#[test]
#[allow(non_snake_case)]
fn 形式に従わないコミットはOtherになる() { /* ... */ }
```

## Mock Implementation

### Mock Only at Boundaries

Following Detroit School principles, mock only the following external boundaries:

#### 1. CommandExecutor

```rust
#[cfg(test)]
pub struct MockCommandExecutor {
    responses: Arc<Mutex<Vec<(String, String)>>>,
    call_count: Arc<Mutex<usize>>,
}

impl CommandExecutor for MockCommandExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<String> {
        // Return mock response
    }
}
```

#### 2. ProgressReporter

```rust
pub struct NoOpProgressReporter;

impl ProgressReporter for NoOpProgressReporter {
    fn start_fetching_commits(&self, _org_or_user: &str) {}
    fn report_commits_progress(&self, _org_or_user: &str, _count: usize) {}
    fn finish_fetching_commits(&self, _org_or_user: &str, _total: usize) {}
    fn report_error(&self, _error: &str) {}
}
```

#### 3. CommitCache

```rust
pub struct NoOpCache;

impl CommitCache for NoOpCache {
    fn get(&self, ...) -> Result<Option<Vec<Commit>>> { Ok(None) }
    fn set(&self, ...) -> Result<()> { Ok(()) }
    fn clear(&self) -> Result<()> { Ok(()) }
}
```

### Use Real Objects Internally

Use actual instances for domain objects (Commit, Report, CommitTheme, etc.).

## Test Data Management

### Test Object Pattern

Manage complex test data with structures.

```rust
#[cfg(test)]
struct TestCommitBuilder {
    sha: String,
    message: String,
    author: String,
    date: DateTime<Utc>,
    repository: String,
}

#[cfg(test)]
impl TestCommitBuilder {
    fn new() -> Self {
        Self {
            sha: "default123".to_string(),
            message: "feat: default message".to_string(),
            author: "Test Author".to_string(),
            date: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            repository: "test/repo".to_string(),
        }
    }

    fn with_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    fn build(self) -> Commit {
        Commit::new(self.sha, self.message, self.author, self.date, self.repository)
    }
}
```

## pre-commit hooks

Use pre-commit hooks to maintain code quality.

### Configuration

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: rust-fmt
        name: rust fmt
        entry: cargo fmt --all -- --check
        language: system
        pass_filenames: false

      - id: rust-clippy
        name: rust clippy
        entry: cargo clippy --all-targets --all-features -- -D warnings
        language: system
        pass_filenames: false
```

### Execution

```bash
# Check all files
pre-commit run --all-files

# Auto-execute (on git commit)
pre-commit install
```

## Running Tests

### Basic Test Execution

```bash
# Run all tests
cargo test

# Run specific test only
cargo test コミットを作成できる

# Detailed output
cargo test -- --nocapture

# Test with release build
cargo test --release
```

### Tests with Coverage

```bash
# Measure coverage
cargo llvm-cov --all-features --workspace

# Output in HTML format
cargo llvm-cov --all-features --workspace --html

# Fail at specific threshold
cargo llvm-cov --all-features --workspace --fail-under-lines 80
```

## Test Best Practices

### 1. Japanese Test Names

```rust
#[test]
#[allow(non_snake_case)]
fn キャッシュが存在しない場合はNoneを返す() {
    // Test implementation
}
```

### 2. AAA Pattern (Arrange-Act-Assert)

```rust
#[test]
fn test_example() {
    // Arrange: Prepare test data
    let input = create_test_data();

    // Act: Execute test target
    let result = function_under_test(input);

    // Assert: Verify result
    assert_eq!(result, expected);
}
```

### 3. Edge Case Testing

```rust
#[test]
#[allow(non_snake_case)]
fn 年度開始月が1から12の範囲外の場合はエラーになる() {
    let result = Department::new("test", 0, vec![], vec![]);
    assert!(result.is_err());

    let result = Department::new("test", 13, vec![], vec![]);
    assert!(result.is_err());
}
```

### 4. Regression Testing

Always add tests when fixing bugs.

```rust
#[test]
#[allow(non_snake_case)]
fn organization不在時もuserデータを取得できる() {
    // Regression test for Issue #XXX
    // ...
}
```

## Summary

nenpo's testing strategy:

- ✅ **TDD Practice**: Red → Green → Refactor
- ✅ **High Coverage**: 89.51% (exceeds 80% target)
- ✅ **Detroit School**: Mock only at boundaries
- ✅ **Living Documentation**: Express specifications with Japanese test names
- ✅ **Quality Assurance**: Automatic checking with pre-commit hooks

Tests are not just quality assurance tools, but also design tools and documentation.
