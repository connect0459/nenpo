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
- [x] ローカルドキュメント読み込み
  - [x] DocumentContentエンティティ
  - [x] DocumentRepositoryトレイト
  - [x] LocalFileDocumentRepository実装
  - [x] Reportエンティティへのドキュメント情報追加
  - [x] Markdown出力へのドキュメントセクション追加
- [x] 複数部門のサポート
  - [x] ReportGeneratorサービス実装
  - [x] 複数部門ループ処理
  - [x] 部門フィルタ機能
  - [x] 年度期間計算
  - [x] main.rsへの統合
- [ ] JSON/HTML形式での出力
- [ ] Conventional Commitsによるテーマ別要約
