# todo

## Phase 1: MVP

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
- [ ] Markdown出力機能の実装（TDD）
- [ ] CLIコマンドの実装
