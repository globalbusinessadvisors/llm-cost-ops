#!/bin/bash
#
# Pre-Release Verification Script
# Run this before creating a release tag to ensure everything is ready
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SDK_DIR="$(dirname "$SCRIPT_DIR")"

cd "$SDK_DIR"

echo "========================================"
echo "TypeScript SDK - Pre-Release Checks"
echo "========================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track status
ALL_PASSED=true

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
        ALL_PASSED=false
    fi
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# Check if we're in the SDK directory
if [ ! -f "package.json" ]; then
    echo -e "${RED}Error: package.json not found. Run this script from the SDK directory.${NC}"
    exit 1
fi

# Extract version from package.json
VERSION=$(node -p "require('./package.json').version")
echo "Package version: $VERSION"
echo ""

# 1. Type Checking
echo "1. Type Checking..."
if npm run typecheck > /dev/null 2>&1; then
    print_status 0 "Type checking passed"
else
    print_status 1 "Type checking failed"
fi

# 2. Linting
echo "2. Linting..."
if npm run lint > /dev/null 2>&1; then
    print_status 0 "Linting passed"
else
    print_status 1 "Linting failed (try: npm run lint:fix)"
fi

# 3. Unit Tests
echo "3. Running tests..."
if npm run test > /dev/null 2>&1; then
    print_status 0 "Tests passed"
else
    print_status 1 "Tests failed"
fi

# 4. Build
echo "4. Building package..."
if npm run build > /dev/null 2>&1; then
    print_status 0 "Build successful"
else
    print_status 1 "Build failed"
fi

# 5. Verify Build Outputs
echo "5. Verifying build outputs..."
OUTPUTS_OK=true
if [ ! -f "dist/index.js" ]; then
    echo "  - CJS bundle missing (dist/index.js)"
    OUTPUTS_OK=false
fi
if [ ! -f "dist/index.mjs" ]; then
    echo "  - ESM bundle missing (dist/index.mjs)"
    OUTPUTS_OK=false
fi
if [ ! -f "dist/index.d.ts" ]; then
    echo "  - Type declarations missing (dist/index.d.ts)"
    OUTPUTS_OK=false
fi

if [ "$OUTPUTS_OK" = true ]; then
    print_status 0 "Build outputs verified"
else
    print_status 1 "Build outputs incomplete"
fi

# 6. Bundle Size Check
echo "6. Checking bundle size..."
if [ -f "dist/index.mjs" ]; then
    SIZE=$(stat -f%z dist/index.mjs 2>/dev/null || stat -c%s dist/index.mjs 2>/dev/null || echo "0")
    SIZE_KB=$((SIZE / 1024))

    if [ $SIZE_KB -lt 100 ]; then
        print_status 0 "Bundle size OK (${SIZE_KB}KB < 100KB)"
    else
        print_warning "Bundle size large (${SIZE_KB}KB > 100KB)"
    fi
else
    print_warning "Bundle not found, skipping size check"
fi

# 7. Security Audit
echo "7. Running security audit..."
AUDIT_OUTPUT=$(npm audit --production --json 2>/dev/null || echo '{"error":true}')
CRITICAL=$(echo "$AUDIT_OUTPUT" | jq -r '.metadata.vulnerabilities.critical // 0' 2>/dev/null || echo "0")
HIGH=$(echo "$AUDIT_OUTPUT" | jq -r '.metadata.vulnerabilities.high // 0' 2>/dev/null || echo "0")

if [ "$CRITICAL" -eq 0 ] && [ "$HIGH" -eq 0 ]; then
    print_status 0 "No critical/high vulnerabilities"
else
    print_status 1 "Found $CRITICAL critical and $HIGH high vulnerabilities"
    echo "  Run: npm audit"
fi

# 8. License Check
echo "8. Checking licenses..."
if command -v npx > /dev/null 2>&1; then
    ALLOWED="MIT;Apache-2.0;BSD-2-Clause;BSD-3-Clause;ISC;0BSD;CC0-1.0;Unlicense"
    if npx license-checker --production --onlyAllow "$ALLOWED" --summary > /dev/null 2>&1; then
        print_status 0 "License compliance OK"
    else
        print_warning "License compliance check failed or not all licenses allowed"
        echo "  Run: npx license-checker --production --summary"
    fi
else
    print_warning "npx not available, skipping license check"
fi

# 9. Check if version tag exists
echo "9. Checking version tag..."
TAG="v${VERSION}-typescript"
if git rev-parse "$TAG" >/dev/null 2>&1; then
    print_warning "Tag $TAG already exists"
    echo "  Consider bumping version: npm version patch|minor|major"
else
    print_status 0 "Tag $TAG available"
fi

# 10. Check git status
echo "10. Checking git status..."
if [ -z "$(git status --porcelain)" ]; then
    print_status 0 "Working directory clean"
else
    print_warning "Working directory has uncommitted changes"
    echo "  Run: git status"
fi

echo ""
echo "========================================"

if [ "$ALL_PASSED" = true ]; then
    echo -e "${GREEN}All checks passed!${NC}"
    echo ""
    echo "Ready to release version $VERSION"
    echo ""
    echo "Next steps:"
    echo "  1. Commit any remaining changes"
    echo "  2. Create tag: git tag v${VERSION}-typescript"
    echo "  3. Push tag: git push origin v${VERSION}-typescript"
    echo ""
    exit 0
else
    echo -e "${RED}Some checks failed!${NC}"
    echo ""
    echo "Please fix the issues above before releasing."
    echo ""
    exit 1
fi
