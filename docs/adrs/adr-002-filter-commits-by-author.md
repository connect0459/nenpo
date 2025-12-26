# ADR-002: 特定ユーザーのコミットのみを取得する機能

## ステータス

- [ ] Proposed
- [x] Accepted
- [x] Implemented (2025-12-26)
- [ ] Deprecated

## コンテキスト

### 背景

年次報告書では、複数の組織（個人アカウント、STUDY FOR TWO、CARTA HOLDINGS等）における特定ユーザーの活動を集計したい。

### 問題点

Phase 2の初期実装では、組織のすべてのリポジトリのすべてのコミットを取得しようとしていた。これにより、以下の問題が発生：

1. **データ量の爆発**: voyagegroupのような大規模組織では、数千〜数万のコミットが存在
2. **API制限**: GitHub GraphQL APIのレート制限に引っかかる可能性
3. **パースエラー**: レスポンスサイズが大きすぎてJSONパースに失敗
4. **処理時間**: 不要なコミットまで取得するため、処理時間が長い
5. **不正確な集計**: 特定ユーザーの活動のみを知りたいのに、組織全体のコミットを取得していた

### 実際のエラー

```text
Fetching commits for voyagegroup...
  2779 commits fetched from voyagegroup...
  4227 commits fetched from voyagegroup...
Error: Failed to generate report: Failed to parse commits GraphQL response
```

### 要求事項

設定ファイル `nenpo-config.toml` のコメント「connect0459の活動を3つの組織で集計」から：

- connect0459ユーザーが各組織で作成したコミットのみを集計
- 組織内の他のユーザーのコミットは不要
- 効率的にデータを取得（API制限を回避）

## 決定事項

### 1. GitHub GraphQL APIのauthorフィルタを採用

**理由**:

- GitHub GraphQL APIの`history`クエリには`author`パラメータが存在
- 特定ユーザーのコミットのみをサーバー側でフィルタリング可能
- GitHub Search APIよりもレート制限が緩い
- contributionsCollection APIはコミットメッセージを取得できない

**形式**:

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

### 2. 設定ファイルにtarget_github_user追加

組織の場合、特定ユーザーのコミットのみを取得するよう設定可能に：

```toml
[global]
# すべての組織で対象とするGitHubユーザー（オプショナル）
target_github_user = "connect0459"

[[departments]]
name = "CARTA HOLDINGS"
github_organizations = ["voyagegroup"]
# このフィールドでtarget_github_userを上書き可能（将来の拡張）
# target_github_user = "other_user"
```

### 3. 実装方針

#### 3.1 ドメイン層

**Config entity**:

```rust
pub struct Config {
    target_github_user: Option<String>,  // 新規フィールド
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
        author: Option<&str>,  // 新規パラメータ
    ) -> Result<Vec<Commit>>;
}
```

#### 3.2 インフラ層

**ユーザーID取得**:

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

**GraphQLクエリ修正**:

```rust
fn build_commits_query(
    org_or_user: &str,
    from: NaiveDate,
    to: NaiveDate,
    after_cursor: Option<&str>,
    author_id: Option<&str>,  // 新規パラメータ
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

#### 3.3 アプリケーション層

**ReportGenerator**:

```rust
let author = self.config_repository.get()?.target_github_user();
self.github_repository.fetch_commits(org, from, to, author.as_deref())?;
```

### 4. キャッシュ戦略

キャッシュファイル名にauthor情報を含める：

```text
~/.cache/nenpo/voyagegroup_20250101_20251231_connect0459_commits.json
```

**理由**:

- 同じ組織でも、ユーザーが異なれば別のデータ
- キャッシュの一意性を保証

## 結果

### 効果

1. **データ量削減**: voyagegroupで数万コミット → 数百コミット（connect0459のみ）
2. **API制限回避**: サーバー側フィルタリングにより、転送データ量が大幅削減
3. **処理速度向上**: 不要なデータを取得しないため、高速化
4. **正確な集計**: ユーザーの意図通り、特定ユーザーの活動のみを集計
5. **エラー解消**: GraphQLレスポンスパースエラーが解消

### 実績

connect0459ユーザーのコミット数（2025年1月1日〜12月31日）：

- 個人アカウント (connect0459): 842件
- STUDY FOR TWO (study-for-two): 437件
- CARTA HOLDINGS (voyagegroup): 329件（2025年度: 2025年4月1日〜2026年3月31日）

合計: 約1,608件（組織全体では数万件）

**効果測定**:

- voyagegroup組織: 4,227件 → 329件（約92%削減）
- データ取得の成功率: 100%（以前は大量データでパースエラー）

### 制限事項

1. **GitHub user ID取得**: 初回実行時に追加のGraphQLクエリが必要
2. **キャッシュ無効化**: authorが変わった場合、キャッシュをクリアする必要がある
3. **複数ユーザー**: 現在の実装では1ユーザーのみ対応（将来の拡張で複数対応可能）

### 実装時の問題と解決

#### 問題: TomlConfigRepositoryがtarget_github_userを読み込んでいなかった

**発見経緯**:

実装完了後の動作確認で、voyagegroup組織から依然として大量のコミット（2892件、4888件）が取得され、author filterが適用されていないことが判明。調査の結果、`TomlConfig`構造体に`target_github_user`フィールドが存在せず、設定ファイルから読み込まれていなかった。

**症状**:

```text
Fetching commits for voyagegroup...
  2892 commits fetched from voyagegroup...  # author filterが効いていない
  4888 commits fetched from voyagegroup...
Error: Failed to parse commits GraphQL response
```

**原因**:

```rust
// 修正前: TomlConfig構造体にtarget_github_userフィールドが存在しない
struct TomlConfig {
    default_fiscal_year_start_month: u32,
    default_output_format: String,
    output_directory: String,
    departments: Vec<TomlDepartment>,
}

// Config::new()を使用（target_github_userを受け取らない）
Ok(Config::new(
    toml_config.default_fiscal_year_start_month,
    output_format,
    toml_config.output_directory,
    departments,
))
```

**解決方法**:

1. `TomlConfig`構造体に`target_github_user`フィールドを追加：

```rust
struct TomlConfig {
    #[serde(default)]  // オプショナルフィールド
    target_github_user: Option<String>,
    default_fiscal_year_start_month: u32,
    default_output_format: String,
    output_directory: String,
    departments: Vec<TomlDepartment>,
}
```

1. `Config::with_target_user()`を使用するように変更：

```rust
Ok(Config::with_target_user(
    toml_config.target_github_user,  // 追加
    toml_config.default_fiscal_year_start_month,
    output_format,
    toml_config.output_directory,
    departments,
))
```

1. テストを追加：

```rust
#[test]
fn target_github_userを含む設定を読み込める() {
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

**結果**: 修正後、author filterが正しく適用され、voyagegroup組織で329件のコミット（connect0459のみ）が正常に取得できるようになった。

## 参考資料

- [GitHub GraphQL API - CommitHistoryConnection](https://docs.github.com/en/graphql/reference/objects#commithistoryconnection)
- [GitHub GraphQL API - CommitAuthor](https://docs.github.com/en/graphql/reference/input-objects#commitauthor)
- [ADR-001: Conventional Commitsによるテーマ別コミット集計](./adr-001-aggregate-by-commits.md)

## 関連ファイルのパス

### 初期実装時 (2025-12-26)

#### ドメイン層

- `src/domain/entities/config.rs` (target_github_user フィールド追加)
- `src/domain/repositories/github_repository.rs` (fetch_commits に author 引数追加)

#### インフラ層

- `src/infrastructure/github/gh_command_repository.rs` (変更)
  - fetch_user_id() メソッド追加
  - build_commits_query() に author_id パラメータ追加
  - fetch_commits() 実装修正
- `src/infrastructure/cache/commit_cache.rs` (変更)
  - キャッシュファイル名に author 情報を含める

#### アプリケーション層

- `src/application/services/report_generator.rs` (変更)
  - fetch_commits() 呼び出し時に author を渡す

#### 設定

- `src/infrastructure/config/toml_config_repository.rs` (変更)
  - TomlConfig構造体に target_github_user フィールドを追加
  - Config::with_target_user() を使用するように変更
  - target_github_user を含む設定を読み込むテストを追加

#### テスト

- `src/domain/entities/config.rs` (tests module)
  - target_github_user を含む設定を作成できる
  - target_github_user が None の場合
- `src/infrastructure/config/toml_config_repository.rs` (tests module)
  - target_github_user を含む設定を読み込める
  - target_github_user がない設定でも読み込める（後方互換性）
- `src/infrastructure/github/gh_command_repository.rs` (tests module)
  - ユーザーIDを取得できる
  - 特定ユーザーのコミットのみを取得できる
  - authorなしでもコミットを取得できる（後方互換性）

#### ドキュメント

- `nenpo-config.toml.example` (target_github_user 設定例追加)
- `README.md` (target_github_user の説明追加)

## 今後の拡張可能性

### 1. 部門ごとのtarget_user設定

```toml
[[departments]]
name = "Team A"
github_organizations = ["org-a"]
target_github_user = "user_a"  # 部門ごとに異なるユーザー

[[departments]]
name = "Team B"
github_organizations = ["org-b"]
target_github_user = "user_b"
```

### 2. 複数ユーザーのサポート

```toml
[global]
target_github_users = ["connect0459", "other_user"]
```

### 3. Issueとdraft PRのフィルタリング

現在はコミットのみだが、IssueやPRも同様にauthorでフィルタリング可能：

```graphql
issues(first: 100, filterBy: {createdBy: "connect0459"}) {
  nodes { ... }
}
```
