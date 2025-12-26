# テスト戦略

## テスト哲学

nenpoは**t-wada流TDD（デトロイト派）**に基づいて開発されています。

### デトロイト派の原則

1. **モックは外部境界のみ**: 内部のオブジェクト協調は実際のインスタンスでテスト
2. **実際のオブジェクトでテスト**: ビジネスロジックは本物のドメインオブジェクトを使用
3. **Living Documentation**: テスト名を日本語で記述し、仕様書として機能させる

## TDDワークフロー

### Red → Green → Refactor

```bash
# 1. Red: テストを書く（失敗する）
cargo test

# 2. Green: 最小限の実装で通す
cargo test

# 3. Refactor: リファクタリング
cargo test

# カバレッジ確認
cargo llvm-cov --all-features --workspace --summary-only
```

### 実践例

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

## テストカバレッジ

### 目標と実績

- **目標**: 80%以上
- **実績**: **89.51%** ✅

### カバレッジ計測

```bash
# cargo-llvm-covのインストール
cargo install cargo-llvm-cov

# カバレッジ計測
cargo llvm-cov --all-features --workspace --lcov --output-path coverage/lcov.info

# サマリー表示
cargo llvm-cov --all-features --workspace --summary-only
```

### カバレッジ詳細（Phase 2完了時点）

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

main.rs                                   0.00%  ⚠️ (エントリーポイント)
--------------------------------------------------------
TOTAL                                    89.51%  ✅
```

## テスト構造

### 1. ユニットテスト

各モジュールに`#[cfg(test)]`モジュールを配置。

```rust
// src/domain/entities/commit.rs
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    #[allow(non_snake_case)]
    fn コミットを作成できる() {
        // テスト実装
    }
}
```

### 2. 統合テスト

複数のコンポーネントを組み合わせたテスト。

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

テスト名を日本語で記述し、仕様書として機能させる。

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

## モック実装

### 外部境界のみモック化

デトロイト派の原則に従い、以下の外部境界のみモック化：

#### 1. CommandExecutor

```rust
#[cfg(test)]
pub struct MockCommandExecutor {
    responses: Arc<Mutex<Vec<(String, String)>>>,
    call_count: Arc<Mutex<usize>>,
}

impl CommandExecutor for MockCommandExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<String> {
        // モックレスポンスを返す
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

### 内部はリアルオブジェクト

ドメインオブジェクト（Commit, Report, CommitTheme等）は実際のインスタンスを使用。

## テストデータの管理

### Test Object Pattern

複雑なテストデータは構造体で管理。

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

コード品質を保つため、pre-commitフックを使用。

### 設定

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

### 実行

```bash
# 全ファイルでチェック
pre-commit run --all-files

# 自動的に実行（git commit時）
pre-commit install
```

## テストの実行

### 基本的なテスト実行

```bash
# 全テスト実行
cargo test

# 特定のテストのみ実行
cargo test コミットを作成できる

# 詳細な出力
cargo test -- --nocapture

# リリースビルドでテスト
cargo test --release
```

### カバレッジ付きテスト

```bash
# カバレッジ計測
cargo llvm-cov --all-features --workspace

# HTML形式で出力
cargo llvm-cov --all-features --workspace --html

# 特定のしきい値で失敗させる
cargo llvm-cov --all-features --workspace --fail-under-lines 80
```

## テストのベストプラクティス

### 1. 日本語テスト名

```rust
#[test]
#[allow(non_snake_case)]
fn キャッシュが存在しない場合はNoneを返す() {
    // テスト実装
}
```

### 2. AAA パターン（Arrange-Act-Assert）

```rust
#[test]
fn test_example() {
    // Arrange: テストデータの準備
    let input = create_test_data();

    // Act: テスト対象の実行
    let result = function_under_test(input);

    // Assert: 結果の検証
    assert_eq!(result, expected);
}
```

### 3. エッジケースのテスト

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

### 4. リグレッションテスト

バグ修正時は必ずテストを追加。

```rust
#[test]
#[allow(non_snake_case)]
fn organization不在時もuserデータを取得できる() {
    // Issue #XXX のリグレッションテスト
    // ...
}
```

## まとめ

nenpoのテスト戦略：

- ✅ **TDD実践**: Red → Green → Refactor
- ✅ **高カバレッジ**: 89.51%（目標80%超過）
- ✅ **デトロイト派**: モックは外部境界のみ
- ✅ **Living Documentation**: 日本語テスト名で仕様を表現
- ✅ **品質保証**: pre-commitフックで自動チェック

テストは単なる品質保証ツールではなく、設計ツールであり、ドキュメントでもあります。
