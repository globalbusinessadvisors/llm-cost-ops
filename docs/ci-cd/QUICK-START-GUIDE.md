# CI/CD Quick Start Guide

Get your SDK CI/CD pipeline running in 15 minutes.

## Prerequisites

- [x] GitHub repository with SDK code
- [x] Admin access to repository
- [x] SDK structured in `sdks/{language}/` directory

## Step 1: Copy Reusable Workflows (2 min)

```bash
# From llm-cost-ops repository
cp -r .github/workflows/reusable /path/to/your-sdk/.github/workflows/

# Verify files copied
ls -la /path/to/your-sdk/.github/workflows/reusable/
```

Expected files:
```
reusable/
├── test-matrix.yml
├── lint-quality.yml
├── security-scan.yml
├── release-automation.yml
└── publish-package.yml
```

## Step 2: Create Language-Specific Workflow (3 min)

Choose your SDK language:

### Python SDK

```bash
cp .github/workflows/sdk/python-sdk.yml /path/to/your-sdk/.github/workflows/
```

Edit the file and update:
```yaml
env:
  WORKING_DIR: 'sdks/python'  # Change to your SDK path

on:
  push:
    paths:
      - 'sdks/python/**'  # Change to your SDK path
```

### TypeScript SDK

```bash
cp .github/workflows/sdk/typescript-sdk.yml /path/to/your-sdk/.github/workflows/
```

Update `WORKING_DIR` and paths as above.

### Go / Java / Rust

Follow the same pattern - copy template and update paths.

## Step 3: Configure GitHub Secrets (5 min)

Go to: **Repository Settings > Secrets and variables > Actions**

### Required Secrets

#### For Testing
```
LLM_COST_OPS_TEST_API_KEY     = your-test-api-key
LLM_COST_OPS_TEST_BASE_URL    = https://staging-api.llm-cost-ops.com
```

#### For Code Coverage
```
CODECOV_TOKEN = your-codecov-token
```

#### For Publishing (choose your language)

**Python:**
```
PYPI_TOKEN = pypi-...your-token...
```

**TypeScript:**
```
NPM_TOKEN = npm_...your-token...
```

**Rust:**
```
CARGO_REGISTRY_TOKEN = your-crates-io-token
```

**Java:**
```
OSSRH_USERNAME = your-username
OSSRH_PASSWORD = your-password
GPG_PRIVATE_KEY = -----BEGIN PGP PRIVATE KEY BLOCK-----...
GPG_PASSPHRASE = your-gpg-passphrase
```

#### Optional (Monitoring)
```
SLACK_WEBHOOK_URL = https://hooks.slack.com/services/...
SNYK_TOKEN = your-snyk-token
```

### How to Get Tokens

**PyPI:**
1. Go to https://pypi.org/manage/account/token/
2. Create new token with scope "Entire account"
3. Copy token

**NPM:**
1. Go to https://www.npmjs.com/settings/your-username/tokens
2. Generate New Token > Automation
3. Copy token

**Codecov:**
1. Go to https://codecov.io/gh/your-org/your-repo/settings
2. Copy repository upload token

## Step 4: Enable Branch Protection (2 min)

Go to: **Repository Settings > Branches**

Click **Add rule** for `main` branch:

```
☑ Require pull request before merging
  ☑ Require approvals: 2

☑ Require status checks to pass before merging
  ☑ Require branches to be up to date before merging

  Select these status checks:
    ☑ Test Matrix
    ☑ Code Quality
    ☑ Security Scan
    ☑ Build

☑ Require conversation resolution before merging

☑ Do not allow bypassing the above settings
```

## Step 5: Test the Pipeline (3 min)

### Create Test PR

```bash
# Create test branch
git checkout -b test/ci-cd-setup

# Make small change
echo "# CI/CD Test" >> README.md

# Commit and push
git add README.md
git commit -m "test: CI/CD pipeline setup"
git push origin test/ci-cd-setup
```

### Create Pull Request

1. Go to GitHub repository
2. Click "Pull requests" > "New pull request"
3. Select `test/ci-cd-setup` branch
4. Create pull request

### Verify Workflows Run

You should see these checks:
- ✓ Test Matrix (Python 3.12, ubuntu-latest)
- ✓ Code Quality
- ✓ Security Scan
- ✓ Build

**If checks fail**, see [Troubleshooting](#troubleshooting) below.

## Step 6: Configure Dependabot (Optional, 2 min)

Create `.github/dependabot.yml`:

```yaml
version: 2
updates:
  # Choose your language
  - package-ecosystem: "pip"  # or "npm", "gomod", "gradle", "cargo"
    directory: "/sdks/python"
    schedule:
      interval: "weekly"
    reviewers:
      - "your-team"
```

Commit and push:
```bash
git add .github/dependabot.yml
git commit -m "chore: add Dependabot configuration"
git push
```

## Troubleshooting

### Workflow Not Running

**Problem:** PR created but no CI checks appear

**Solutions:**
1. Check workflow file syntax:
   ```bash
   # Use GitHub's workflow validator
   gh workflow view python-sdk.yml
   ```

2. Verify path filters match your changes:
   ```yaml
   on:
     pull_request:
       paths:
         - 'sdks/python/**'  # Must match files changed
   ```

3. Check Actions are enabled:
   - Settings > Actions > General
   - Select "Allow all actions and reusable workflows"

### Test Failures

**Problem:** Tests fail in CI but pass locally

**Common Causes:**

1. **Missing dependencies:**
   ```yaml
   # Add to workflow
   - name: Install dependencies
     run: pip install -e ".[dev,test]"
   ```

2. **Wrong working directory:**
   ```yaml
   # All steps need this
   working-directory: sdks/python
   ```

3. **Environment differences:**
   ```yaml
   # Add environment variables
   env:
     PYTHONPATH: ${{ github.workspace }}/sdks/python
   ```

### Secret Not Found

**Problem:** `Error: Input required and not supplied: PYPI_TOKEN`

**Solutions:**

1. Verify secret exists in Settings > Secrets
2. Check secret name matches exactly (case-sensitive)
3. For reusable workflows, pass secrets:
   ```yaml
   uses: ./.github/workflows/reusable/publish.yml
   secrets:
     PYPI_TOKEN: ${{ secrets.PYPI_TOKEN }}
   ```

### Coverage Upload Fails

**Problem:** Coverage report not appearing on Codecov

**Solutions:**

1. Verify CODECOV_TOKEN is set
2. Check coverage file is generated:
   ```bash
   # In CI logs, look for:
   coverage.xml
   ```
3. Ensure pytest-cov is installed:
   ```bash
   pip install pytest-cov
   ```

## Next Steps

### Customize Your Workflows

1. **Adjust test matrix:**
   ```yaml
   matrix-versions: '["3.9", "3.10", "3.11", "3.12"]'  # Add/remove versions
   ```

2. **Add integration tests:**
   ```yaml
   integration-tests:
     needs: [test]
     steps:
       - name: Run integration tests
         run: pytest tests/integration/
   ```

3. **Customize notifications:**
   ```yaml
   - name: Notify on success
     if: success()
     uses: slackapi/slack-github-action@v1
   ```

### Enable Advanced Features

1. **Performance benchmarking:**
   - Add benchmark tests
   - Configure benchmark-action

2. **E2E testing:**
   - Set up test environment
   - Add E2E test job

3. **Multi-environment deployments:**
   - Create staging/production environments
   - Add deployment workflows

### Read Full Documentation

- [CI/CD Architecture](./CI-CD-ARCHITECTURE.md) - Complete system design
- [Security Guide](./SECURITY-GUIDE.md) - Security best practices
- [Optimization Guide](./OPTIMIZATION-GUIDE.md) - Performance tuning

## Cheat Sheet

### Common Commands

```bash
# View workflow runs
gh run list --workflow=python-sdk.yml

# View specific run
gh run view <run-id>

# Re-run failed jobs
gh run rerun <run-id> --failed

# Cancel running workflow
gh run cancel <run-id>

# List secrets
gh secret list

# Set secret
gh secret set PYPI_TOKEN < token.txt
```

### Workflow Triggers

```yaml
# On push to main
on:
  push:
    branches: [main]

# On PR to main or develop
on:
  pull_request:
    branches: [main, develop]

# On release
on:
  release:
    types: [published]

# Manual trigger
on:
  workflow_dispatch:

# Scheduled (daily at 2am UTC)
on:
  schedule:
    - cron: '0 2 * * *'
```

### Useful Filters

```yaml
# Only run on specific paths
on:
  push:
    paths:
      - 'src/**'
      - '!src/docs/**'  # Exclude

# Only run on specific files
on:
  push:
    paths:
      - '**.py'  # All Python files
```

---

**Need Help?**
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [LLM-CostOps Discord](https://discord.gg/llm-cost-ops)
- [Open an Issue](https://github.com/llm-cost-ops/sdk/issues)
