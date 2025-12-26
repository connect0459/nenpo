# ADR-001: Conventional Commitsによるテーマ別コミット集計

## ステータス

- [ ] Proposed
- [x] Accepted
- [ ] Deprecated

## コンテキスト

年次報告書において、単なるコミット数だけでなく、「どのような種類の活動を行ったか」を可視化したい。

### 背景

- GitHubのコミット数は量的な指標として有用だが、質的な側面が見えない
- 「新機能開発が多かったのか」「バグ修正が多かったのか」を把握したい
- 手動でコミットメッセージを分類するのは現実的ではない

### 要求事項

1. コミットメッセージから自動的にテーマを抽出
2. テーマごとの集計を年次報告書に含める
3. 業界標準の規約に準拠
4. 形式に従わないコミットも適切に処理

## 決定事項

### 1. Conventional Commitsを採用

**理由**:

- 業界標準として広く採用されている
- Angular、Electron、Vue.jsなど多くのOSSプロジェクトで使用
- セマンティックバージョニングとの親和性が高い
- 明確な仕様が定義されている

**形式**:

```text
<type>(<scope>): <subject>

type: feat, fix, docs, refactor, test, chore, style, ci, build
scope: 任意（省略可）
subject: 変更内容の簡潔な説明
```

### 2. サポートするテーマ

以下の9種類のテーマ + Otherカテゴリを定義：

| Type | テーマ | 説明 | 例 |
| :--- | :--- | :--- | :--- |
| `feat` | New Features | 新機能追加 | `feat: add user authentication` |
| `fix` | Bug Fixes | バグ修正 | `fix: resolve login issue` |
| `docs` | Documentation | ドキュメント更新 | `docs: update README` |
| `refactor` | Refactoring | リファクタリング | `refactor: simplify auth logic` |
| `test` | Tests | テスト追加・修正 | `test: add unit tests` |
| `chore` | Chores | 雑務・依存関係更新 | `chore: update dependencies` |
| `style` | Code Style | コードスタイル修正 | `style: format code` |
| `ci` | CI/CD | CI/CD設定変更 | `ci: add GitHub Actions` |
| `build` | Build System | ビルドシステム変更 | `build: update webpack` |
| - | Other | 上記以外 | その他のコミット |

### 3. 実装方針

#### 3.1 値オブジェクトとして実装

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

**理由**:

- ドメイン層の不変な概念として表現
- パターンマッチングでコンパイル時の安全性を確保
- シリアライズ/デシリアライズ対応（JSON/キャッシュ用）

#### 3.2 大文字小文字を区別しない

```rust
impl CommitTheme {
    pub fn from_commit_message(message: &str) -> Self {
        let lower_message = message.to_lowercase();
        // ...
    }
}
```

**理由**:

- 人間の入力ミスに寛容
- `Feat:`, `FEAT:`, `feat:` すべて同じテーマとして扱う

#### 3.3 括弧付きスコープに対応

```rust
if lower_message.starts_with("feat:") || lower_message.starts_with("feat(") {
    CommitTheme::Feat
}
```

**理由**:

- `feat(api): add endpoint` のような形式も標準的
- スコープの有無に関わらず同じテーマとして分類

#### 3.4 デフォルトはOther

形式に従わないコミットは `Other` カテゴリに分類。

**理由**:

- エラーとして扱わない（寛容な設計）
- すべてのコミットを集計対象にする
- 既存のプロジェクトでも利用可能

### 4. 集計処理の実装

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

**特徴**:

- シンプルなループ処理
- `HashMap` で O(1) のカウント更新
- イミュータブルなCommitオブジェクトから抽出

### 5. 出力形式

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

## 結果

### 実績（connect0459、2025年）

849コミットを以下のように分類：

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

### 効果

1. **質的な可視化**: 単なる「コミット数1,441件」から、「新機能170件、バグ修正173件」という具体的な活動内容が明確に
2. **プロジェクトの特性理解**: ドキュメント作成が全体の16.5%を占めていることが判明
3. **既存プロジェクトでも利用可能**: Conventional Commitsに完全準拠していないプロジェクトでも、部分的にテーマを抽出できる（Otherカテゴリで吸収）

### 制限事項

1. **日本語コミットメッセージへの対応**: 現在は英語の `feat:`, `fix:` 等のみ対応。日本語の「機能:」「修正:」などは未対応
2. **複数行コミットメッセージ**: 最初の行のみを解析（一般的なConventional Commitsの慣習に従う）
3. **詳細なスコープ解析**: `feat(api)` のスコープ部分は無視（テーマ分類のみ）

## 参考資料

- [Conventional Commits Specification](https://www.conventionalcommits.org/)
- [Angular Commit Message Guidelines](https://github.com/angular/angular/blob/main/CONTRIBUTING.md#commit)
- [Semantic Versioning 2.0.0](https://semver.org/)

## 関連ファイルのパス

### Phase 2 実装時 (2025-01-26)

#### ドメイン層

- `src/domain/value_objects/commit_theme.rs` (新規)
  - CommitTheme enum定義
  - from_commit_message() メソッド実装
  - display_name(), short_name() メソッド実装
- `src/domain/entities/commit.rs` (新規)
  - Commit エンティティ定義
  - SHA、メッセージ、作成者、日時、リポジトリ情報を保持
- `src/domain/entities/report.rs` (変更)
  - theme_summary フィールド追加: `HashMap<CommitTheme, u32>`
  - theme_summary() getter追加

#### アプリケーション層

- `src/application/services/report_generator.rs` (変更)
  - build_theme_summary() メソッド追加
  - fetch_commits() 呼び出し追加
  - テーマ集計ロジック統合

#### インフラ層

- `src/infrastructure/output/markdown_output_repository.rs` (変更)
  - Commit Themes セクション追加
  - テーマを件数の多い順にソート表示
- `src/infrastructure/output/json_output_repository.rs` (変更)
  - theme_summary を JSON に自動シリアライズ（serde）
- `src/infrastructure/output/html_output_repository.rs` (変更)
  - Commit Themes セクション追加（HTML形式）

#### テスト

- `src/domain/value_objects/commit_theme.rs` (tests module)
  - コミットメッセージからテーマを抽出できる
  - 大文字小文字を区別しない
  - 形式に従わないコミットはOtherになる
  - 短縮名を取得できる
  - 表示名を取得できる
- `src/application/services/report_generator.rs` (tests module)
  - コミットメッセージからテーマ別要約を構築できる

### テストカバレッジ

- `src/domain/value_objects/commit_theme.rs`: 81.74%
- `src/domain/entities/commit.rs`: 100%
- `src/application/services/report_generator.rs`: 87.83%

## 今後の拡張可能性

### 1. 日本語対応

```rust
// 将来的な実装案
if lower_message.starts_with("機能:") || lower_message.starts_with("feat:") {
    CommitTheme::Feat
}
```

### 2. カスタムテーマ

設定ファイルで独自のテーマを定義可能にする：

```toml
[custom_themes]
perf = "Performance"
security = "Security"
```

### 3. スコープ別集計

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

### 4. 時系列推移

月別・週別のテーマ分布を可視化。
