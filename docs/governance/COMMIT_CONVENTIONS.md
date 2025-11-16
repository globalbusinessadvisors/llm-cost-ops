# Commit Message Conventions

This document describes the commit message conventions used in LLM-CostOps. We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

## Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type

The type describes the kind of change being made:

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, etc.)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes that affect the build system or external dependencies
- **ci**: Changes to CI configuration files and scripts
- **chore**: Other changes that don't modify src or test files
- **revert**: Reverts a previous commit

### Scope

The scope describes what part of the codebase is affected. Common scopes include:

- **api**: REST API server
- **auth**: Authentication and authorization
- **cli**: Command-line interface
- **pricing**: Pricing models and calculations
- **forecasting**: Forecasting engine and algorithms
- **storage**: Database and storage layer
- **export**: Export and reporting system
- **observability**: Metrics, tracing, and logging
- **k8s**: Kubernetes deployment and configuration
- **ingestion**: Data ingestion and processing
- **domain**: Domain models and business logic
- **engine**: Cost calculation engine
- **deps**: Dependency updates

### Subject

The subject is a brief description of the change:

- Use the imperative, present tense: "change" not "changed" nor "changes"
- Don't capitalize the first letter
- No period (.) at the end
- Limit to 50 characters

### Body

The body should include:

- Motivation for the change
- Contrast with previous behavior
- Implementation details (if complex)

Use the imperative, present tense. Wrap at 72 characters.

### Footer

The footer should contain:

- **Breaking Changes**: Start with `BREAKING CHANGE:` followed by description
- **Issue References**: Use keywords like `Fixes #123`, `Closes #456`, `Refs #789`
- **Co-authors**: `Co-authored-by: Name <email@example.com>`

## Examples

### Feature Addition

```
feat(pricing): add tiered volume pricing support

Add support for volume-based tiered pricing that applies different
rates based on monthly token consumption thresholds.

This enables providers like OpenAI that offer volume discounts to be
accurately modeled in the cost calculation engine.

Closes #234
```

### Bug Fix

```
fix(auth): resolve JWT token expiration validation

Fix issue where expired JWT tokens were being accepted due to
incorrect timezone handling in the validation logic.

The validator was comparing UTC timestamps with local time,
causing tokens to appear valid past their expiration.

Fixes #567
```

### Breaking Change

```
feat(api): redesign cost query API for better performance

Replace the existing cost query endpoint with a new GraphQL-based
API that supports efficient filtering and aggregation.

The REST endpoints `/api/v1/costs/query` and `/api/v1/costs/summary`
are deprecated and will be removed in v2.0.0.

BREAKING CHANGE: The REST cost query endpoints are deprecated.
Migrate to the new GraphQL API at `/api/graphql`.

Migration guide: docs/migration/v1-to-v2.md

Closes #890
```

### Documentation

```
docs(api): update REST API endpoint documentation

Add examples for all query parameters and response formats.
Clarify authentication requirements for each endpoint.

Fixes #123
```

### Refactoring

```
refactor(storage): optimize database connection pooling

Extract connection pool configuration into a dedicated module
and implement connection health checks.

No functional changes to the public API.
```

### Performance Improvement

```
perf(forecasting): optimize ARIMA model training

Reduce training time by 60% by using parallel computation
for seasonal decomposition and parameter estimation.

Benchmark results:
- Before: 45s for 10,000 data points
- After: 18s for 10,000 data points

Closes #456
```

### Test Addition

```
test(pricing): add integration tests for tiered pricing

Add comprehensive test coverage for volume-based tiered
pricing including edge cases and threshold boundaries.

Increases coverage from 75% to 92% for pricing module.
```

### Dependency Update

```
chore(deps): update tokio to 1.37.0

Update async runtime to latest version for performance
improvements and security fixes.

Also updates related dependencies:
- tokio-util: 0.7.10
- tokio-stream: 0.1.15
```

### CI/CD Changes

```
ci: add code coverage reporting to GitHub Actions

Configure Tarpaulin to generate coverage reports and
upload to Codecov on every PR and merge to main.

Minimum coverage threshold set to 80%.
```

### Revert

```
revert: feat(api): add GraphQL support

This reverts commit a1b2c3d4e5f6.

GraphQL integration introduced breaking changes that need
more discussion before implementation.
```

## Multi-line Commits

For complex changes, provide detailed explanation in the body:

```
feat(forecasting): implement ensemble forecasting models

Add support for combining multiple forecasting models (ARIMA,
exponential smoothing, and linear regression) using weighted
averaging to improve prediction accuracy.

The ensemble approach automatically weights models based on
their historical accuracy on similar data patterns. This
typically provides 15-20% better accuracy than single models.

Implementation details:
- Models run in parallel using Tokio tasks
- Weights are calculated using exponential decay of errors
- Results are cached for 1 hour to improve performance

Configuration:
  forecasting:
    ensemble:
      enabled: true
      models: [arima, ets, linear]
      weight_decay: 0.1

Performance impact:
- Training time: +40% (parallelized)
- Prediction time: +10%
- Accuracy improvement: +18% (MAPE)

Closes #234
Refs #567
```

## Scopes Reference

### Core Components

- **api**: REST API, GraphQL, gRPC services
- **cli**: Command-line interface and tools
- **domain**: Domain models and business logic
- **engine**: Cost calculation and processing engine

### Features

- **pricing**: Pricing models and calculations
- **forecasting**: Time-series forecasting
- **auth**: Authentication and authorization
- **export**: Export and reporting
- **ingestion**: Data ingestion and streaming

### Infrastructure

- **storage**: Database, cache, and persistence
- **observability**: Metrics, tracing, logging
- **k8s**: Kubernetes deployment
- **ci**: CI/CD pipelines
- **build**: Build configuration

### Other

- **deps**: Dependency updates
- **docs**: Documentation
- **test**: Testing infrastructure
- **chore**: Maintenance tasks

## Commit Message Best Practices

### Do

- Write clear, concise subject lines
- Explain **why** not just **what** in the body
- Reference related issues
- Keep commits atomic (one logical change per commit)
- Use imperative mood ("add feature" not "added feature")
- Separate subject from body with a blank line

### Don't

- Mix unrelated changes in a single commit
- Write vague subjects like "fix bug" or "update code"
- Omit issue references for bug fixes
- Forget to document breaking changes
- Use past tense ("fixed", "added")
- Exceed 72 characters per line in body

## Tools and Automation

### Commit Message Validation

We use Git hooks to validate commit messages:

```bash
# Install commit message hook
cp scripts/commit-msg .git/hooks/commit-msg
chmod +x .git/hooks/commit-msg
```

### Commitizen

For interactive commit message creation:

```bash
# Install commitizen
npm install -g commitizen cz-conventional-changelog

# Create commit interactively
git cz
```

### Commitlint

Automated commit message linting in CI:

```yaml
# .commitlintrc.yml
extends:
  - '@commitlint/config-conventional'
rules:
  type-enum:
    - 2
    - always
    - [feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert]
  scope-enum:
    - 2
    - always
    - [api, auth, cli, pricing, forecasting, storage, export, observability, k8s, ingestion, domain, engine, deps]
  subject-max-length:
    - 2
    - always
    - 50
  body-max-line-length:
    - 2
    - always
    - 72
```

## Changelog Generation

Commit messages are used to automatically generate the CHANGELOG:

```bash
# Generate changelog
conventional-changelog -p angular -i CHANGELOG.md -s

# Generate changelog for specific version
conventional-changelog -p angular -i CHANGELOG.md -s -r 0
```

### Changelog Sections

Based on commit types:

- **Features**: `feat` commits
- **Bug Fixes**: `fix` commits
- **Performance Improvements**: `perf` commits
- **Breaking Changes**: Commits with `BREAKING CHANGE:` footer
- **Documentation**: `docs` commits (in separate section)

## Version Bumping

Commit types determine version bumps (semantic versioning):

- **MAJOR** (breaking): Commits with `BREAKING CHANGE:`
- **MINOR** (feature): `feat` commits
- **PATCH** (fix): `fix` commits

Example:

```
# Results in PATCH bump (0.1.0 -> 0.1.1)
fix(api): correct response status codes

# Results in MINOR bump (0.1.0 -> 0.2.0)
feat(forecasting): add exponential smoothing

# Results in MAJOR bump (0.1.0 -> 1.0.0)
feat(api): redesign query API

BREAKING CHANGE: Old REST endpoints removed
```

## Additional Resources

- [Conventional Commits Specification](https://www.conventionalcommits.org/)
- [Semantic Versioning](https://semver.org/)
- [How to Write a Git Commit Message](https://chris.beams.io/posts/git-commit/)
- [Angular Commit Message Guidelines](https://github.com/angular/angular/blob/main/CONTRIBUTING.md#commit)

## Questions?

If you have questions about commit conventions, ask in:

- [GitHub Discussions](https://github.com/yourusername/llm-cost-ops/discussions)
- [Discord #development](https://discord.gg/example)
