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
- [x] JSON/HTML形式での出力
  - [x] JsonOutputRepository実装
  - [x] HtmlOutputRepository実装
  - [x] ReportGeneratorにファイル拡張子パラメータ追加
  - [x] main.rsでフォーマット選択機能
- [x] Conventional Commitsによるテーマ別要約
  - [x] CommitTheme値オブジェクト実装
  - [x] Reportエンティティにtheme_summaryフィールド追加
  - [x] Markdown出力にCommit Themesセクション追加
  - [x] HTML出力にCommit Themesセクション追加
  - [x] JSON出力にtheme_summaryが含まれることを確認（serdeで自動対応）
  - [x] 実際のコミットメッセージ取得（Phase 2 完全実装: 基本機能）
    - [x] Commitエンティティ実装（SHA、メッセージ、作成者、日時、リポジトリ名）
    - [x] GitHubRepositoryトレイトに`fetch_commits()`メソッド追加
    - [x] GraphQLクエリ拡張（ページネーション対応）
    - [x] GhCommandRepositoryでコミット取得実装（再帰的ページネーション）
    - [x] MockCommandExecutor改良（複数レスポンス対応）
    - [x] ReportGeneratorでtheme_summary構築処理追加
    - [x] テスト作成（コミット取得、ページネーション、テーマ別要約）
  - [x] Phase 2 完全実装: 最適化機能 ✅
    - [x] 進捗表示機能（ProgressReporterトレイト）
      - [x] ProgressReporterトレイト定義
      - [x] StdoutProgressReporter実装
      - [x] NoOpProgressReporter実装
      - [x] GhCommandRepositoryに統合
    - [x] エラーハンドリング強化（API制限エラー、リトライ処理）
      - [x] RetryConfigとwith_retry関数実装
      - [x] 指数バックオフによるリトライロジック
      - [x] API rate limitエラー検出と自動リトライ
      - [x] GraphQLエラーハンドリング（organization不在時のユーザーデータ取得）
    - [x] キャッシュ機能（~/.cache/nenpo/にJSON保存）
      - [x] CommitCacheトレイト定義
      - [x] FileCache実装（~/.cache/nenpo/）
      - [x] NoOpCache実装
      - [x] GhCommandRepositoryに統合
      - [x] main.rsでFileCache有効化
      - [x] キャッシュヒット時の高速読み込み確認
    - [ ] 並列処理最適化（tokio非同期処理）※保留
    - [x] 統合テスト実施（89.51%カバレッジ達成、目標80%超過）✅
      - [x] cargo-llvm-covによるカバレッジ計測
      - [x] 全57テスト成功
      - [x] pre-commit checks全て通過
    - [x] 実際のGitHubデータで動作確認 ✅
      - [x] 849コミット取得成功（ページネーション動作確認）
      - [x] 進捗表示機能動作確認
      - [x] キャッシュ機能動作確認（2回目実行で高速化）
      - [x] Markdown/JSON/HTML全形式での出力確認
      - [x] Conventional Commitsテーマ集計動作確認
    - [x] 注意事項: API制限（認証済み5,000req/h）、大規模プロジェクトのメモリ消費

## Phase 3: リポジトリ内コミット履歴のページネーション（ADR-003）

**背景**: 各リポジトリから最大100件しかコミットを取得できず、コミットの取りこぼしが発生している（voyagegroup/ecnaviで646件あるのに100件のみ取得）

**関連ADR**: [ADR-003](adrs/adr-003-paginate-commits-within-repositories.md)

### Phase 3.1: 設計とデータ構造の準備

- [ ] CommitHistoryConnectionDetailedにpageInfo追加を確認
- [ ] PageInfo構造体の再利用性を確認
- [ ] GraphQLレスポンス構造の分析
  - [ ] 単一リポジトリのコミット履歴取得クエリをテスト
  - [ ] pageInfoとafterパラメータの動作確認

### Phase 3.2: GraphQLクエリの修正

- [ ] build_commits_query()の修正案を決定
  - [ ] Option A: 既存クエリにhistoryのafterパラメータを追加
  - [ ] Option B: build_repo_commits_query()を新規作成（単一リポジトリ用）
- [ ] 選択したオプションでクエリを実装
- [ ] クエリのテスト（手動でgh api graphqlで確認）

### Phase 3.3: コアロジックの実装

- [ ] fetch_commits()メソッドの修正
  - [ ] リポジトリリストの取得ループ（外側）を維持
  - [ ] 各リポジトリのコミット履歴のページネーションループ（内側）を追加
  - [ ] ネストしたループの実装
  - [ ] エラーハンドリングの追加
- [ ] parse_commits_response()の修正（必要に応じて）
- [ ] 進捗報告の改善
  - [ ] リポジトリごとの進捗表示
  - [ ] 現在のリポジトリ名の表示

### Phase 3.4: テストの追加

- [ ] ユニットテストの追加
  - [ ] リポジトリ内コミットのページネーションをテスト
  - [ ] 100件を超えるコミットを持つリポジトリのテスト
  - [ ] ネストしたページネーションのテスト
  - [ ] pageInfoのhasNextPageがfalseの場合のテスト
- [ ] 既存テストの修正（必要に応じて）
- [ ] テストカバレッジの確認（目標: 80%以上維持）

### Phase 3.5: 統合テストと動作確認

- [ ] キャッシュをクリア
- [ ] voyagegroup組織でテスト実行
  - [ ] ecnaviリポジトリで646件すべて取得できることを確認
  - [ ] ecnavi-enquete-appで100件以上取得できることを確認
  - [ ] 全リポジトリの合計コミット数を確認
- [ ] 個人アカウント、study-for-twoでも動作確認
- [ ] パフォーマンスの測定
  - [ ] API呼び出し回数の確認
  - [ ] 実行時間の測定
  - [ ] レート制限に引っかからないか確認

### Phase 3.6: ドキュメントとクリーンアップ

- [ ] ADR-003のステータスを「Accepted」に更新
- [ ] ADR-003の実績セクションを実データで更新
- [ ] コード内のコメント追加
- [ ] 実装時の学びをADRに追記
- [ ] Phase 3を完了としてマーク

### 技術的な検討事項

#### パフォーマンス最適化（将来の拡張）

- [ ] 並列処理の検討
  - [ ] rayonを使った各リポジトリの並列取得
  - [ ] API レート制限との兼ね合い
- [ ] 増分キャッシュの検討
  - [ ] リポジトリごとのキャッシュ保存
  - [ ] 差分取得の実装

#### エラーハンドリング

- [ ] リトライ機構の確認
  - [ ] ネストループでのリトライ動作
  - [ ] 部分的な失敗の処理（一部リポジトリでエラー）
- [ ] タイムアウト処理
  - [ ] 大量コミットを持つリポジトリでのタイムアウト対策

#### 品質保証

- [ ] エッジケースのテスト
  - [ ] コミット数0のリポジトリ
  - [ ] defaultBranchRefがnullのリポジトリ
  - [ ] 100件ちょうどのコミットを持つリポジトリ
- [ ] メモリ使用量の確認
  - [ ] 大量コミットでのメモリ消費
  - [ ] ストリーミング処理の検討（必要に応じて）

### 完了条件

- [ ] すべてのテストが通過（60+ tests）
- [ ] voyagegroup/ecnaviで646件すべてのコミットが取得できる
- [ ] レポートのYour Commitsが正確な数値になる
- [ ] パフォーマンスが許容範囲内（レート制限に引っかからない）
- [ ] テストカバレッジ80%以上維持
- [ ] ADR-003のステータスが「Accepted」
- [ ] ドキュメント更新完了
