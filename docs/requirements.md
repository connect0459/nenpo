# Requirements

## プロジェクト概要

一年間の活動をGitHubとローカルドキュメントから要約し、年次報告書を生成するCLIツール。

### プロジェクト名

**nenpou**（年報）

### 技術スタック

- 言語: Rust
- 開発手法: t-wada流TDD（デトロイト派）
- GitHub連携: `gh` コマンド経由

---

## 機能要件

### 1. データソース

#### 1.1 GitHub

- `gh` コマンドを使用してGitHubからデータを取得
- 部門ごとに対象組織/ユーザーを設定可能
  - 例: 「個人」部門 → `https://github.com/connect0459/`
  - 例: 「企業」部門 → `https://github.com/voyagegroup/`

#### 1.2 ローカルドキュメント

- Globパターンで対象ファイルを指定
- 例: `**/*-202*.md`、`nippo/**/*.md`
- 部門ごとに異なるGlobパターンを設定可能
- ファイルの全文を要約に含める

### 2. 集計項目

#### 2.1 基本的な活動量（必須）

- コミット数
- プルリクエスト数
- イシュー数
- レビュー数

#### 2.2 詳細な分析（オプション）

- 追加/削除行数
- 言語別統計
- 時系列推移
- コントリビューション先の一覧

#### 2.3 テーマ別要約（オプション）

- Conventional Commitsの形式に基づいてコミットを分類
  - `feat:` → 新機能
  - `fix:` → バグ修正
  - `docs:` → ドキュメント
  - `refactor:` → リファクタリング
  - など

### 3. 集計期間

#### 3.1 基本: 年度単位

- 年度の開始月は設定ファイルで指定可能
  - 例: 会計年度（4月〜3月）
  - 例: 暦年（1月〜12月）
- 部門ごとに異なる年度開始月を設定可能

#### 3.2 柔軟な期間指定

- コマンドライン引数で開始日・終了日を指定可能
- 設定ファイルでデフォルト期間を設定可能

### 4. 出力機能

#### 4.1 出力フォーマット

複数のフォーマットに対応:

- **Markdown**: 人間が読みやすい形式
- **JSON**: プログラムで処理しやすい形式
- **HTML**: Webブラウザで閲覧可能

設定ファイルまたはコマンドライン引数でフォーマットを指定

#### 4.2 出力先

- ファイルに保存
- 出力ディレクトリは設定ファイルで指定
- ファイル名は自動生成（例: `nenpou-2024.md`、`nenpou-個人-2024.json`）

#### 4.3 部門の扱い

以下の3つのモードをサポート:

1. **全部門を一括処理**: すべての部門を1つのレポートにまとめる
2. **部門別に分割**: 部門ごとに別々のレポートを生成
3. **特定部門のみ**: コマンドライン引数で部門を指定

### 5. 設定ファイル

#### 5.1 フォーマット

- **TOML形式**を採用
- 可読性が高く、コメントが書ける
- Rustエコシステムで標準的

#### 5.2 設定項目

```toml
# デフォルトの年度開始月（1-12）
default_fiscal_year_start_month = 4

# デフォルトの出力フォーマット
default_output_format = "markdown"

# 出力ディレクトリ
output_directory = "./reports"

# 部門定義
[[departments]]
name = "個人"
fiscal_year_start_month = 4  # 部門ごとに上書き可能
github_organizations = ["connect0459"]
local_documents = ["path/to/docs/**/*.md"]

[[departments]]
name = "企業"
fiscal_year_start_month = 4
github_organizations = ["voyagegroup"]
local_documents = [".connect0459/nippo/nippo-*.md"]
```

---

## CLI設計

### コマンド構成

シンプルな実行形式を採用:

```bash
# 基本的な使い方（設定ファイルの内容に従って実行）
nenpou generate

# 年度を指定
nenpou generate --year 2024

# 特定の部門のみを処理
nenpou generate --department 個人

# 出力フォーマットを指定
nenpou generate --format json

# カスタム期間を指定
nenpou generate --from 2024-01-01 --to 2024-12-31
```

### オプション

- `--year <YEAR>`: 対象年度（年度開始月は設定ファイルから取得）
- `--department <NAME>`: 特定の部門のみを処理
- `--format <FORMAT>`: 出力フォーマット（markdown, json, html）
- `--from <DATE>`: 開始日（YYYY-MM-DD形式）
- `--to <DATE>`: 終了日（YYYY-MM-DD形式）
- `--config <PATH>`: 設定ファイルのパス（デフォルト: `./nenpou.toml`）

---

## 実装例

### 部門設定の例

```toml
[[departments]]
name = "個人"
fiscal_year_start_month = 4
github_organizations = ["connect0459"]
github_users = []  # 特定ユーザーも指定可能
local_documents = []

[[departments]]
name = "企業"
fiscal_year_start_month = 4
github_organizations = ["voyagegroup"]
local_documents = [
  ".connect0459/nippo/nippo-ca*.md",
  "docs/monthly-reports/**/*.md"
]
```

### 出力イメージ（Markdown）

```markdown
# 年次報告書 2024年度

## 個人部門

### GitHub活動

- コミット数: 1,234
- プルリクエスト: 56
- イシュー: 23
- レビュー: 78

#### テーマ別

- feat: 新機能実装 (45件)
- fix: バグ修正 (32件)
- docs: ドキュメント更新 (12件)

### ローカルドキュメント

（該当なし）

---

## 企業部門

### GitHub活動

- コミット数: 567
- プルリクエスト: 89
- イシュー: 45
- レビュー: 123

### ローカルドキュメント

- nippo-ca02569-202501.md
- nippo-ca02569-202502.md
...
```

---

## 非機能要件

### エラーハンドリング

- GitHubアクセストークンが無い場合は明確なエラーメッセージを表示
- リポジトリにアクセスできない場合はスキップして継続
- ローカルドキュメントが存在しない場合は警告を表示

### パフォーマンス

- GitHub APIのレート制限を考慮
- 並行処理で複数リポジトリを効率的に取得

### テスト

- t-wada流TDD（デトロイト派）に従う
- モックは外部境界（GitHub API、ファイルシステム）のみ
- ビジネスロジックは実際のオブジェクトでテスト

---

## 開発ロードマップ

### Phase 1: MVP

1. 設定ファイルの読み込み（TOML）
2. GitHub基本活動量の集計（コミット、PR、Issue）
3. Markdown形式での出力
4. 単一部門のサポート

### Phase 2: 拡張

1. ローカルドキュメントの読み込み
2. 複数部門のサポート
3. Conventional Commitsによるテーマ別要約
4. JSON/HTML形式での出力

### Phase 3: 詳細分析

1. 追加/削除行数の集計
2. 言語別統計
3. 時系列推移グラフ（HTMLフォーマット時）
4. コントリビューション先の詳細分析
