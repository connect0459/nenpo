# nenpo（年報）

GitHubのリソースやローカルドキュメントから年次報告書を生成するコマンドラインツール。

## 特徴

- 📊 **GitHub活動の自動集計**: コミット、PR、イシュー、レビュー数を集計
- 🏷️ **Conventional Commits対応**: コミットメッセージから自動的にテーマ別に分類
- 📝 **複数の出力形式**: Markdown、JSON、HTML形式をサポート
- 🗂️ **複数部門対応**: 個人・企業など、複数の組織/ユーザーを部門別に管理
- ⚡ **高速キャッシュ**: 2回目以降の実行はキャッシュから高速読み込み
- 🔄 **自動リトライ**: GitHub API制限時の自動リトライ機能

## 必要要件

- Rust 1.70以上
- [GitHub CLI (`gh`)](https://cli.github.com/)
- GitHubアカウント（認証済み）

## インストール

```bash
# リポジトリをクローン
git clone https://github.com/connect0459/nenpo.git
cd nenpo

# ビルド
cargo build --release

# （オプション）パスを通す
cp target/release/nenpo ~/.local/bin/
```

## クイックスタート

### 1. 設定ファイルの作成

サンプルをコピーして編集：

```bash
cp nenpo-config.toml.example nenpo-config.toml
```

`nenpo-config.toml`を編集：

```toml
default_fiscal_year_start_month = 1
default_output_format = "markdown"
output_directory = "./reports"

[[departments]]
name = "Personal Projects"
fiscal_year_start_month = 1
github_organizations = ["your-github-username"]
local_documents = []
```

### 2. レポート生成

```bash
# 2025年のレポートを生成
./target/release/nenpo generate --config nenpo-config.toml --year 2025

# JSON形式で出力
./target/release/nenpo generate --config nenpo-config.toml --year 2025 --format json

# HTML形式で出力
./target/release/nenpo generate --config nenpo-config.toml --year 2025 --format html
```

### 3. レポート確認

```bash
# Markdownレポートを表示
cat ./reports/report-Personal\ Projects-2025.md
```

## 使い方

### 基本的なコマンド

```bash
# デフォルト設定でレポート生成
nenpo generate --config nenpo-config.toml

# 年度を指定
nenpo generate --config nenpo-config.toml --year 2024

# 特定の部門のみを処理
nenpo generate --config nenpo-config.toml --department "Personal Projects"

# 出力フォーマットを指定
nenpo generate --config nenpo-config.toml --year 2025 --format json
```

### オプション

- `--config <PATH>`: 設定ファイルのパス（必須）
- `--year <YEAR>`: 対象年度（年度開始月は設定ファイルから取得）
- `--department <NAME>`: 特定の部門のみを処理
- `--format <FORMAT>`: 出力フォーマット（`markdown`, `json`, `html`）

## 設定ファイル

### 基本構造

```toml
# デフォルトの年度開始月（1-12）
default_fiscal_year_start_month = 4

# デフォルトの出力フォーマット
default_output_format = "markdown"

# 出力ディレクトリ
output_directory = "./reports"

# 部門定義（複数定義可能）
[[departments]]
name = "個人"
fiscal_year_start_month = 4
github_organizations = ["connect0459"]
local_documents = []

[[departments]]
name = "企業"
fiscal_year_start_month = 4
github_organizations = ["voyagegroup"]
local_documents = ["docs/**/*.md"]
```

### 設定項目の説明

#### トップレベル設定

- `default_fiscal_year_start_month`: 年度の開始月（1=1月、4=4月など）
- `default_output_format`: デフォルトの出力形式（`markdown`, `json`, `html`）
- `output_directory`: レポートの出力先ディレクトリ

#### 部門設定（`[[departments]]`）

- `name`: 部門名（レポートのファイル名に使用）
- `fiscal_year_start_month`: この部門の年度開始月（トップレベルの設定を上書き）
- `github_organizations`: 対象のGitHub組織またはユーザー名のリスト
- `local_documents`: ローカルドキュメントのGlobパターン（現在未実装）

## 出力形式

### Markdown

人間が読みやすい形式。以下の情報が含まれます：

```markdown
# Annual Report 2025

## Personal Projects

### Period
- From: 2025-01-01
- To: 2025-12-31

### GitHub Activity
- Commits: 1441
- Pull Requests: 168
- Issues: 22
- Reviews: 0

### Commit Themes
- Other: 211
- Bug Fixes: 173
- New Features: 170
- Documentation: 140
...
```

### JSON

プログラムで処理しやすい形式：

```json
{
  "year": 2025,
  "department_name": "Personal Projects",
  "period_from": "2025-01-01",
  "period_to": "2025-12-31",
  "github_activity": {
    "commits": 1441,
    "pull_requests": 168,
    "issues": 22,
    "reviews": 0
  },
  "theme_summary": {
    "feat": 170,
    "fix": 173,
    "docs": 140,
    ...
  }
}
```

### HTML

Webブラウザで閲覧可能な形式。視覚的に整理されたレポートが生成されます。

## キャッシュ機能

nenpoは取得したコミット情報を`~/.cache/nenpo/`にキャッシュします。

- **初回実行**: GitHubからデータを取得（数秒〜数分）
- **2回目以降**: キャッシュから読み込み（瞬時）

### キャッシュのクリア

```bash
rm -rf ~/.cache/nenpo/
```

## Conventional Commits

nenpoはコミットメッセージを自動的に分類します：

| プレフィックス | テーマ | 例 |
| :--- | :--- | :--- |
| `feat:` | New Features | `feat: add user authentication` |
| `fix:` | Bug Fixes | `fix: resolve login issue` |
| `docs:` | Documentation | `docs: update README` |
| `refactor:` | Refactoring | `refactor: simplify auth logic` |
| `test:` | Tests | `test: add unit tests` |
| `chore:` | Chores | `chore: update dependencies` |
| `style:` | Code Style | `style: format code` |
| `ci:` | CI/CD | `ci: add GitHub Actions` |
| `build:` | Build System | `build: update webpack config` |
| その他 | Other | 上記に該当しないコミット |

## トラブルシューティング

### GitHub認証エラー

```bash
# GitHub CLIの認証状態を確認
gh auth status

# 認証されていない場合
gh auth login
```

### API制限エラー

GitHub APIには制限があります：

- **認証済み**: 5,000リクエスト/時
- **未認証**: 60リクエスト/時

nenpoは自動的にリトライしますが、大規模なリポジトリでは時間がかかる場合があります。

### organization not foundエラー

個人ユーザーの場合、このエラーは無視されます。userデータは正常に取得されます。

## 開発者向け情報

開発者の方は [docs/development/](docs/development/) を参照してください。

- [アーキテクチャ設計](docs/ARCHITECTURE.md)
- [テスト戦略](docs/development/testing.md)

## ライセンス

MIT License

## 作者

connect0459
