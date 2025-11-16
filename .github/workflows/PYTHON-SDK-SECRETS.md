# Python SDK CI/CD - Required Secrets & Configuration

This document lists all secrets and configuration required for the Python SDK CI/CD workflows.

## GitHub Secrets

Configure these secrets in: `Settings > Secrets and variables > Actions`

### Optional Secrets

| Secret Name | Description | Used By | Required |
|-------------|-------------|---------|----------|
| `CODECOV_TOKEN` | Codecov API token for coverage reporting | Testing Workflow | No |
| `SNYK_TOKEN` | Snyk API token for security scanning | Security Workflow | No |
| `SLACK_WEBHOOK_URL` | Slack webhook for notifications | Testing Workflow | No |

**Note**: All secrets are optional. Workflows will skip respective steps if secrets are not configured.

## PyPI Trusted Publishing (Recommended)

**No secrets required!** Configure trusted publishing on PyPI:

### Setup Steps

1. **Create PyPI Account**
   - Go to https://pypi.org/account/register/
   - Verify email address

2. **Register Project** (first-time only)
   - Create initial release manually OR
   - Use TestPyPI first: https://test.pypi.org

3. **Configure Trusted Publisher**
   - Go to: https://pypi.org/manage/project/llm-cost-ops/settings/publishing/
   - Click "Add a new publisher"
   - Fill in:
     ```
     Owner: llm-devops
     Repository name: llm-cost-ops
     Workflow name: sdk-python-release.yml
     Environment name: pypi-release
     ```
   - Save

4. **Create GitHub Environment**
   - Go to: Repository Settings > Environments
   - Click "New environment"
   - Name: `pypi-release`
   - Add protection rules:
     - [x] Required reviewers (recommended for production)
     - Reviewers: DevOps team members
   - Save

### Benefits of Trusted Publishing

- ✅ No API tokens to manage
- ✅ No secrets to rotate
- ✅ More secure (OIDC-based)
- ✅ Easier setup
- ✅ Automatic credential expiration

### Fallback: API Token Method

If trusted publishing is not available:

1. Generate API token: https://pypi.org/manage/account/token/
2. Scope: Project-specific (llm-cost-ops)
3. Add secret: `PYPI_TOKEN`
4. Modify workflow to use token-based publishing

## GitHub Environments

### pypi-release (Required for releases)

```yaml
Name: pypi-release
Protection rules:
  - Required reviewers: 1+ (recommended)
  - Wait timer: 0 minutes
Environment secrets: None (using trusted publishing)
```

### Configuration in Repository

Settings > Environments > New environment

## GitHub Actions Permissions

### Required Permissions

Workflows use the following permissions:

```yaml
# Testing Workflow
permissions:
  contents: read
  checks: write        # For test annotations
  pull-requests: write # For PR comments

# Release Workflow
permissions:
  contents: write      # For creating releases
  id-token: write      # For trusted publishing (REQUIRED)

# Security Workflow
permissions:
  contents: read
  security-events: write # For CodeQL
  issues: write          # For automated issues

# Documentation Workflow
permissions:
  contents: write      # For gh-pages deployment
  pages: write         # For GitHub Pages
  id-token: write      # For Pages deployment
```

### Setting Permissions

Repository Settings > Actions > General > Workflow permissions:
- Select: "Read and write permissions"
- [x] Allow GitHub Actions to create and approve pull requests

## Third-Party Service Configuration

### 1. Codecov (Optional)

**Purpose**: Code coverage reporting and visualization

**Setup**:
1. Go to https://codecov.io/
2. Sign in with GitHub
3. Add repository: `llm-devops/llm-cost-ops`
4. Copy token from Settings
5. Add to GitHub secrets as `CODECOV_TOKEN`

**Benefits**:
- Coverage trends over time
- PR comments with coverage changes
- Coverage badges for README

### 2. Snyk (Optional)

**Purpose**: Advanced vulnerability scanning and monitoring

**Setup**:
1. Go to https://snyk.io/
2. Sign in with GitHub
3. Add repository
4. Get API token: Account Settings > API Token
5. Add to GitHub secrets as `SNYK_TOKEN`

**Benefits**:
- Continuous monitoring
- Automated PR checks
- Fix suggestions
- License compliance

### 3. GitHub Pages (For Documentation)

**Setup**:
1. Settings > Pages
2. Source: "Deploy from a branch"
3. Branch: `gh-pages`
4. Folder: `/` (root)
5. Custom domain (optional): `docs.llm-cost-ops.dev`

**DNS Configuration** (if using custom domain):
```
CNAME docs.llm-cost-ops.dev -> llm-devops.github.io
```

## Branch Protection Rules

Recommended settings for `main` branch:

```yaml
Branch name pattern: main

Protect matching branches:
  [x] Require a pull request before merging
      Required approvals: 1
      [x] Dismiss stale pull request approvals when new commits are pushed
      [x] Require review from Code Owners

  [x] Require status checks to pass before merging
      [x] Require branches to be up to date before merging
      Required checks:
        - Lint & Type Check
        - Test - Python 3.12 on ubuntu-latest
        - Coverage Check
        - Build Distribution

  [x] Require conversation resolution before merging
  [x] Require signed commits
  [x] Require linear history
  [x] Include administrators

  [ ] Allow force pushes
  [ ] Allow deletions
```

## Dependabot Configuration

Create `.github/dependabot.yml` for automated dependency updates:

```yaml
version: 2
updates:
  - package-ecosystem: "pip"
    directory: "/python-sdk"
    schedule:
      interval: "weekly"
      day: "monday"
      time: "09:00"
    open-pull-requests-limit: 10
    reviewers:
      - "devops-team"
    labels:
      - "dependencies"
      - "python-sdk"
    commit-message:
      prefix: "chore"
      prefix-development: "chore"
      include: "scope"

  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "weekly"
    labels:
      - "dependencies"
      - "github-actions"
```

## Security Policies

### SECURITY.md

Create `SECURITY.md` in repository root:

```markdown
# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

Email: security@llm-cost-ops.dev

Expected response time: 48 hours
```

### CodeQL Configuration

Create `.github/codeql/codeql-config.yml`:

```yaml
name: "CodeQL Config"
queries:
  - uses: security-extended
  - uses: security-and-quality

paths-ignore:
  - "python-sdk/tests/**"
  - "python-sdk/examples/**"
```

## Workflow Notifications

### Slack Integration (Optional)

**Setup**:
1. Create Slack app: https://api.slack.com/apps
2. Enable Incoming Webhooks
3. Add webhook to workspace
4. Copy webhook URL
5. Add to GitHub secrets as `SLACK_WEBHOOK_URL`

**Notification Triggers**:
- Test failures on `main`
- Security vulnerabilities found
- Release completions

## Testing Configuration

### Local Testing Tools

Install GitHub Act for local workflow testing:

```bash
# Install act
brew install act  # macOS
# or
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Test workflow locally
act -W .github/workflows/sdk-python-test.yml
```

### Pre-commit Hooks

Install pre-commit to run checks locally:

```bash
# Install pre-commit
pip install pre-commit

# Create .pre-commit-config.yaml
cat > .pre-commit-config.yaml << 'EOF'
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-added-large-files

  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.1.0
    hooks:
      - id: ruff
        args: [--fix]

  - repo: https://github.com/psf/black
    rev: 23.10.0
    hooks:
      - id: black

  - repo: https://github.com/pre-commit/mirrors-mypy
    rev: v1.5.0
    hooks:
      - id: mypy
        additional_dependencies: [types-all]
EOF

# Install hooks
pre-commit install
```

## Checklist for Initial Setup

Use this checklist when setting up CI/CD:

- [ ] Configure PyPI trusted publishing
  - [ ] Create PyPI account
  - [ ] Register project (or use TestPyPI)
  - [ ] Add trusted publisher
  - [ ] Create `pypi-release` environment in GitHub

- [ ] Configure GitHub repository settings
  - [ ] Enable GitHub Actions
  - [ ] Set workflow permissions
  - [ ] Configure branch protection for `main`

- [ ] Optional: Add third-party integrations
  - [ ] Codecov (add `CODECOV_TOKEN`)
  - [ ] Snyk (add `SNYK_TOKEN`)
  - [ ] Slack (add `SLACK_WEBHOOK_URL`)

- [ ] Configure GitHub Pages (for docs)
  - [ ] Enable Pages
  - [ ] Set source to `gh-pages` branch
  - [ ] Optional: Configure custom domain

- [ ] Create security policies
  - [ ] Add `SECURITY.md`
  - [ ] Configure Dependabot
  - [ ] Set up CodeQL

- [ ] Test workflows
  - [ ] Trigger test workflow manually
  - [ ] Create test PR to verify checks
  - [ ] Create test release (to TestPyPI)

- [ ] Document and train team
  - [ ] Share CI/CD documentation
  - [ ] Review release process
  - [ ] Set up monitoring/alerts

## Troubleshooting Common Issues

### "PyPI: 403 Forbidden"

**Cause**: Trusted publishing not configured correctly

**Solution**:
1. Verify environment name is exactly `pypi-release`
2. Check workflow name matches: `sdk-python-release.yml`
3. Ensure repository owner/name correct
4. Verify publisher was added on PyPI

### "Coverage upload failed"

**Cause**: Codecov token missing or invalid

**Solution**:
1. Verify `CODECOV_TOKEN` secret exists
2. Check token hasn't expired
3. Re-generate token from Codecov dashboard
4. Or: Remove Codecov upload step (optional feature)

### "Snyk test failed"

**Cause**: Snyk token missing or dependencies have vulnerabilities

**Solution**:
1. Check `SNYK_TOKEN` secret
2. Review Snyk report in artifacts
3. Update vulnerable dependencies
4. Or: Skip Snyk job (it's optional)

### "CodeQL init failed"

**Cause**: CodeQL doesn't support Python version or repo too large

**Solution**:
1. Check Python version compatibility
2. Add paths-ignore for large files
3. Increase timeout in workflow

## Support

For CI/CD setup issues:
- Email: devops@llm-cost-ops.dev
- GitHub Issues: Tag with `ci-cd` label
- Documentation: `.github/workflows/README-PYTHON-CICD.md`

---

Last updated: 2025-01-16
