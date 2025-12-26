# ADR-001: Theme-Based Commit Aggregation Using Conventional Commits

## Status

- [ ] Proposed
- [x] Accepted
- [ ] Deprecated

## Context

In annual reports, we want to visualize not just the number of commits, but "what kind of activities were performed."

### Background

- GitHub commit count is useful as a quantitative metric, but qualitative aspects are not visible
- We want to understand whether "there were many new feature developments" or "many bug fixes"
- Manual classification of commit messages is not realistic

### Requirements

1. Automatically extract themes from commit messages
2. Include theme-based aggregation in annual reports
3. Comply with industry-standard conventions
4. Handle commits that don't follow the format appropriately

## Decision

### 1. Adopt Conventional Commits

**Rationale**:

- Widely adopted as an industry standard
- Used by many OSS projects including Angular, Electron, and Vue.js
- High compatibility with semantic versioning
- Clear specification is defined

**Format**:

```text
<type>(<scope>): <subject>

type: feat, fix, docs, refactor, test, chore, style, ci, build
scope: optional (can be omitted)
subject: brief description of changes
```

### 2. Supported Themes

Define the following 9 theme types + Other category:

| Type | Theme | Description | Example |
| :--- | :--- | :--- | :--- |
| `feat` | New Features | New feature addition | `feat: add user authentication` |
| `fix` | Bug Fixes | Bug fix | `fix: resolve login issue` |
| `docs` | Documentation | Documentation update | `docs: update README` |
| `refactor` | Refactoring | Refactoring | `refactor: simplify auth logic` |
| `test` | Tests | Add/modify tests | `test: add unit tests` |
| `chore` | Chores | Chores, dependency updates | `chore: update dependencies` |
| `style` | Code Style | Code style fixes | `style: format code` |
| `ci` | CI/CD | CI/CD configuration changes | `ci: add GitHub Actions` |
| `build` | Build System | Build system changes | `build: update webpack` |
| - | Other | Other than above | Other commits |

### 3. Implementation Strategy

#### 3.1 Implement as Value Object

```rust
// src/domain/value_objects/commit_theme.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommitTheme {
    Feat,
    Fix,
    Docs,
    Refactor,
    Test,
    Chore,
    Style,
    Ci,
    Build,
    Other,
}
```

**Rationale**:

- Express as an immutable concept in the domain layer
- Ensure compile-time safety through pattern matching
- Support serialization/deserialization (for JSON/caching)

#### 3.2 Case-Insensitive

```rust
impl CommitTheme {
    pub fn from_commit_message(message: &str) -> Self {
        let lower_message = message.to_lowercase();
        // ...
    }
}
```

**Rationale**:

- Tolerant of human input errors
- Treat `Feat:`, `FEAT:`, `feat:` all as the same theme

#### 3.3 Support Scopes in Parentheses

```rust
if lower_message.starts_with("feat:") || lower_message.starts_with("feat(") {
    CommitTheme::Feat
}
```

**Rationale**:

- Formats like `feat(api): add endpoint` are also standard
- Classify as the same theme regardless of scope presence

#### 3.4 Default to Other

Commits that don't follow the format are classified as `Other` category.

**Rationale**:

- Don't treat as errors (tolerant design)
- Include all commits in aggregation
- Available for existing projects

### 4. Aggregation Processing Implementation

```rust
// src/application/services/report_generator.rs
fn build_theme_summary(commits: &[Commit]) -> HashMap<CommitTheme, u32> {
    let mut theme_summary = HashMap::new();
    for commit in commits {
        let theme = CommitTheme::from_commit_message(commit.message());
        *theme_summary.entry(theme).or_insert(0) += 1;
    }
    theme_summary
}
```

**Features**:

- Simple loop processing
- O(1) count updates with `HashMap`
- Extract from immutable Commit objects

### 5. Output Format

#### Markdown

```markdown
### Commit Themes

- Other: 211
- Bug Fixes: 173
- New Features: 170
- Documentation: 140
- Chores: 85
...
```

#### JSON

```json
{
  "theme_summary": {
    "feat": 170,
    "fix": 173,
    "docs": 140,
    "refactor": 30,
    "test": 13,
    "chore": 85,
    "style": 5,
    "ci": 8,
    "build": 14,
    "other": 211
  }
}
```

## Consequences

### Results (connect0459, 2025)

Classified 849 commits as follows:

- Other: 211 (24.9%)
- Bug Fixes: 173 (20.4%)
- New Features: 170 (20.0%)
- Documentation: 140 (16.5%)
- Chores: 85 (10.0%)
- Refactoring: 30 (3.5%)
- Build System: 14 (1.6%)
- Tests: 13 (1.5%)
- CI/CD: 8 (0.9%)
- Code Style: 5 (0.6%)

### Effects

1. **Qualitative Visualization**: From simply "1,441 commits" to specific activity details like "170 new features, 173 bug fixes"
2. **Project Characteristics Understanding**: Discovered that documentation accounts for 16.5% of all work
3. **Available for Existing Projects**: Even for projects not fully compliant with Conventional Commits, themes can be partially extracted (absorbed by Other category)

### Limitations

1. **Japanese Commit Message Support**: Currently only supports English `feat:`, `fix:`, etc. Japanese equivalents like "機能:", "修正:" are not supported
2. **Multi-Line Commit Messages**: Only analyzes the first line (following common Conventional Commits practice)
3. **Detailed Scope Analysis**: Scope portion of `feat(api)` is ignored (theme classification only)

## References

- [Conventional Commits Specification](https://www.conventionalcommits.org/)
- [Angular Commit Message Guidelines](https://github.com/angular/angular/blob/main/CONTRIBUTING.md#commit)
- [Semantic Versioning 2.0.0](https://semver.org/)

## Related File Paths

### Phase 2 Implementation (2025-01-26)

#### Domain Layer

- `src/domain/value_objects/commit_theme.rs` (new)
  - CommitTheme enum definition
  - from_commit_message() method implementation
  - display_name(), short_name() method implementation
- `src/domain/entities/commit.rs` (new)
  - Commit entity definition
  - Holds SHA, message, author, date, repository info
- `src/domain/entities/report.rs` (modified)
  - Added theme_summary field: `HashMap<CommitTheme, u32>`
  - Added theme_summary() getter

#### Application Layer

- `src/application/services/report_generator.rs` (modified)
  - Added build_theme_summary() method
  - Added fetch_commits() call
  - Integrated theme aggregation logic

#### Infrastructure Layer

- `src/infrastructure/output/markdown_output_repository.rs` (modified)
  - Added Commit Themes section
  - Sort and display themes by count in descending order
- `src/infrastructure/output/json_output_repository.rs` (modified)
  - Automatically serialize theme_summary to JSON (serde)
- `src/infrastructure/output/html_output_repository.rs` (modified)
  - Added Commit Themes section (HTML format)

#### Tests

- `src/domain/value_objects/commit_theme.rs` (tests module)
  - Can extract theme from commit message
  - Case-insensitive
  - Non-conforming commits become Other
  - Can get short name
  - Can get display name
- `src/application/services/report_generator.rs` (tests module)
  - Can build theme summary from commit messages

### Test Coverage

- `src/domain/value_objects/commit_theme.rs`: 81.74%
- `src/domain/entities/commit.rs`: 100%
- `src/application/services/report_generator.rs`: 87.83%

## Future Extensibility

### 1. Japanese Support

```rust
// Future implementation idea
if lower_message.starts_with("機能:") || lower_message.starts_with("feat:") {
    CommitTheme::Feat
}
```

### 2. Custom Themes

Allow defining custom themes in configuration file:

```toml
[custom_themes]
perf = "Performance"
security = "Security"
```

### 3. Aggregation by Scope

```json
{
  "theme_summary": {
    "feat": {
      "total": 170,
      "by_scope": {
        "api": 50,
        "ui": 80,
        "db": 40
      }
    }
  }
}
```

### 4. Time Series Trends

Visualize theme distribution by month/week.
