# ADR-003: Pagination Implementation for Commit History Within Repositories

## Status

- [x] Proposed
- [x] Accepted (2025-12-26)
- [ ] Deprecated

## Context

### Background

After implementing the author filter in ADR-002, we could fetch only commits by specific users. However, the following issue was discovered in actual operation:

### Problem

**Missing commits** are occurring:

- `voyagegroup/ecnavi` repository: Has 646 commits but only 100 are fetched
- `voyagegroup/ecnavi-enquete-app` repository: Truncated at 100 commits
- Total: Only 329 commits fetched (actually much more)

### Root Cause

In the current implementation, when fetching commit history for each repository:

```graphql
history(first: 100, since: "...", until: "...", author: {...}) {
  nodes { ... }
}
```

- Only fetches maximum 100 commits with `first: 100`
- While the repository list itself is paginated, **commit history within each repository is not paginated**
- As a result, for repositories with more than 100 commits, the remaining commits are not fetched

### Impact Scope

- connect0459's actual results: 329 commits fetched from voyagegroup organization → Actually 646+ commits (from ecnavi alone)
- Missing commits in large repositories
- Decreased accuracy of annual reports

## Decision

### 1. Paginate Commit History Within Each Repository

Fetch `pageInfo` in each repository's `history` query and implement pagination with nested loops.

#### Structure Before Fix

```rust
loop {
    // Repository pagination
    let query = build_commits_query(...);
    let repos = fetch_and_parse(...);

    for repo in repos {
        // Fetch up to 100 commits from each repository (no pagination)
        commits.extend(repo.commits);
    }

    if !has_next_repo_page { break; }
}
```

#### Structure After Fix

```rust
loop {
    // Repository pagination
    let repos = fetch_repositories(...);

    for repo in repos {
        // Paginate commit history within each repository
        loop {
            let commits = fetch_repo_commits(repo, cursor);
            all_commits.extend(commits);

            if !has_next_commit_page { break; }
        }
    }

    if !has_next_repo_page { break; }
}
```

### 2. GraphQL Query Modification

#### Before Fix

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

#### After Fix

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

### 3. Implementation Strategy

#### 3.1 Architecture

**Two-level pagination**:

1. **Outer loop**: Repository list pagination (existing)
2. **Inner loop**: Commit history pagination within each repository (new)

#### 3.2 Data Structure Changes

Add `pageInfo` to **CommitsRepository** structure:

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
    page_info: PageInfo,  // existing
    nodes: Vec<CommitNode>,
}
```

#### 3.3 Query Builder Modification

Add **query for fetching commits per repository**:

```rust
fn build_repo_commits_query(
    org_or_user: &str,
    repo_name: &str,
    from: NaiveDate,
    to: NaiveDate,
    author_id: Option<&str>,
    after_cursor: Option<&str>,
) -> String {
    // Query to fetch commit history for a single repository
}
```

**Or, extend existing query** to add after parameter to each repository's history.

#### 3.4 Modify fetch_commits() Method

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

    // Check cache
    if let Some(ref cache) = self.cache {
        if let Some(cached) = cache.get(org_or_user, from, to, author)? {
            return Ok(cached);
        }
    }

    let mut all_commits = Vec::new();
    let mut repo_cursor: Option<String> = None;

    // Outer loop: Repository pagination
    loop {
        let repos_query = build_repos_query(org_or_user, repo_cursor.as_deref());
        let repos = fetch_and_parse_repos(&repos_query)?;

        // Process each repository
        for repo in repos.nodes {
            let mut commit_cursor: Option<String> = None;

            // Inner loop: Paginate commit history within each repository
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

    // Save to cache
    if let Some(ref cache) = self.cache {
        cache.set(org_or_user, from, to, author, &all_commits)?;
    }

    Ok(all_commits)
}
```

### 4. Progress Reporting Improvement

Report progress per repository:

```rust
self.progress_reporter.report_repo_progress(repo_name, commit_count);
```

### 5. Performance Considerations

#### Increased API Call Count

- **Before fix**: Repository pages × 1 call
- **After fix**: Repository pages × (commit pages per repository)

**Example**: voyagegroup organization (764 repositories), ecnavi repository (646 commits = 7 pages)

- Before fix: 8 pages (100 repositories each)
- After fix: 8 pages (repositories) + 7 pages (ecnavi commits) + α (other repositories)

#### Rate Limit Countermeasures

- Utilize retry mechanism (existing with_retry)
- Consider adding delays if necessary

## Consequences

### Expected Effects

1. **Complete commit fetching**: Fetch all commits without missing any
2. **Accurate statistics**: Improved annual report accuracy
3. **Scalability**: Handle any number of commits

### Results

Commit counts for connect0459 user (Jan 1, 2025 - Dec 31, 2025):

| Item | Before Fix | After Fix | Increase |
| :--- | :--- | :--- | :--- |
| voyagegroup total | 329 | **887** | **+169.6%** |
| ecnavi | 100 | **646** | **+546.0%** |
| ecnavi-enquete-app | 100 | **112** | **+12.0%** |

#### Key Improvements

1. **Complete commit fetching**: 646 commits from ecnavi repository (perfect match with expected value)
2. **Significant increase**: voyagegroup organization 329 → 887 commits (2.7x)
3. **Empty repository handling**: Properly skip repositories without default branch
4. **Performance**: Completed in 8 minutes 4 seconds (~300 API calls)

## Implementation Learnings

### Technical Challenges and Solutions

1. **GraphQL Query Separation**
   - Initially considered extending existing `build_commits_query()`
   - Finally created two new queries:
     - `build_repositories_query()`: Fetch repository list only
     - `build_repo_commits_query()`: Fetch commits for single repository
   - Reason: Each repository needs individual `after` cursor

2. **Handling Empty Repositories**
   - Problem: Error occurs for repositories with `null` `defaultBranchRef`
   - Solution: Return empty commit list with `Option` pattern matching and output warning

   ```rust
   let Some(branch_ref) = repository.default_branch_ref else {
       eprintln!("⚠ Skipping {}/{}: No default branch", ...);
       return Ok((Vec::new(), PageInfo { ... }));
   };
   ```

3. **Test Modifications**
   - Existing tests didn't support new two-level pagination
   - Mock two responses per test (repository list + commits)
   - Added two new tests:
     - `can_test_pagination_within_repository_commits()`
     - `can_test_nested_pagination()`

4. **PageInfo Structure Extension**
   - Added `Clone` trait to facilitate data passing

### Performance and API Usage

- **API call count**: ~300 calls (for voyagegroup organization)
  - Repository list fetch: ~8 times (100 repositories each)
  - Commit fetch per repository: Many (depending on commit count)
- **Execution time**: 8 minutes 4 seconds
- **Rate limiting**: No issues (existing retry mechanism effective)

### Coverage and Test Quality

- **Test coverage**: 88.42% (exceeds 80% target)
- **Test count**: 62 tests (2 added)
- **All tests passing**: ✅

## References

- [GitHub GraphQL API - CommitHistoryConnection](https://docs.github.com/en/graphql/reference/objects#commithistoryconnection)
- [GitHub GraphQL API - PageInfo](https://docs.github.com/en/graphql/reference/objects#pageinfo)
- [ADR-002: Feature to Fetch Commits by Specific User](./adr-002-filter-commits-by-author.md)

## Related File Paths

### Initial Implementation (2025-12-26)

#### Infrastructure Layer

- `src/infrastructure/github/gh_command_repository.rs` (modified)
  - Modified build_commits_query() (added pageInfo and after parameter to history)
  - Or added build_repo_commits_query() (query for single repository)
  - Modified fetch_commits() (implemented two-level pagination)
  - Modified parse_commits_response() (pageInfo support)
  - Modified CommitHistoryConnectionDetailed structure (added pageInfo)

#### Tests

- `src/infrastructure/github/gh_command_repository.rs` (tests module)
  - Test pagination within repository commits
  - Test repositories with over 100 commits
  - Test nested pagination

#### Documentation

- `docs/adrs/adr-003-paginate-commits-within-repositories.md` (new)

## Future Extensibility

### 1. Parallel Processing

Parallelize commit fetching for each repository to improve performance:

```rust
use rayon::prelude::*;

repos.par_iter().map(|repo| {
    fetch_all_commits_for_repo(repo)
}).flatten().collect()
```

### 2. Incremental Caching

Save cache per repository, fetch only differences next time:

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

### 3. Detailed Progress Bar

Display progress per repository:

```text
Fetching commits for voyagegroup...
  [ecnavi] 100/646 commits fetched...
  [ecnavi] 200/646 commits fetched...
  ...
```
