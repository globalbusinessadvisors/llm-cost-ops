# Python SDK CI/CD Documentation

This document describes the comprehensive CI/CD workflows for the Python SDK.

## Overview

The Python SDK has four production-ready GitHub Actions workflows:

1. **Testing Workflow** (`sdk-python-test.yml`) - Comprehensive testing across multiple Python versions and OS
2. **Release Workflow** (`sdk-python-release.yml`) - Automated release and PyPI publishing
3. **Security Workflow** (`sdk-python-security.yml`) - Weekly security scanning and vulnerability detection
4. **Documentation Workflow** (`sdk-python-docs.yml`) - Automated documentation generation and deployment

## Workflows

### 1. Testing Workflow (`sdk-python-test.yml`)

**Purpose**: Ensure code quality and functionality across different Python versions and operating systems.

**Triggers**:
- Push to `main` or `develop` branches (when Python SDK files change)
- Pull requests to `main` or `develop`
- Manual workflow dispatch

**Jobs**:

#### Lint & Type Check
- Runs ruff linting with GitHub annotations
- Executes mypy type checking
- Verifies black code formatting
- Checks import sorting with isort

#### Test Matrix
- **Python versions**: 3.9, 3.10, 3.11, 3.12
- **Operating systems**: Ubuntu, macOS, Windows
- Runs pytest with coverage
- Generates JUnit XML reports
- Uploads coverage to Codecov (Ubuntu + Python 3.12 only)

#### Coverage Check
- Enforces minimum 80% code coverage threshold
- Fails if coverage falls below threshold

#### Build Distribution
- Builds wheel and source distribution
- Validates package with twine
- Uploads build artifacts

#### Test Installation
- Tests package installation on multiple platforms
- Verifies imports work correctly
- Tests basic functionality

**Quality Gates**:
- All linting must pass
- Type checking must pass with no errors
- All tests must pass across all matrix combinations
- Code coverage must be >= 80%
- Package must build successfully

**Artifacts**:
- Test results (pytest JUnit XML)
- Coverage reports (HTML)
- Distribution packages (wheel, sdist)

---

### 2. Release Workflow (`sdk-python-release.yml`)

**Purpose**: Automate the release process, security scanning, and publishing to PyPI.

**Triggers**:
- Tags matching `v*-python` (e.g., `v1.0.0-python`)
- Manual workflow dispatch with version input

**Jobs**:

#### Validate Release
- Extracts version from git tag or manual input
- Validates version format (X.Y.Z)
- Verifies version matches `pyproject.toml`

#### Test Before Release
- Runs full test suite
- Type checking with mypy
- Linting with ruff
- Enforces 80% coverage

#### Security Scan
- **Bandit**: Static security analysis for Python code
- **Safety**: Checks for known vulnerabilities in dependencies
- **Pip-audit**: Audits dependencies for security issues
- Uploads security reports as artifacts

#### Build Distribution
- Builds wheel and source distribution
- Validates packages with twine
- Lists package contents
- Uploads artifacts

#### SBOM Generation
- Generates CycloneDX SBOM (Software Bill of Materials)
- Creates license report in JSON and Markdown
- Uploads SBOM artifacts

#### Publish to PyPI
- Uses **PyPI Trusted Publishing** (OIDC authentication)
- Supports both PyPI and TestPyPI
- No API tokens stored in repository
- Automatic retry on transient failures

#### GitHub Release
- Generates changelog from git history
- Creates GitHub release with tag
- Attaches distribution packages
- Attaches SBOM and security reports

#### Verify Release
- Waits for PyPI propagation
- Installs package from PyPI
- Verifies installation and imports

**Required Secrets**:
- None for PyPI (uses trusted publishing)
- Optional: `CODECOV_TOKEN` for coverage reports

**Environment Configuration**:
- Requires `pypi-release` environment in GitHub settings
- Configure PyPI trusted publishing for the repository

**Artifacts**:
- Distribution packages (wheel, sdist)
- Security reports (bandit, safety, pip-audit)
- SBOM files (CycloneDX, licenses)

---

### 3. Security Workflow (`sdk-python-security.yml`)

**Purpose**: Continuous security monitoring and vulnerability detection.

**Triggers**:
- Weekly schedule (Mondays at 9 AM UTC)
- Push to `main` branch (when Python SDK files change)
- Pull requests to `main`
- Manual workflow dispatch

**Jobs**:

#### Dependency Scan
- **Safety**: Checks dependencies for known vulnerabilities
- **Pip-audit**: Additional dependency security scanning
- Creates GitHub issues for vulnerabilities found in scheduled runs
- Uploads reports as artifacts

#### SAST Scan (Static Application Security Testing)
- **Bandit**: Identifies common security issues in Python code
- **Semgrep**: Advanced pattern-based security analysis
- Scans for SQL injection, XSS, hardcoded secrets, etc.

#### Snyk Scan
- Comprehensive vulnerability scanning
- License compliance checking
- Requires `SNYK_TOKEN` secret (optional)
- Skipped for external pull requests

#### CodeQL Analysis
- GitHub's semantic code analysis
- Security and quality queries
- Results appear in Security tab
- Automatic PR comments for findings

#### Secret Scanning
- **TruffleHog**: Scans for leaked credentials
- Checks entire git history
- Only reports verified secrets
- Generates JSON report

#### SBOM Generation
- Weekly SBOM updates
- CycloneDX format (JSON and XML)
- License inventory (JSON, Markdown, CSV)
- Verifies license compliance

#### License Compliance
- Checks all dependency licenses
- Flags problematic licenses (GPL, AGPL, etc.)
- Allows MIT, Apache, BSD, ISC, MPL-2.0, LGPL
- Customizable license policy

#### Security Policy Check
- Verifies SECURITY.md exists
- Checks for security-related configuration
- Validates dependabot configuration

**Required Secrets**:
- `SNYK_TOKEN` (optional, for Snyk scanning)

**Artifacts**:
- Dependency scan reports
- SAST scan results
- Snyk reports
- SBOM files
- License reports
- Secret scan results

**Quality Gates**:
- CodeQL critical findings fail the workflow
- Verified secrets fail the workflow
- License violations fail the workflow

---

### 4. Documentation Workflow (`sdk-python-docs.yml`)

**Purpose**: Generate and deploy comprehensive documentation.

**Triggers**:
- Push to `main` or `develop` branches (when Python SDK files change)
- Pull requests to `main`
- Manual workflow dispatch

**Jobs**:

#### Build Documentation
- Generates Sphinx documentation
- Creates API documentation with pdoc
- Validates documentation links
- Auto-generates config if docs/ doesn't exist

#### Deploy to GitHub Pages
- Deploys to `sdk/python` subdirectory
- Only on pushes to `main`
- Uses custom domain: `docs.llm-cost-ops.dev`
- Preserves other documentation

#### README Check
- Validates README completeness
- Checks for required sections
- Validates Python code examples
- Ensures documentation quality

#### Changelog Generation
- Auto-generates CHANGELOG.md
- Follows Keep a Changelog format
- Groups changes by version
- Extracts from git history

**Artifacts**:
- Sphinx HTML documentation
- pdoc API documentation
- Generated changelog

**Documentation Structure**:
```
docs/
├── conf.py               # Sphinx configuration
├── index.rst            # Documentation homepage
├── installation.rst     # Installation guide
├── quickstart.rst       # Quick start guide
├── api.rst              # API reference
├── examples.rst         # Examples
└── _build/
    ├── html/            # Sphinx output
    └── pdoc/            # pdoc output
```

---

## Setup Instructions

### 1. Configure PyPI Trusted Publishing

For secure, token-free PyPI publishing:

1. Go to PyPI project settings: https://pypi.org/manage/project/llm-cost-ops/settings/publishing/
2. Add a new publisher:
   - **Repository**: `llm-devops/llm-cost-ops`
   - **Workflow**: `sdk-python-release.yml`
   - **Environment**: `pypi-release`
3. Create GitHub environment in repository settings:
   - Name: `pypi-release`
   - Protection rules: Require reviewers for releases

### 2. Configure GitHub Secrets

Optional secrets for enhanced functionality:

```yaml
# Optional: For Codecov coverage reports
CODECOV_TOKEN: <your-codecov-token>

# Optional: For Snyk security scanning
SNYK_TOKEN: <your-snyk-token>

# Optional: For Slack notifications
SLACK_WEBHOOK_URL: <your-slack-webhook>
```

### 3. Enable GitHub Pages

1. Go to repository Settings > Pages
2. Source: Deploy from a branch
3. Branch: `gh-pages`, folder: `/root`
4. Custom domain (optional): `docs.llm-cost-ops.dev`

### 4. Configure Branch Protection

Recommended branch protection rules for `main`:

- Require pull request reviews
- Require status checks to pass:
  - `Lint & Type Check`
  - `Test Matrix`
  - `Coverage Check`
  - `Build Distribution`
- Require branches to be up to date
- Require linear history

---

## Usage Examples

### Running Tests Manually

```bash
# Trigger test workflow
gh workflow run sdk-python-test.yml
```

### Creating a Release

#### Option 1: Tag-based (Recommended)

```bash
# Update version in pyproject.toml
vim python-sdk/pyproject.toml  # Set version = "1.2.3"

# Commit and tag
git add python-sdk/pyproject.toml
git commit -m "chore: Bump Python SDK version to 1.2.3"
git tag v1.2.3-python
git push origin main --tags

# Release workflow triggers automatically
```

#### Option 2: Manual Dispatch

```bash
# Via GitHub CLI
gh workflow run sdk-python-release.yml \
  --field version=1.2.3 \
  --field pypi-repository=testpypi  # or 'pypi'

# Or via GitHub web interface
# Actions > Python SDK - Release > Run workflow
```

### Running Security Scans

```bash
# Manual security scan
gh workflow run sdk-python-security.yml

# Check latest security scan results
gh run list --workflow=sdk-python-security.yml --limit 1
```

### Building Documentation

```bash
# Trigger documentation build
gh workflow run sdk-python-docs.yml

# View deployed docs
# https://docs.llm-cost-ops.dev/sdk/python/
```

---

## Caching Strategy

All workflows use intelligent caching to speed up builds:

### Pip Dependency Caching

```yaml
- uses: actions/setup-python@v5
  with:
    python-version: '3.12'
    cache: 'pip'
    cache-dependency-path: python-sdk/pyproject.toml
```

**Benefits**:
- Reduces dependency installation time by 80-90%
- Cache automatically invalidated when dependencies change
- Separate cache per Python version and OS

### Artifact Retention

- Test results: 30 days
- Coverage reports: 30 days
- Security reports: 90 days
- SBOM: 90 days
- Distribution packages: 90 days

---

## Quality Gates Summary

### Testing Workflow
- ✅ Ruff linting (no errors)
- ✅ MyPy type checking (no errors)
- ✅ Black formatting (properly formatted)
- ✅ isort import sorting (correct order)
- ✅ Tests pass on all OS and Python versions
- ✅ Code coverage >= 80%
- ✅ Package builds successfully
- ✅ Package installs correctly

### Release Workflow
- ✅ Version format valid (X.Y.Z)
- ✅ Version matches pyproject.toml
- ✅ All tests pass
- ✅ Type checking passes
- ✅ Linting passes
- ✅ Coverage >= 80%
- ✅ Security scans complete
- ✅ Package builds successfully
- ✅ Package published to PyPI
- ✅ GitHub release created
- ✅ Installation verification passes

### Security Workflow
- ✅ No critical vulnerabilities in dependencies
- ✅ No verified secrets in code
- ✅ All licenses compliant
- ✅ CodeQL analysis passes
- ⚠️ SAST findings reviewed (warnings allowed)
- ⚠️ Dependency vulnerabilities documented (warnings allowed)

---

## Troubleshooting

### Test Failures

**Coverage below threshold**:
```bash
# Run locally to see missing coverage
cd python-sdk
pytest --cov=llm_cost_ops --cov-report=html
open htmlcov/index.html
```

**Type checking errors**:
```bash
cd python-sdk
mypy llm_cost_ops --show-error-codes
```

**Linting errors**:
```bash
cd python-sdk
ruff check llm_cost_ops --fix  # Auto-fix
black llm_cost_ops tests       # Format
isort llm_cost_ops tests       # Sort imports
```

### Release Issues

**Version mismatch**:
- Ensure `pyproject.toml` version matches tag version
- Tag format: `v1.2.3-python` (not `v1.2.3`)

**PyPI trusted publishing fails**:
- Verify environment name is `pypi-release`
- Check PyPI publisher configuration
- Ensure workflow file name matches exactly

**Release not triggering**:
- Tag must match pattern `v*-python`
- Push tags with: `git push --tags`

### Security Scan Issues

**Snyk token missing**:
- Snyk scan is optional
- Add `SNYK_TOKEN` secret or skip Snyk job

**False positive vulnerabilities**:
- Review security reports in artifacts
- Document exceptions in SECURITY.md
- Use `continue-on-error: true` for specific jobs

---

## Monitoring and Notifications

### Workflow Status

View workflow status:
```bash
# List recent runs
gh run list --workflow=sdk-python-test.yml

# View specific run
gh run view <run-id>

# Watch live run
gh run watch
```

### GitHub Actions Dashboard

- https://github.com/llm-devops/llm-cost-ops/actions
- Filter by workflow, branch, or status
- Download artifacts from completed runs

### Job Summaries

Each workflow generates a summary with:
- Job results
- Coverage metrics
- Security findings
- Build information
- Artifact links

---

## Performance Metrics

Expected workflow durations:

| Workflow | Duration | Notes |
|----------|----------|-------|
| Testing | 15-25 min | Depends on test matrix size |
| Release | 20-30 min | Includes all security scans |
| Security | 10-15 min | CodeQL takes longest |
| Documentation | 5-10 min | Sphinx build time |

Optimization tips:
- Use caching (already implemented)
- Run independent jobs in parallel
- Use matrix strategy for multi-version testing
- Cache test results for PRs

---

## Best Practices

### Development Workflow

1. **Create feature branch**
   ```bash
   git checkout -b feature/new-feature
   ```

2. **Make changes and test locally**
   ```bash
   cd python-sdk
   pytest --cov=llm_cost_ops
   mypy llm_cost_ops
   ruff check llm_cost_ops
   ```

3. **Create pull request**
   - Test workflow runs automatically
   - Review test results and coverage
   - Address any failures

4. **Merge to main**
   - Test workflow runs again
   - Documentation deploys automatically

5. **Create release**
   - Update version in `pyproject.toml`
   - Create and push tag
   - Release workflow handles rest

### Version Bumping

Follow semantic versioning:

- **Major** (X.0.0): Breaking changes
- **Minor** (x.Y.0): New features, backward compatible
- **Patch** (x.y.Z): Bug fixes, backward compatible

### Security Maintenance

- Review weekly security scan results
- Update dependencies regularly
- Address critical vulnerabilities within 7 days
- Document security decisions

---

## Future Enhancements

Potential improvements:

1. **Integration Tests**
   - Test against live API in staging environment
   - Run on release workflow

2. **Performance Benchmarks**
   - Track performance over time
   - Fail on significant regressions

3. **Nightly Builds**
   - Test against development dependencies
   - Early detection of breaking changes

4. **Multi-region Testing**
   - Test installation from different regions
   - Verify CDN propagation

5. **Automated Dependency Updates**
   - Dependabot for security updates
   - Automated PR creation and testing

---

## Support

For issues with CI/CD workflows:

1. Check workflow logs in GitHub Actions
2. Review this documentation
3. Open an issue with workflow run URL
4. Contact DevOps team

---

## Changelog

- **2025-01-16**: Initial CI/CD workflows created
  - Testing workflow with matrix testing
  - Release workflow with PyPI trusted publishing
  - Security workflow with comprehensive scanning
  - Documentation workflow with GitHub Pages deployment
