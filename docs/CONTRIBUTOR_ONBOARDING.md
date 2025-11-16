# Contributor Onboarding Guide

Welcome to the LLM-CostOps contributor community! This guide will help you get started contributing to the project.

## Table of Contents

1. [Before You Start](#before-you-start)
2. [Development Environment Setup](#development-environment-setup)
3. [Understanding the Codebase](#understanding-the-codebase)
4. [Making Your First Contribution](#making-your-first-contribution)
5. [Development Workflow](#development-workflow)
6. [Code Review Process](#code-review-process)
7. [Getting Help](#getting-help)
8. [Community Guidelines](#community-guidelines)

## Before You Start

### Read These First

- [ ] [README.md](../README.md) - Project overview
- [ ] [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [ ] [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md) - Community standards
- [ ] [Architecture Documentation](ARCHITECTURE.md) - System design

### Join the Community

- [ ] Star the repository on GitHub
- [ ] Watch the repository for notifications
- [ ] Join our [Discord server](https://discord.gg/example)
- [ ] Follow us on [Twitter](https://twitter.com/llmcostops)
- [ ] Subscribe to [GitHub Discussions](https://github.com/yourusername/llm-cost-ops/discussions)

### Find Your First Issue

Look for issues labeled:

- `good first issue` - Perfect for newcomers
- `help wanted` - Community help needed
- `documentation` - Documentation improvements
- `testing` - Test additions/improvements

## Development Environment Setup

### Step 1: Prerequisites

Install required tools:

```bash
# Rust (required)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Git (required)
# Install via your package manager

# PostgreSQL (optional, for production-like testing)
# macOS
brew install postgresql

# Ubuntu/Debian
sudo apt-get install postgresql postgresql-contrib

# Docker (optional, for containerized testing)
# Follow instructions at https://docs.docker.com/get-docker/
```

### Step 2: Fork and Clone

```bash
# Fork the repository on GitHub (click "Fork" button)

# Clone your fork
git clone https://github.com/YOUR-USERNAME/llm-cost-ops.git
cd llm-cost-ops

# Add upstream remote
git remote add upstream https://github.com/yourusername/llm-cost-ops.git

# Verify remotes
git remote -v
```

### Step 3: Install Development Tools

```bash
# Update Rust toolchain
rustup update

# Install additional components
rustup component add rustfmt clippy

# Install development tools
cargo install cargo-watch      # Auto-rebuild on changes
cargo install cargo-tarpaulin  # Code coverage
cargo install sqlx-cli --no-default-features --features rustls,postgres,sqlite
```

### Step 4: Setup Database

Choose either SQLite (easier) or PostgreSQL (production-like):

#### Option A: SQLite (Recommended for beginners)

```bash
# Create database
sqlx database create --database-url sqlite:cost-ops.db

# Run migrations
sqlx migrate run --database-url sqlite:cost-ops.db

# Set environment variable
echo "DATABASE_URL=sqlite:cost-ops.db" > .env
```

#### Option B: PostgreSQL

```bash
# Start PostgreSQL
# macOS
brew services start postgresql

# Ubuntu/Debian
sudo service postgresql start

# Create database and user
createdb llm_costops
createuser costops -P  # Enter password when prompted

# Run migrations
export DATABASE_URL="postgresql://costops:password@localhost/llm_costops"
sqlx database create
sqlx migrate run

# Save to .env
echo "DATABASE_URL=postgresql://costops:password@localhost/llm_costops" > .env
```

### Step 5: Build and Test

```bash
# Build the project
cargo build

# Run tests
cargo test

# Check formatting
cargo fmt -- --check

# Run linter
cargo clippy -- -D warnings

# Run in development mode
cargo run -- --help
```

### Step 6: Verify Setup

```bash
# All of these should succeed
cargo build         # ✓ Build succeeds
cargo test          # ✓ All tests pass
cargo fmt -- --check  # ✓ Code is formatted
cargo clippy -- -D warnings  # ✓ No lint warnings
```

If all checks pass, you're ready to start contributing! 🎉

## Understanding the Codebase

### Project Structure

```
llm-cost-ops/
├── src/
│   ├── domain/          # Core business logic and models
│   ├── engine/          # Cost calculation engine
│   ├── storage/         # Database and persistence
│   ├── api/             # REST API server
│   ├── auth/            # Authentication & authorization
│   ├── cli/             # Command-line interface
│   ├── forecasting/     # Time-series forecasting
│   ├── export/          # Export and reporting
│   ├── observability/   # Metrics, tracing, logging
│   ├── ingestion/       # Data ingestion
│   └── bin/             # Binary entry point
├── tests/               # Integration tests
├── benches/             # Performance benchmarks
├── migrations/          # Database migrations (SQLite)
├── migrations_postgres/ # Database migrations (PostgreSQL)
├── k8s/                 # Kubernetes deployment
├── docs/                # Documentation
└── examples/            # Usage examples
```

### Key Concepts

#### 1. Domain Models (`src/domain/`)

Core business entities:

- `UsageRecord` - LLM usage data from providers
- `CostModel` - Pricing structures
- `CostRecord` - Calculated costs
- `ForecastResult` - Forecasting predictions

#### 2. Cost Calculation (`src/engine/`)

Engine for calculating costs:

- `CostCalculator` - Main calculation logic
- `PricingEngine` - Applies pricing models
- `Aggregator` - Aggregates costs by dimensions

#### 3. Storage Layer (`src/storage/`)

Database abstraction:

- `Repository` - Data access trait
- `SqliteRepository` - SQLite implementation
- `PostgresRepository` - PostgreSQL implementation

#### 4. API Layer (`src/api/`)

HTTP endpoints:

- `routes/` - API route handlers
- `middleware/` - Auth, CORS, rate limiting
- `models/` - Request/response models

### Code Patterns

#### Error Handling

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CostError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Invalid pricing model")]
    InvalidPricing,
}

pub fn calculate() -> Result<Decimal, CostError> {
    // Use ? operator
    let data = fetch_data()?;
    Ok(process(data))
}
```

#### Async/Await

```rust
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let result = fetch_data().await?;
    process(result).await?;
    Ok(())
}
```

#### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculation() {
        let result = calculate(100, Decimal::from(10));
        assert_eq!(result, Decimal::from(1000));
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = fetch_data().await.unwrap();
        assert!(!result.is_empty());
    }
}
```

## Making Your First Contribution

### Step 1: Choose an Issue

Find a good first issue:

1. Go to [Issues](https://github.com/yourusername/llm-cost-ops/issues)
2. Filter by `good first issue` label
3. Read the issue description
4. Comment that you'd like to work on it

### Step 2: Create a Branch

```bash
# Update your fork
git checkout main
git fetch upstream
git merge upstream/main

# Create feature branch
git checkout -b feature/your-feature-name
```

### Step 3: Make Changes

Follow the development workflow:

```bash
# Make changes to code
vim src/some/file.rs

# Run tests frequently
cargo test

# Format code
cargo fmt

# Check for lint issues
cargo clippy
```

### Step 4: Commit Changes

Use [Conventional Commits](governance/COMMIT_CONVENTIONS.md):

```bash
# Stage changes
git add .

# Commit with conventional commit message
git commit -m "feat(cli): add new export command

Add support for exporting cost data in CSV format.
Includes comprehensive tests and documentation.

Fixes #123"
```

### Step 5: Push and Create PR

```bash
# Push to your fork
git push origin feature/your-feature-name

# Create pull request on GitHub
# Use the PR template and fill it out completely
```

## Development Workflow

### Daily Workflow

```bash
# Start of day: Update your fork
git checkout main
git pull upstream main
git push origin main

# Create feature branch
git checkout -b feature/my-feature

# Development cycle
while working:
    # Write code
    vim src/...

    # Run tests
    cargo test

    # Check formatting
    cargo fmt

    # Fix lint issues
    cargo clippy --fix

# Commit frequently
git commit -am "feat: add feature X"

# End of day: Push to your fork
git push origin feature/my-feature
```

### Testing Workflow

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test '*'

# Run benchmarks
cargo bench

# Generate coverage report
cargo tarpaulin --out Html
```

### Code Quality Checks

Before submitting PR:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run linter
cargo clippy -- -D warnings

# Run all tests
cargo test

# Check documentation
cargo doc --no-deps --open
```

## Code Review Process

### What to Expect

1. **Initial Review** (1-3 days): A maintainer will review your PR
2. **Feedback**: You may receive feedback and change requests
3. **Iteration**: Address feedback and push updates
4. **Approval**: Once approved, PR will be merged
5. **Merge**: Your contribution is now part of the project!

### Addressing Feedback

```bash
# Make requested changes
vim src/file.rs

# Test changes
cargo test

# Commit and push
git commit -am "fix: address review feedback"
git push origin feature/my-feature
```

### After PR is Merged

```bash
# Update your fork
git checkout main
git pull upstream main
git push origin main

# Delete feature branch
git branch -d feature/my-feature
git push origin --delete feature/my-feature
```

## Getting Help

### Where to Ask

- **GitHub Discussions**: General questions
- **Discord #help**: Real-time help
- **Issue Comments**: Specific to an issue
- **PR Comments**: Specific to your PR

### Tips for Getting Help

1. **Search First**: Check if your question has been answered
2. **Be Specific**: Provide context and details
3. **Share Code**: Include relevant code snippets
4. **Show Effort**: Explain what you've tried
5. **Be Patient**: Maintainers are volunteers

### Common Questions

**Q: How do I find an issue to work on?**
A: Look for issues labeled `good first issue` or `help wanted`.

**Q: Can I work on an issue that's already assigned?**
A: No, please find an unassigned issue or ask if you can help.

**Q: How long should I wait for PR review?**
A: Usually 1-3 business days. Ping after a week if no response.

**Q: What if my PR is rejected?**
A: Don't worry! Learn from feedback and try again.

**Q: Can I work on multiple issues?**
A: Yes, but complete one before starting another.

## Community Guidelines

### Be Respectful

- Treat everyone with respect
- Be welcoming to newcomers
- Assume good intentions
- Provide constructive feedback

### Be Professional

- Use professional language
- Keep discussions on-topic
- Respect others' time
- Follow the Code of Conduct

### Be Collaborative

- Help others when you can
- Share knowledge
- Celebrate others' contributions
- Work together on solutions

## Next Steps

### Learn More

- [ ] Read [Architecture Documentation](ARCHITECTURE.md)
- [ ] Explore [Examples](../examples/)
- [ ] Review [API Documentation](https://docs.rs/llm-cost-ops)
- [ ] Check [Roadmap](../README.md#roadmap)

### Grow Your Skills

- Start with documentation improvements
- Move to bug fixes
- Then tackle features
- Eventually become a reviewer

### Get Recognized

- Contribute regularly
- Help review PRs
- Assist other contributors
- Share knowledge in discussions

## Resources

### Documentation

- [CONTRIBUTING.md](../CONTRIBUTING.md)
- [Architecture Guide](ARCHITECTURE.md)
- [Commit Conventions](governance/COMMIT_CONVENTIONS.md)
- [Versioning Policy](governance/VERSIONING.md)

### Tools

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [SQLx Documentation](https://docs.rs/sqlx)

### Community

- [Discord](https://discord.gg/example)
- [GitHub Discussions](https://github.com/yourusername/llm-cost-ops/discussions)
- [Twitter](https://twitter.com/llmcostops)

---

**Welcome aboard! We're excited to have you as a contributor! 🚀**

If you have questions about this guide, please ask in [Discord #onboarding](https://discord.gg/example) or open a [Discussion](https://github.com/yourusername/llm-cost-ops/discussions).
