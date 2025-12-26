# ADR-003: リポジトリ内コミット履歴のページネーション実装

## ステータス

- [x] Proposed
- [ ] Accepted
- [ ] Deprecated

## コンテキスト

### 背景

ADR-002でauthorフィルタを実装し、特定ユーザーのコミットのみを取得できるようになった。しかし、実際の運用で以下の問題が発見された：

### 問題点

**コミットの取りこぼし**が発生している：

- `voyagegroup/ecnavi`リポジトリ: 646件のコミットがあるが、100件しか取得できていない
- `voyagegroup/ecnavi-enquete-app`リポジトリ: 100件で打ち切られている
- 合計: 329件しか取得できていない（実際はもっと多い）

### 根本原因

現在の実装では、各リポジトリのコミット履歴を取得する際に：

```graphql
history(first: 100, since: "...", until: "...", author: {...}) {
  nodes { ... }
}
```

- `first: 100`で最大100件しか取得していない
- リポジトリのリスト自体はページネーションしているが、**各リポジトリ内のコミット履歴はページネーションしていない**
- 結果として、コミット数が100件を超えるリポジトリでは、残りのコミットが取得されない

### 影響範囲

- connect0459の実績: voyagegroup組織で329件取得 → 実際は646件以上（ecnaviだけで）
- 大規模リポジトリでの取りこぼしが発生
- 年次レポートの精度が低下

## 決定事項

### 1. 各リポジトリのコミット履歴をページネーションする

各リポジトリの`history`クエリでも`pageInfo`を取得し、ネストしたループでページネーションを実装する。

#### 修正前の構造

```rust
loop {
    // リポジトリのページネーション
    let query = build_commits_query(...);
    let repos = fetch_and_parse(...);

    for repo in repos {
        // 各リポジトリから最大100件のコミットを取得（ページネーションなし）
        commits.extend(repo.commits);
    }

    if !has_next_repo_page { break; }
}
```

#### 修正後の構造

```rust
loop {
    // リポジトリのページネーション
    let repos = fetch_repositories(...);

    for repo in repos {
        // 各リポジトリ内でコミット履歴をページネーション
        loop {
            let commits = fetch_repo_commits(repo, cursor);
            all_commits.extend(commits);

            if !has_next_commit_page { break; }
        }
    }

    if !has_next_repo_page { break; }
}
```

### 2. GraphQLクエリの修正

#### 修正前

```graphql
history(first: 100, since: "...", until: "...", author: {...}) {
  nodes {
    oid
    message
    author { name }
    committedDate
  }
}
```

#### 修正後

```graphql
history(first: 100, since: "...", until: "...", author: {...}, after: "cursor") {
  pageInfo {
    hasNextPage
    endCursor
  }
  nodes {
    oid
    message
    author { name }
    committedDate
  }
}
```

### 3. 実装方針

#### 3.1 アーキテクチャ

**2段階のページネーション**:

1. **外側のループ**: リポジトリリストのページネーション（既存）
2. **内側のループ**: 各リポジトリ内のコミット履歴のページネーション（新規）

#### 3.2 データ構造の変更

**CommitsRepository構造体**に`pageInfo`を追加:

```rust
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
    page_info: PageInfo,  // 既存
    nodes: Vec<CommitNode>,
}
```

#### 3.3 クエリビルダーの修正

**リポジトリごとのコミット取得用クエリ**を追加:

```rust
fn build_repo_commits_query(
    org_or_user: &str,
    repo_name: &str,
    from: NaiveDate,
    to: NaiveDate,
    author_id: Option<&str>,
    after_cursor: Option<&str>,
) -> String {
    // 単一リポジトリのコミット履歴を取得するクエリ
}
```

**または、既存のクエリを拡張**して、各リポジトリのhistoryにafterパラメータを追加。

#### 3.4 fetch_commits()メソッドの修正

```rust
fn fetch_commits(
    &self,
    org_or_user: &str,
    from: NaiveDate,
    to: NaiveDate,
    author: Option<&str>,
) -> Result<Vec<Commit>> {
    let author_id = if let Some(author_login) = author {
        Some(self.fetch_user_id(author_login)?)
    } else {
        None
    };

    // キャッシュチェック
    if let Some(ref cache) = self.cache {
        if let Some(cached) = cache.get(org_or_user, from, to, author)? {
            return Ok(cached);
        }
    }

    let mut all_commits = Vec::new();
    let mut repo_cursor: Option<String> = None;

    // 外側のループ: リポジトリのページネーション
    loop {
        let repos_query = build_repos_query(org_or_user, repo_cursor.as_deref());
        let repos = fetch_and_parse_repos(&repos_query)?;

        // 各リポジトリを処理
        for repo in repos.nodes {
            let mut commit_cursor: Option<String> = None;

            // 内側のループ: 各リポジトリのコミット履歴をページネーション
            loop {
                let commits_query = build_repo_commits_query(
                    org_or_user,
                    &repo.name,
                    from,
                    to,
                    author_id.as_deref(),
                    commit_cursor.as_deref(),
                );

                let commit_page = fetch_and_parse_commits(&commits_query)?;
                all_commits.extend(commit_page.commits);

                if commit_page.page_info.has_next_page {
                    commit_cursor = commit_page.page_info.end_cursor;
                } else {
                    break;
                }
            }
        }

        if repos.page_info.has_next_page {
            repo_cursor = repos.page_info.end_cursor;
        } else {
            break;
        }
    }

    // キャッシュに保存
    if let Some(ref cache) = self.cache {
        cache.set(org_or_user, from, to, author, &all_commits)?;
    }

    Ok(all_commits)
}
```

### 4. 進捗報告の改善

リポジトリごとの進捗を報告:

```rust
self.progress_reporter.report_repo_progress(repo_name, commit_count);
```

### 5. パフォーマンス考慮事項

#### API呼び出し回数の増加

- **修正前**: リポジトリページ数 × 1回
- **修正後**: リポジトリページ数 × (各リポジトリのコミットページ数)

**例**: voyagegroup組織（764リポジトリ）、ecnaviリポジトリ（646コミット = 7ページ）

- 修正前: 8ページ（リポジトリ100個ずつ）
- 修正後: 8ページ（リポジトリ） + 7ページ（ecnaviのコミット） + α（他のリポジトリ）

#### レート制限対策

- リトライ機構の活用（既存のwith_retry）
- 必要に応じて遅延追加を検討

## 結果

### 期待される効果

1. **完全なコミット取得**: すべてのコミットを漏れなく取得
2. **正確な統計**: 年次レポートの精度向上
3. **スケーラビリティ**: コミット数に関わらず対応可能

### 実績（予想）

connect0459ユーザーのコミット数（2025年1月1日〜12月31日）：

- **修正前**: 329件（取りこぼしあり）
- **修正後**: 650件以上（ecnaviだけで646件確認済み）

## 参考資料

- [GitHub GraphQL API - CommitHistoryConnection](https://docs.github.com/en/graphql/reference/objects#commithistoryconnection)
- [GitHub GraphQL API - PageInfo](https://docs.github.com/en/graphql/reference/objects#pageinfo)
- [ADR-002: 特定ユーザーのコミットのみを取得する機能](./adr-002-filter-commits-by-author.md)

## 関連ファイルのパス

### 初期実装時 (2025-12-26)

#### インフラ層

- `src/infrastructure/github/gh_command_repository.rs` (変更)
  - build_commits_query() の修正（historyにpageInfoとafterパラメータを追加）
  - または build_repo_commits_query() の追加（単一リポジトリ用クエリ）
  - fetch_commits() の修正（2段階ページネーション実装）
  - parse_commits_response() の修正（pageInfo対応）
  - CommitHistoryConnectionDetailed構造体の修正（pageInfo追加）

#### テスト

- `src/infrastructure/github/gh_command_repository.rs` (tests module)
  - リポジトリ内コミットのページネーションをテスト
  - 100件を超えるコミットを持つリポジトリのテスト
  - ネストしたページネーションのテスト

#### ドキュメント

- `docs/adrs/adr-003-paginate-commits-within-repositories.md` (新規)

## 今後の拡張可能性

### 1. 並列処理

各リポジトリのコミット取得を並列化して、パフォーマンスを向上:

```rust
use rayon::prelude::*;

repos.par_iter().map(|repo| {
    fetch_all_commits_for_repo(repo)
}).flatten().collect()
```

### 2. 増分キャッシュ

リポジトリごとにキャッシュを保存し、次回は差分のみ取得:

```json
{
  "org": "voyagegroup",
  "repos": {
    "ecnavi": {
      "last_cursor": "...",
      "commits": [...]
    }
  }
}
```

### 3. 進捗バーの詳細化

リポジトリごとの進捗を表示:

```text
Fetching commits for voyagegroup...
  [ecnavi] 100/646 commits fetched...
  [ecnavi] 200/646 commits fetched...
  ...
```
