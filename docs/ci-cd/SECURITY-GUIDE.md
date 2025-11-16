# CI/CD Security Guide

Comprehensive security practices for SDK CI/CD pipelines.

## Overview

This guide covers security best practices, threat models, and mitigation strategies for the LLM-CostOps SDK CI/CD infrastructure.

## Security Layers

```
┌─────────────────────────────────────────────────────────────┐
│                  CI/CD SECURITY LAYERS                      │
└─────────────────────────────────────────────────────────────┘

Layer 1: Code Security (SAST)
├─ CodeQL semantic analysis
├─ Semgrep pattern matching
└─ Language-specific linters

Layer 2: Dependency Security
├─ Vulnerability scanning
├─ License compliance
└─ SBOM generation

Layer 3: Secret Management
├─ Secret detection
├─ Secure storage
└─ Rotation policies

Layer 4: Infrastructure Security
├─ Workflow permissions
├─ Environment protection
└─ Branch protection

Layer 5: Supply Chain Security
├─ Provenance tracking
├─ Artifact signing
└─ Verification
```

## 1. Static Application Security Testing (SAST)

### CodeQL Configuration

**Advanced Setup:**

```yaml
- name: Initialize CodeQL
  uses: github/codeql-action/init@v3
  with:
    languages: ${{ matrix.language }}

    # Use security-extended query suite
    queries: +security-extended,security-and-quality

    # Custom queries
    config-file: ./.github/codeql/codeql-config.yml

    # Increase timeout for large repos
    setup-timeout-minutes: 20
```

**Custom Query Configuration (`.github/codeql/codeql-config.yml`):**

```yaml
name: "Custom CodeQL Configuration"

queries:
  - uses: security-extended
  - uses: security-and-quality

# Custom query packs
packs:
  - codeql/python-queries
  - codeql/javascript-queries

# Path filters
paths-ignore:
  - '**/test/**'
  - '**/tests/**'
  - '**/vendor/**'
  - '**/node_modules/**'

# Query filters
query-filters:
  - exclude:
      id:
        - py/clear-text-logging-sensitive-data
  - include:
      tags:
        - security
        - external/cwe
```

### Semgrep Rules

**Custom Rules (`.semgrep/rules/custom.yml`):**

```yaml
rules:
  - id: hardcoded-api-key
    pattern: |
      api_key = "..."
    message: "Hardcoded API key detected"
    severity: ERROR
    languages: [python]

  - id: sql-injection
    pattern: |
      db.execute(f"SELECT * FROM {$TABLE}")
    message: "Potential SQL injection"
    severity: ERROR
    languages: [python]

  - id: unsafe-deserialization
    pattern: pickle.loads($DATA)
    message: "Unsafe deserialization with pickle"
    severity: WARNING
    languages: [python]
```

## 2. Dependency Security

### Multi-Tool Scanning Strategy

**Why Multiple Tools?**

Different tools have different vulnerability databases:

```
Tool              Database            Coverage
──────────────────────────────────────────────
pip-audit        PyPI Advisory       Python
Safety           Safety DB           Python
Snyk             Snyk Vuln DB        Multi-lang
govulncheck      Go Vuln DB          Go
cargo audit      RustSec             Rust
npm audit        NPM Advisory        JavaScript
OWASP Dep Check  NVD                 Java
```

### Python Example

```yaml
- name: Comprehensive Python Dependency Scan
  run: |
    # Install scanners
    pip install pip-audit safety

    # pip-audit (PyPI Advisory Database)
    pip-audit --format json --output pip-audit.json || true

    # Safety (Safety DB)
    safety check --json --output safety.json || true

    # Combine results
    python scripts/merge-scan-results.py pip-audit.json safety.json > combined.json
```

### Vulnerability Thresholds

**Set Acceptable Risk Levels:**

```yaml
- name: Check vulnerability severity
  run: |
    python scripts/check-vulnerabilities.py \
      --max-critical 0 \
      --max-high 0 \
      --max-medium 5 \
      --max-low 10 \
      --input combined.json
```

**Script (`scripts/check-vulnerabilities.py`):**

```python
import json
import sys
from collections import Counter

def check_vulnerabilities(scan_file, max_critical=0, max_high=0, max_medium=5):
    with open(scan_file) as f:
        results = json.load(f)

    severity_counts = Counter()

    for vuln in results.get('vulnerabilities', []):
        severity = vuln.get('severity', 'UNKNOWN').upper()
        severity_counts[severity] += 1

    # Check thresholds
    failures = []

    if severity_counts['CRITICAL'] > max_critical:
        failures.append(f"Critical: {severity_counts['CRITICAL']} (max: {max_critical})")

    if severity_counts['HIGH'] > max_high:
        failures.append(f"High: {severity_counts['HIGH']} (max: {max_high})")

    if severity_counts['MEDIUM'] > max_medium:
        failures.append(f"Medium: {severity_counts['MEDIUM']} (max: {max_medium})")

    if failures:
        print("❌ Vulnerability threshold exceeded:")
        for failure in failures:
            print(f"  - {failure}")
        sys.exit(1)
    else:
        print("✅ All vulnerability thresholds met")

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('--input', required=True)
    parser.add_argument('--max-critical', type=int, default=0)
    parser.add_argument('--max-high', type=int, default=0)
    parser.add_argument('--max-medium', type=int, default=5)
    args = parser.parse_args()

    check_vulnerabilities(args.input, args.max_critical, args.max_high, args.max_medium)
```

## 3. Secret Detection

### Multi-Layer Secret Scanning

```yaml
secret-detection:
  runs-on: ubuntu-latest
  steps:
    # Layer 1: Gitleaks (fast, pattern-based)
    - name: Run Gitleaks
      uses: gitleaks/gitleaks-action@v2
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

    # Layer 2: TruffleHog (entropy-based)
    - name: Run TruffleHog
      uses: trufflesecurity/trufflehog@main
      with:
        extra_args: --only-verified

    # Layer 3: Custom patterns
    - name: Custom secret patterns
      run: |
        # Check for common API key patterns
        if grep -r "api[_-]key.*=.*['\"][a-zA-Z0-9]{32,}['\"]" .; then
          echo "❌ Potential API key found"
          exit 1
        fi
```

### Secret Rotation Policy

**Automated Rotation:**

```yaml
# .github/workflows/rotate-secrets.yml
name: Secret Rotation Reminder

on:
  schedule:
    - cron: '0 0 1 */3 *'  # Quarterly

jobs:
  remind:
    runs-on: ubuntu-latest
    steps:
      - name: Create rotation issue
        uses: actions/github-script@v7
        with:
          script: |
            github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: '🔄 Quarterly Secret Rotation',
              body: `
                # Secret Rotation Checklist

                It's time for quarterly secret rotation:

                - [ ] PYPI_TOKEN
                - [ ] NPM_TOKEN
                - [ ] CODECOV_TOKEN
                - [ ] Snyk token
                - [ ] API keys

                **Process:**
                1. Generate new tokens
                2. Update GitHub secrets
                3. Test in dev environment
                4. Deploy to production
                5. Revoke old tokens

                Due: ${new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]}
              `,
              labels: ['security', 'maintenance']
            })
```

## 4. Workflow Security

### Principle of Least Privilege

**Default Permissions:**

```yaml
# Deny by default
permissions: {}

jobs:
  build:
    runs-on: ubuntu-latest

    # Grant only what's needed
    permissions:
      contents: read
      packages: write

    steps:
      - uses: actions/checkout@v4
```

**Permission Matrix:**

| Job | contents | packages | pull-requests | security-events |
|-----|----------|----------|---------------|-----------------|
| Test | read | - | write (PR comments) | - |
| Lint | read | - | write | - |
| Security Scan | read | - | - | write (SARIF) |
| Build | read | write | - | - |
| Publish | read | write | - | - |

### Environment Protection

**Production Environment:**

```yaml
jobs:
  publish-production:
    environment:
      name: production
      url: https://pypi.org/project/llm-cost-ops-sdk

    steps:
      - name: Publish
        run: twine upload dist/*
```

**Configuration (Settings > Environments > production):**

```
Protection rules:
├─ Required reviewers: 2 (from security team)
├─ Wait timer: 5 minutes
├─ Deployment branches: main only
└─ Environment secrets:
   ├─ PYPI_TOKEN (production)
   ├─ GPG_PRIVATE_KEY
   └─ GPG_PASSPHRASE
```

## 5. Supply Chain Security

### SBOM Generation

**Comprehensive SBOM:**

```yaml
- name: Generate SBOM (SPDX)
  uses: anchore/sbom-action@v0
  with:
    format: spdx-json
    output-file: sbom.spdx.json

- name: Generate SBOM (CycloneDX)
  run: |
    pip install cyclonedx-bom
    cyclonedx-py -o sbom.cdx.json

- name: Upload to Dependency Track
  run: |
    curl -X POST "${{ secrets.DEPENDENCY_TRACK_URL }}/api/v1/bom" \
      -H "X-Api-Key: ${{ secrets.DEPENDENCY_TRACK_KEY }}" \
      -H "Content-Type: multipart/form-data" \
      -F "project=${{ github.repository }}" \
      -F "bom=@sbom.cdx.json"
```

### Artifact Signing

**Sigstore Signing:**

```yaml
- name: Sign artifacts with Sigstore
  uses: sigstore/cosign-installer@v3

- name: Sign release artifacts
  run: |
    # Sign all distribution files
    for file in dist/*; do
      cosign sign-blob \
        --yes \
        --output-signature "${file}.sig" \
        --output-certificate "${file}.crt" \
        "$file"
    done

- name: Verify signatures
  run: |
    for file in dist/*.whl; do
      cosign verify-blob \
        --signature "${file}.sig" \
        --certificate "${file}.crt" \
        --certificate-identity-regexp=".*" \
        --certificate-oidc-issuer="https://token.actions.githubusercontent.com" \
        "$file"
    done
```

### Provenance Generation

**SLSA Provenance:**

```yaml
- name: Generate provenance
  uses: slsa-framework/slsa-github-generator/.github/workflows/generator_generic_slsa3.yml@v1.9.0
  with:
    base64-subjects: |
      $(cat dist/* | base64)

    # Upload to release
    upload-assets: true
```

## 6. Access Control

### CODEOWNERS

**File:** `.github/CODEOWNERS`

```
# CI/CD Infrastructure
/.github/workflows/ @llm-devops/cicd-team @security-team

# Security-sensitive files
/secrets/ @security-team
*.key @security-team
*.pem @security-team

# SDK Code
/sdks/python/ @llm-devops/python-team
/sdks/typescript/ @llm-devops/typescript-team

# Documentation
/docs/ @llm-devops/docs-team

# Catch-all
* @llm-devops/core-team
```

### Branch Protection

**Security-Focused Rules:**

```
Branch: main
├─ Require pull request reviews: 2
│  ├─ Dismiss stale reviews: Yes
│  ├─ Require code owner review: Yes
│  └─ Restrict push to CODEOWNERS: Yes
├─ Require status checks:
│  ├─ CodeQL
│  ├─ Security Scan
│  ├─ Dependency Check
│  └─ Secret Scan
├─ Require signed commits: Yes
├─ Require linear history: Yes
├─ Include administrators: Yes
└─ Restrict deletions: Yes
```

## 7. Monitoring & Alerting

### Security Alerts

**GitHub Security Tab:**

```yaml
- name: Upload security results
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: results.sarif
    category: ${{ github.workflow }}
```

**Slack Notifications:**

```yaml
- name: Security alert to Slack
  if: failure()
  uses: slackapi/slack-github-action@v1
  with:
    payload: |
      {
        "text": "🚨 Security scan failed!",
        "blocks": [
          {
            "type": "header",
            "text": {
              "type": "plain_text",
              "text": "Security Vulnerability Detected"
            }
          },
          {
            "type": "section",
            "fields": [
              {"type": "mrkdwn", "text": "*Repository:*\n${{ github.repository }}"},
              {"type": "mrkdwn", "text": "*Branch:*\n${{ github.ref_name }}"},
              {"type": "mrkdwn", "text": "*Commit:*\n${{ github.sha }}"},
              {"type": "mrkdwn", "text": "*Workflow:*\n${{ github.workflow }}"}
            ]
          },
          {
            "type": "actions",
            "elements": [
              {
                "type": "button",
                "text": {"type": "plain_text", "text": "View Security Tab"},
                "url": "${{ github.server_url }}/${{ github.repository }}/security",
                "style": "danger"
              }
            ]
          }
        ]
      }
  env:
    SLACK_WEBHOOK_URL: ${{ secrets.SECURITY_SLACK_WEBHOOK }}
```

### Audit Logging

**Enable Audit Log:**

```yaml
- name: Log security event
  run: |
    cat >> security-audit.log <<EOF
    {
      "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
      "event": "security_scan_completed",
      "workflow": "${{ github.workflow }}",
      "run_id": "${{ github.run_id }}",
      "repository": "${{ github.repository }}",
      "actor": "${{ github.actor }}",
      "ref": "${{ github.ref }}",
      "sha": "${{ github.sha }}",
      "status": "${{ job.status }}"
    }
    EOF

    # Upload to centralized logging
    aws s3 cp security-audit.log s3://security-logs/${{ github.repository }}/
```

## 8. Incident Response

### Security Incident Workflow

```yaml
# .github/workflows/security-incident.yml
name: Security Incident Response

on:
  workflow_dispatch:
    inputs:
      severity:
        description: 'Incident severity'
        required: true
        type: choice
        options:
          - critical
          - high
          - medium
          - low
      description:
        description: 'Incident description'
        required: true

jobs:
  respond:
    runs-on: ubuntu-latest
    steps:
      - name: Create incident issue
        uses: actions/github-script@v7
        with:
          script: |
            const issue = await github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: `🚨 Security Incident: ${{ inputs.severity }}`,
              body: `
                # Security Incident Report

                **Severity:** ${{ inputs.severity }}
                **Reported by:** ${{ github.actor }}
                **Date:** ${new Date().toISOString()}

                ## Description
                ${{ inputs.description }}

                ## Immediate Actions
                - [ ] Assess impact
                - [ ] Contain incident
                - [ ] Notify stakeholders
                - [ ] Document findings

                ## Response Team
                @security-team
              `,
              labels: ['security', 'incident', '${{ inputs.severity }}'],
              assignees: ['security-lead']
            })

      - name: Notify security team
        uses: slackapi/slack-github-action@v1
        with:
          payload: |
            {
              "text": "🚨 Security Incident: ${{ inputs.severity }}",
              "attachments": [{
                "color": "danger",
                "fields": [
                  {"title": "Severity", "value": "${{ inputs.severity }}", "short": true},
                  {"title": "Reporter", "value": "${{ github.actor }}", "short": true},
                  {"title": "Description", "value": "${{ inputs.description }}"}
                ]
              }]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SECURITY_INCIDENT_WEBHOOK }}

      - name: Disable workflows
        if: inputs.severity == 'critical'
        uses: actions/github-script@v7
        with:
          script: |
            // Disable all workflows as precaution
            const workflows = await github.rest.actions.listRepoWorkflows({
              owner: context.repo.owner,
              repo: context.repo.repo
            })

            for (const workflow of workflows.data.workflows) {
              if (workflow.state === 'active') {
                await github.rest.actions.disableWorkflow({
                  owner: context.repo.owner,
                  repo: context.repo.repo,
                  workflow_id: workflow.id
                })
              }
            }
```

## Security Checklist

### Pre-Production

- [ ] All secrets stored in GitHub Secrets (not hardcoded)
- [ ] Branch protection enabled on main/production branches
- [ ] CodeQL analysis enabled
- [ ] Dependency scanning configured
- [ ] Secret scanning enabled
- [ ] SBOM generation implemented
- [ ] Least privilege permissions set
- [ ] Environment protection configured for production
- [ ] CODEOWNERS file created
- [ ] Security team added to repository
- [ ] Audit logging enabled
- [ ] Incident response workflow created

### Ongoing

- [ ] Weekly dependency updates reviewed
- [ ] Quarterly secret rotation completed
- [ ] Monthly security scan review
- [ ] Security alerts addressed within SLA
- [ ] Annual security audit completed
- [ ] Team security training current

---

**Security Contact:** security@llm-cost-ops.com
**Incident Hotline:** Available 24/7
**Bug Bounty:** https://llm-cost-ops.com/security/bounty
