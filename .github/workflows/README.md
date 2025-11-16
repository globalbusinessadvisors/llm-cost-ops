# GitHub Actions Workflows Documentation

This directory contains the CI/CD workflows for the LLM Cost Ops platform and its SDKs.

## Table of Contents

- [Workflow Overview](#workflow-overview)
- [Reusable Workflows](#reusable-workflows)
- [SDK Workflows](#sdk-workflows)
- [Automation Workflows](#automation-workflows)
- [Usage Examples](#usage-examples)
- [Configuration](#configuration)

## Workflow Overview

The workflows are organized into three main categories:

1. **Reusable Workflows** (`reusable/`): Modular, language-agnostic workflows that can be called by other workflows
2. **SDK Workflows** (`sdk/`): Specific workflows for each SDK (Python, TypeScript, Rust, Go, Java)
3. **Automation Workflows**: Top-level workflows for release orchestration, dependency updates, and monitoring

## Reusable Workflows

### version-bump.yml

Automatically manages version bumping across different languages.

**Features:**
- Supports Python, TypeScript, Go, Rust, and Java
- Auto-detects version bump type from conventional commits
- Manual override for version bump type (major, minor, patch)
- Dry-run mode for testing

**Usage:**
```yaml
jobs:
  bump-version:
    uses: ./.github/workflows/reusable/version-bump.yml
    with:
      language: 'python'
      working-directory: './sdks/python'
      bump-type: 'auto'  # or 'major', 'minor', 'patch'
```

### changelog-generator.yml

Generates changelogs from git commit history.

**Features:**
- Multiple formats: Keep a Changelog, Conventional Commits, Simple
- Automatic categorization of changes
- Integration with git-cliff for advanced changelog generation
- Auto-commit generated changelog

**Usage:**
```yaml
jobs:
  changelog:
    uses: ./.github/workflows/reusable/changelog-generator.yml
    with:
      language: 'typescript'
      version: 'v1.2.3'
      format: 'conventional'
```

### coverage-report.yml

Generates and publishes code coverage reports.

**Features:**
- Language-specific coverage tools (pytest-cov, Jest, cargo-llvm-cov, etc.)
- Coverage badge generation
- Codecov integration
- Coverage threshold enforcement
- HTML and JSON reports

**Usage:**
```yaml
jobs:
  coverage:
    uses: ./.github/workflows/reusable/coverage-report.yml
    with:
      language: 'rust'
      coverage-threshold: 80
      fail-on-threshold: true
    secrets:
      CODECOV_TOKEN: ${{ secrets.CODECOV_TOKEN }}
```

### security-scan.yml

Comprehensive security scanning for all languages.

**Features:**
- CodeQL analysis
- Semgrep SAST
- Dependency vulnerability scanning
- Secret scanning (Gitleaks, TruffleHog)
- Container image scanning (Trivy, Grype)
- SBOM generation
- OSSF Scorecard

**Usage:**
```yaml
jobs:
  security:
    uses: ./.github/workflows/reusable/security-scan.yml
    with:
      language: 'python'
      enable-codeql: true
      enable-sast: true
    secrets:
      SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
```

### docs-deploy.yml

Builds and deploys documentation.

**Features:**
- API documentation generation (Sphinx, TypeDoc, rustdoc, etc.)
- Multiple deployment targets (GitHub Pages, Netlify, Vercel, S3)
- Static site generator support (Jekyll, Hugo, MkDocs)
- Version-specific documentation

**Usage:**
```yaml
jobs:
  docs:
    uses: ./.github/workflows/reusable/docs-deploy.yml
    with:
      language: 'python'
      deploy-target: 'github-pages'
      generate-api-docs: true
```

### test-matrix.yml

Runs tests across multiple language versions and operating systems.

**Features:**
- Matrix testing across OS (Linux, macOS, Windows)
- Multiple language versions
- Parallel test execution
- Test result aggregation

### lint-quality.yml

Code quality and linting checks.

**Features:**
- Language-specific linters (pylint, ESLint, clippy, etc.)
- Code formatters (black, prettier, rustfmt)
- Type checking
- Import organization

### publish-package.yml

Publishes packages to registries.

**Features:**
- Multi-registry support (PyPI, npm, crates.io, etc.)
- Dry-run mode
- Release artifact creation
- GPG signing

### release-automation.yml

Automates the release process.

**Features:**
- Pre-release validation
- Automated tagging
- GitHub Release creation
- Multi-stage release process

## SDK Workflows

### python-sdk.yml

Workflow for the Python SDK.

**Triggers:**
- Push to Python SDK files
- Pull requests affecting Python SDK
- Manual dispatch

**Jobs:**
- Lint and quality checks
- Test matrix (Python 3.9-3.12)
- Security scanning
- Coverage reporting

### typescript-sdk.yml

Workflow for the TypeScript SDK.

**Triggers:**
- Push to TypeScript SDK files
- Pull requests affecting TypeScript SDK
- Manual dispatch

**Jobs:**
- Lint and quality checks
- Test matrix (Node 18, 20, 22)
- Security scanning
- Coverage reporting

## Automation Workflows

### sdk-release-orchestrator.yml

Orchestrates SDK releases across all languages.

**Features:**
- Auto-detects SDK from git tag pattern (`python-v*`, `typescript-v*`, etc.)
- Version consistency validation
- Automated changelog generation
- Full test suite execution
- Security scanning
- Package publishing
- Documentation deployment
- Cross-SDK compatibility checks
- Release notifications

**Triggers:**
```bash
# Tag-based release
git tag python-v1.2.3
git push origin python-v1.2.3

# Manual release
# Use GitHub Actions UI to trigger with inputs
```

**Tag Pattern:**
- Python: `python-v1.2.3`
- TypeScript: `typescript-v1.2.3`
- Rust: `rust-v1.2.3`
- Go: `go-v1.2.3`
- Java: `java-v1.2.3`

### dependency-updates.yml

Automated dependency updates with security scanning.

**Features:**
- Daily scheduled updates
- Per-language dependency management
- Security vulnerability detection
- Automated PR creation
- Test execution on update PRs
- Auto-merge for patch updates (optional)

**Languages Supported:**
- Python: pip-tools, pip-audit, safety
- TypeScript: npm-check-updates, npm audit
- Rust: cargo-edit, cargo-audit
- Go: go get -u, govulncheck

**Schedule:**
- Runs daily at 2 AM UTC
- Can be manually triggered

### status-dashboard.yml

Generates and maintains project health dashboard.

**Features:**
- Build status badges
- Version badges
- SDK compatibility matrix
- Health metrics
- Automated README updates
- Stale PR detection
- Security issue tracking

**Schedule:**
- Runs every 6 hours
- Updates on push to main
- Updates on PR events

**Generated Artifacts:**
- Status badges (`.github/badges/`)
- Compatibility matrix
- Health reports

## Usage Examples

### Creating a New SDK Release

1. **Prepare the release:**
   ```bash
   # Make your changes and commit them
   git add .
   git commit -m "feat: add new feature"
   ```

2. **Create a release tag:**
   ```bash
   # Tag follows pattern: <sdk>-v<version>
   git tag python-v1.2.3
   git push origin python-v1.2.3
   ```

3. **The orchestrator will automatically:**
   - Validate version consistency
   - Generate changelog
   - Run full test suite
   - Perform security scans
   - Build artifacts
   - Create GitHub release
   - Publish to package registry
   - Deploy documentation
   - Send notifications

### Manual Version Bump

```yaml
# .github/workflows/my-workflow.yml
jobs:
  bump:
    uses: ./.github/workflows/reusable/version-bump.yml
    with:
      language: 'python'
      bump-type: 'minor'
      dry-run: false
```

### Custom Security Scan

```yaml
jobs:
  custom-security:
    uses: ./.github/workflows/reusable/security-scan.yml
    with:
      language: 'typescript'
      enable-codeql: true
      enable-sast: true
      enable-dependency-scan: true
      enable-secret-scan: false  # Disable specific scans
```

## Configuration

### Secrets Required

| Secret | Description | Used By |
|--------|-------------|---------|
| `CODECOV_TOKEN` | Codecov upload token | coverage-report.yml |
| `SNYK_TOKEN` | Snyk security scanning | security-scan.yml |
| `NPM_TOKEN` | npm registry token | publish-package.yml (TS) |
| `PYPI_TOKEN` | PyPI upload token | publish-package.yml (Python) |
| `CARGO_TOKEN` | crates.io token | publish-package.yml (Rust) |
| `NETLIFY_AUTH_TOKEN` | Netlify deployment | docs-deploy.yml |
| `NETLIFY_SITE_ID` | Netlify site ID | docs-deploy.yml |
| `VERCEL_TOKEN` | Vercel deployment | docs-deploy.yml |
| `AWS_ACCESS_KEY_ID` | AWS S3 access | docs-deploy.yml |
| `AWS_SECRET_ACCESS_KEY` | AWS S3 secret | docs-deploy.yml |
| `SLACK_WEBHOOK_URL` | Slack notifications | sdk-release-orchestrator.yml |

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `COVERAGE_THRESHOLD` | Minimum coverage % | 80 |
| `AUTO_MERGE_PATCH` | Auto-merge patch updates | true |

### Customizing Workflows

#### Override Coverage Threshold

```yaml
jobs:
  coverage:
    uses: ./.github/workflows/reusable/coverage-report.yml
    with:
      coverage-threshold: 90  # Require 90% coverage
      fail-on-threshold: true
```

#### Custom Changelog Format

Create `.github/cliff.toml` to customize git-cliff changelog generation.

#### Custom Badge Colors

Edit `status-dashboard.yml` to adjust badge colors and thresholds.

## Best Practices

1. **Version Tags**: Always use the format `<sdk>-v<version>` for releases
2. **Conventional Commits**: Use conventional commit messages for automatic changelog generation
3. **Security First**: Enable all security scans for production releases
4. **Testing**: Run full test matrix before releases
5. **Documentation**: Keep docs in sync with code releases
6. **Dependencies**: Review and merge dependency update PRs regularly
7. **Monitoring**: Check status dashboard for project health

## Troubleshooting

### Release Failed

1. Check the orchestrator workflow logs
2. Verify version consistency across files
3. Ensure all tests pass
4. Check for security vulnerabilities

### Coverage Below Threshold

1. Add more tests
2. Temporarily lower threshold (not recommended)
3. Use `fail-on-threshold: false` for warnings only

### Dependency Updates Failed

1. Check for breaking changes in dependencies
2. Review and fix test failures
3. Manually update if auto-update fails

## Contributing

When adding new workflows:

1. Use reusable workflows when possible
2. Add comprehensive documentation
3. Include usage examples
4. Test with dry-run mode
5. Update this README

## Support

For issues or questions:
- Create an issue in the repository
- Check workflow logs in Actions tab
- Review workflow YAML comments
- Consult GitHub Actions documentation
