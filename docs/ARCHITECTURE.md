# ARCHITECTURE

## Overview

This project follows the **Onion Architecture** pattern to ensure clean separation of concerns, testability, and long-term maintainability.

## Layers

### 1. Domain Layer (Core)

The innermost layer containing business logic and entities.

**Responsibilities:**

- Define domain entities and value objects
- Define repository traits (interfaces)
- Implement pure business logic
- No dependencies on outer layers

**Modules:**

- `domain::entities`: Domain entities (e.g., `Department`, `Report`, `GitHubActivity`)
- `domain::value_objects`: Value objects (e.g., `Period`, `FiscalYear`)
- `domain::repositories`: Repository trait definitions
- `domain::services`: Domain services for complex business logic

**Example:**

```rust
// domain/entities/department.rs
pub struct Department {
    name: String,
    fiscal_year_start_month: u32,
    github_organizations: Vec<String>,
}
```

### 2. Application Layer

Orchestrates use cases and application logic.

**Responsibilities:**

- Implement use cases (e.g., "Generate Report")
- Coordinate between domain and infrastructure
- Transaction management
- Business workflow orchestration

**Modules:**

- `application::usecases`: Use case implementations
- `application::services`: Application services

**Example:**

```rust
// application/usecases/generate_report.rs
pub struct GenerateReportUseCase {
    config_repository: Box<dyn ConfigRepository>,
    github_repository: Box<dyn GitHubRepository>,
    output_repository: Box<dyn OutputRepository>,
}
```

### 3. Infrastructure Layer

Implements interfaces to external systems.

**Responsibilities:**

- Implement repository traits defined in domain layer
- Handle external API calls (GitHub via `gh` command)
- File I/O operations
- Configuration file parsing

**Modules:**

- `infrastructure::config`: TOML configuration file handling
- `infrastructure::github`: GitHub data fetching via `gh` command
- `infrastructure::output`: File output (Markdown, JSON, HTML)

**Example:**

```rust
// infrastructure/config/toml_config_repository.rs
pub struct TomlConfigRepository;

impl ConfigRepository for TomlConfigRepository {
    fn load(&self, path: &Path) -> Result<Config> {
        // TOML parsing implementation
    }
}
```

### 4. Presentation Layer (CLI)

User-facing interface.

**Responsibilities:**

- Parse command-line arguments
- Validate user input
- Display results and error messages
- Delegate to application layer

**Modules:**

- `presentation::cli`: CLI command definitions using `clap`

**Example:**

```rust
// presentation/cli/mod.rs
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}
```

## Dependency Rule

- **Inward dependencies only**: Outer layers can depend on inner layers, but NOT vice versa
- **Dependency Inversion**: Outer layers implement interfaces defined by inner layers
- **No leakage**: Domain layer has zero knowledge of infrastructure or presentation

## Directory Structure

```text
src/
├── main.rs                     # Entry point
├── domain/
│   ├── mod.rs
│   ├── entities/               # Domain entities
│   │   ├── mod.rs
│   │   ├── department.rs
│   │   ├── report.rs
│   │   └── github_activity.rs
│   ├── value_objects/          # Value objects
│   │   ├── mod.rs
│   │   ├── period.rs
│   │   └── fiscal_year.rs
│   ├── repositories/           # Repository traits
│   │   ├── mod.rs
│   │   ├── config_repository.rs
│   │   ├── github_repository.rs
│   │   └── output_repository.rs
│   └── services/               # Domain services
│       └── mod.rs
├── application/
│   ├── mod.rs
│   ├── usecases/               # Use cases
│   │   ├── mod.rs
│   │   └── generate_report.rs
│   └── services/               # Application services
│       └── mod.rs
├── infrastructure/
│   ├── mod.rs
│   ├── config/                 # Configuration file handling
│   │   ├── mod.rs
│   │   └── toml_config_repository.rs
│   ├── github/                 # GitHub integration
│   │   ├── mod.rs
│   │   └── gh_command_repository.rs
│   └── output/                 # Output generation
│       ├── mod.rs
│       ├── markdown_output_repository.rs
│       ├── json_output_repository.rs
│       └── html_output_repository.rs
└── presentation/
    ├── mod.rs
    └── cli/                    # CLI interface
        ├── mod.rs
        └── commands.rs
```

## Testing Strategy

- **Domain Layer**: Pure unit tests with real objects (Detroit School TDD)
- **Application Layer**: Use case tests with mock repositories
- **Infrastructure Layer**: Integration tests with real external systems
- **Presentation Layer**: CLI integration tests

**Coverage Goal**: 80%+ overall, with focus on domain and application layers

## Key Principles

1. **Rich Domain Objects**: Entities contain both data and behavior
2. **No Getters/Setters**: Use methods that express intent (e.g., `name()` instead of `getName()`)
3. **TDD First**: Write tests before implementation (Red → Green → Refactor)
4. **Mock Only Boundaries**: Mock external systems only, use real objects for internal logic
5. **Evergreen Tests**: Tests should represent business requirements and remain stable
