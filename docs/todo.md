# todo

## Phase 1: MVP ✅ 完了

- [x] プロジェクトの初期化
- [x] Cargo.tomlの修正（edition、依存クレート追加）
- [x] アーキテクチャ設計のドキュメント化
- [x] オニオンアーキテクチャのディレクトリ構造作成
- [x] 設定ファイル読み込み機能の実装（TDD）
  - [x] Departmentエンティティ
  - [x] OutputFormat値オブジェクト
  - [x] Configエンティティ
  - [x] ConfigRepositoryトレイト
  - [x] TomlConfigRepository実装
- [x] GitHubデータ取得機能の実装（TDD）
  - [x] GitHubActivityエンティティ
  - [x] GitHubRepositoryトレイト
  - [x] GhCommandRepository実装（Phase 1: ダミーデータ）
- [x] Markdown出力機能の実装（TDD）
  - [x] Reportエンティティ
  - [x] OutputRepositoryトレイト
  - [x] MarkdownOutputRepository実装
- [x] CLIコマンドの実装
  - [x] CLI構造定義（clap）
  - [x] generateコマンド実装

## Phase 2: 拡張

- [x] 実際のGitHubデータ取得（gh コマンド実行）
  - [x] CommandExecutorトレイト定義
  - [x] GraphQL API実装
  - [x] MockCommandExecutorによるテスト
- [ ] ローカルドキュメント読み込み
- [ ] 複数部門のサポート
- [ ] JSON/HTML形式での出力
- [ ] Conventional Commitsによるテーマ別要約
