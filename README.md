# nenpo (Annual Report)

A command-line tool that generates annual reports from GitHub resources and local documents.

## Features

- 📊 **Automatic GitHub Activity Aggregation**: Aggregates commits, PRs, issues, and reviews
- 🏷️ **Conventional Commits Support**: Automatically categorizes commits by theme based on commit messages
- 📝 **Multiple Output Formats**: Supports Markdown, JSON, and HTML formats
- 🗂️ **Multi-Department Support**: Manage multiple organizations/users by department (personal, corporate, etc.)
- ⚡ **Fast Caching**: Quick loading from cache on subsequent runs
- 🔄 **Automatic Retry**: Auto-retry functionality when hitting GitHub API rate limits

## Requirements

- Rust 1.70 or higher
- [GitHub CLI (`gh`)](https://cli.github.com/)
- GitHub account (authenticated)

## Installation

```bash
# Clone the repository
git clone https://github.com/connect0459/nenpo.git
cd nenpo

# Build
cargo build --release

# (Optional) Add to PATH
cp target/release/nenpo ~/.local/bin/
```

## Quick Start

### 1. Create Configuration File

Copy the sample and edit:

```bash
cp nenpo-config.toml.example nenpo-config.toml
```

Edit `nenpo-config.toml`:

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

### 2. Generate Report

```bash
# Generate report for 2025
./target/release/nenpo generate --config nenpo-config.toml --year 2025

# Output as JSON format
./target/release/nenpo generate --config nenpo-config.toml --year 2025 --format json

# Output as HTML format
./target/release/nenpo generate --config nenpo-config.toml --year 2025 --format html
```

### 3. View Report

```bash
# Display Markdown report
cat ./reports/report-Personal\ Projects-2025.md
```

## Usage

### Basic Commands

```bash
# Generate report with default settings
nenpo generate --config nenpo-config.toml

# Specify year
nenpo generate --config nenpo-config.toml --year 2024

# Process specific department only
nenpo generate --config nenpo-config.toml --department "Personal Projects"

# Specify output format
nenpo generate --config nenpo-config.toml --year 2025 --format json
```

### Options

- `--config <PATH>`: Path to configuration file (required)
- `--year <YEAR>`: Target year (fiscal year start month is obtained from configuration file)
- `--department <NAME>`: Process specific department only
- `--format <FORMAT>`: Output format (`markdown`, `json`, `html`)

## Configuration File

### Basic Structure

```toml
# Default fiscal year start month (1-12)
default_fiscal_year_start_month = 4

# Default output format
default_output_format = "markdown"

# Output directory
output_directory = "./reports"

# Department definitions (multiple allowed)
[[departments]]
name = "Personal"
fiscal_year_start_month = 4
github_organizations = ["connect0459"]
local_documents = []

[[departments]]
name = "Corporate"
fiscal_year_start_month = 4
github_organizations = ["voyagegroup"]
local_documents = ["docs/**/*.md"]
```

### Configuration Options

#### Top-Level Settings

- `default_fiscal_year_start_month`: Fiscal year start month (1=January, 4=April, etc.)
- `default_output_format`: Default output format (`markdown`, `json`, `html`)
- `output_directory`: Output directory for reports

#### Department Settings (`[[departments]]`)

- `name`: Department name (used in report filename)
- `fiscal_year_start_month`: Fiscal year start month for this department (overrides top-level setting)
- `github_organizations`: List of GitHub organizations or usernames to target
- `local_documents`: Glob patterns for local documents (currently not implemented)

## Output Formats

### Markdown

Human-readable format. Includes the following information:

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

Program-friendly format:

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

Viewable in web browsers. Generates a visually organized report.

## Cache Functionality

nenpo caches fetched commit information in `~/.cache/nenpo/`.

- **First run**: Fetch data from GitHub (several seconds to minutes)
- **Subsequent runs**: Load from cache (instant)

### Clear Cache

```bash
rm -rf ~/.cache/nenpo/
```

## Conventional Commits

nenpo automatically categorizes commit messages:

| Prefix | Theme | Example |
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
| Others | Other | Commits that don't match above |

## Troubleshooting

### GitHub Authentication Error

```bash
# Check GitHub CLI authentication status
gh auth status

# If not authenticated
gh auth login
```

### API Rate Limit Error

GitHub API has rate limits:

- **Authenticated**: 5,000 requests/hour
- **Unauthenticated**: 60 requests/hour

nenpo automatically retries, but large repositories may take time.

### "organization not found" Error

For personal users, this error is ignored. User data is fetched normally.

## Developer Information

For developers, see [docs/development/](docs/development/).

- [Architecture Design](docs/ARCHITECTURE.md)
- [Testing Strategy](docs/development/testing.md)

## License

MIT License

## Author

connect0459
