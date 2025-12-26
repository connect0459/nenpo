# todo

## Phase 1: MVP ✅ Completed

- [x] Project initialization
- [x] Cargo.toml modification (edition, dependency crates)
- [x] Architecture design documentation
- [x] Onion architecture directory structure creation
- [x] Configuration file reading functionality (TDD)
  - [x] Department entity
  - [x] OutputFormat value object
  - [x] Config entity
  - [x] ConfigRepository trait
  - [x] TomlConfigRepository implementation
- [x] GitHub data fetching functionality (TDD)
  - [x] GitHubActivity entity
  - [x] GitHubRepository trait
  - [x] GhCommandRepository implementation (Phase 1: dummy data)
- [x] Markdown output functionality (TDD)
  - [x] Report entity
  - [x] OutputRepository trait
  - [x] MarkdownOutputRepository implementation
- [x] CLI command implementation
  - [x] CLI structure definition (clap)
  - [x] generate command implementation

## Phase 2: Extensions

- [x] Actual GitHub data fetching (gh command execution)
  - [x] CommandExecutor trait definition
  - [x] GraphQL API implementation
  - [x] Testing with MockCommandExecutor
- [x] Local document reading
  - [x] DocumentContent entity
  - [x] DocumentRepository trait
  - [x] LocalFileDocumentRepository implementation
  - [x] Add document information to Report entity
  - [x] Add document section to Markdown output
- [x] Multiple department support
  - [x] ReportGenerator service implementation
  - [x] Multiple department loop processing
  - [x] Department filter functionality
  - [x] Fiscal year period calculation
  - [x] Integration into main.rs
- [x] JSON/HTML format output
  - [x] JsonOutputRepository implementation
  - [x] HtmlOutputRepository implementation
  - [x] Add file extension parameter to ReportGenerator
  - [x] Format selection functionality in main.rs
- [x] Theme-based summary by Conventional Commits
  - [x] CommitTheme value object implementation
  - [x] Add theme_summary field to Report entity
  - [x] Add Commit Themes section to Markdown output
  - [x] Add Commit Themes section to HTML output
  - [x] Verify theme_summary is included in JSON output (automatic via serde)
  - [x] Actual commit message fetching (Phase 2 full implementation: basic features)
    - [x] Commit entity implementation (SHA, message, author, date, repository name)
    - [x] Add `fetch_commits()` method to GitHubRepository trait
    - [x] GraphQL query extension (pagination support)
    - [x] Commit fetching implementation in GhCommandRepository (recursive pagination)
    - [x] MockCommandExecutor improvement (multiple response support)
    - [x] Add theme_summary building process in ReportGenerator
    - [x] Create tests (commit fetching, pagination, theme-based summary)
  - [x] Phase 2 full implementation: optimization features ✅
    - [x] Progress display functionality (ProgressReporter trait)
      - [x] ProgressReporter trait definition
      - [x] StdoutProgressReporter implementation
      - [x] NoOpProgressReporter implementation
      - [x] Integration into GhCommandRepository
    - [x] Enhanced error handling (API rate limit errors, retry processing)
      - [x] RetryConfig and with_retry function implementation
      - [x] Retry logic with exponential backoff
      - [x] API rate limit error detection and automatic retry
      - [x] GraphQL error handling (user data fetching when organization is absent)
    - [x] Cache functionality (JSON storage in ~/.cache/nenpo/)
      - [x] CommitCache trait definition
      - [x] FileCache implementation (~/.cache/nenpo/)
      - [x] NoOpCache implementation
      - [x] Integration into GhCommandRepository
      - [x] FileCache activation in main.rs
      - [x] Verify fast loading on cache hit
    - [ ] Parallel processing optimization (tokio async processing) ※On hold
    - [x] Integration testing (89.51% coverage achieved, exceeded 80% goal) ✅
      - [x] Coverage measurement with cargo-llvm-cov
      - [x] All 57 tests passed
      - [x] All pre-commit checks passed
    - [x] Verification with actual GitHub data ✅
      - [x] Successfully fetched 849 commits (pagination verification)
      - [x] Progress display functionality verification
      - [x] Cache functionality verification (faster on 2nd run)
      - [x] Output verification in all formats: Markdown/JSON/HTML
      - [x] Conventional Commits theme aggregation verification
    - [x] Notes: API limits (5,000 req/h authenticated), memory consumption for large projects

## Phase 3: Pagination of Commit History Within Repositories (ADR-003)

**Background**: Only fetching up to 100 commits from each repository, causing commit loss (voyagegroup/ecnavi has 646 commits but only 100 fetched)

**Related ADR**: [ADR-003](adrs/adr-003-paginate-commits-within-repositories.md)

### Phase 3.1: Design and Data Structure Preparation ✅

- [x] Verify pageInfo added to CommitHistoryConnectionDetailed
- [x] Verify PageInfo struct reusability
- [x] GraphQL response structure analysis
  - [x] Test single repository commit history fetch query
  - [x] Verify pageInfo and after parameter behavior

### Phase 3.2: GraphQL Query Modification ✅

- [x] Decide on build_commits_query() modification plan
  - [x] Option B: Create build_repo_commits_query() (for single repository)
- [x] Implement query with selected option
  - [x] Create build_repo_commits_query()
  - [x] Create build_repositories_query()
  - [x] Create parse_repo_commits_response()
  - [x] Create parse_repositories_response()
- [x] Test query (manually verify with gh api graphql)

### Phase 3.3: Core Logic Implementation ✅

- [x] Modify fetch_commits() method
  - [x] Implement repository list fetch loop (outer)
  - [x] Add pagination loop for each repository's commit history (inner)
  - [x] Implement nested loops
  - [x] Error handling (utilize existing with_retry)
- [x] Implement new parse functions
- [x] Progress reporting (utilize existing ProgressReporter)

### Phase 3.4: Add Tests ✅

- [x] Add unit tests
  - [x] Test pagination within repository commits
  - [x] Test repositories with over 100 commits
  - [x] Test nested pagination
  - [x] Test when pageInfo hasNextPage is false
- [x] Modify existing tests (adapt 2 tests to new implementation)
- [x] Verify test coverage (88.42%, exceeded 80% goal)

### Phase 3.5: Integration Testing and Verification ✅

- [x] Clear cache
- [x] Test execution with voyagegroup organization
  - [x] Fetch all 646 commits from ecnavi repository (perfect match with expected value)
  - [x] Fetch 112 commits from ecnavi-enquete-app (previously only 100)
  - [x] Total across all repositories: 887 commits (previously 329 commits, 2.7x increase)
- [x] Handle empty repositories (skip ict-nessus, lakebi_common, rmh-task)
- [x] Measure performance
  - [x] API calls: ~300
  - [x] Execution time: 8 minutes 4 seconds
  - [x] Rate limiting: No issues

### Phase 3.6: Documentation and Cleanup ✅

- [x] Update ADR-003 status to "Accepted"
- [x] Update ADR-003 results section with actual data
- [x] Document implementation learnings in ADR
- [x] Mark Phase 3 as completed

### Technical Considerations

#### Performance Optimization (Future Extensions)

- [ ] Consider parallel processing
  - [ ] Parallel fetching of each repository using rayon
  - [ ] Balance with API rate limits
- [ ] Consider incremental caching
  - [ ] Cache storage per repository
  - [ ] Implement differential fetching

#### Error Handling

- [ ] Verify retry mechanism
  - [ ] Retry behavior in nested loops
  - [ ] Handle partial failures (errors in some repositories)
- [ ] Timeout handling
  - [ ] Timeout countermeasures for repositories with large commit counts

#### Quality Assurance

- [ ] Edge case testing
  - [ ] Repositories with 0 commits
  - [ ] Repositories where defaultBranchRef is null
  - [ ] Repositories with exactly 100 commits
- [ ] Memory usage verification
  - [ ] Memory consumption with large commit counts
  - [ ] Consider streaming processing (if needed)

### Completion Criteria ✅

- [x] All tests pass (62 tests)
- [x] All 646 commits can be fetched from voyagegroup/ecnavi (646 commits fetched)
- [x] Your Commits in report shows accurate numbers (887 commits)
- [x] Performance within acceptable range (8 min 4 sec, no rate limit issues)
- [x] Test coverage maintained above 80% (88.42%)
- [x] ADR-003 status is "Accepted"
- [x] Documentation updates completed
