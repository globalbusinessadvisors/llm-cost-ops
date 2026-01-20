#!/bin/bash
# Verify Cost Attribution Agent structure

echo "========================================="
echo "Cost Attribution Agent - Structure Check"
echo "========================================="
echo ""

echo "✓ Checking core files..."
files=(
    "src/handler/index.ts"
    "src/handler/middleware.ts"
    "src/handler/response.ts"
    "src/index.ts"
    "src/types.ts"
    "src/schemas.ts"
    "src/calculator.ts"
    "src/attributor.ts"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo "  ✓ $file"
    else
        echo "  ✗ $file (MISSING)"
    fi
done

echo ""
echo "✓ Checking configuration files..."
configs=(
    "package.json"
    "tsconfig.json"
    "jest.config.js"
    ".gitignore"
    ".gcloudignore"
    ".env.example"
)

for config in "${configs[@]}"; do
    if [ -f "$config" ]; then
        echo "  ✓ $config"
    else
        echo "  ✗ $config (MISSING)"
    fi
done

echo ""
echo "✓ Checking test files..."
tests=(
    "tests/calculator.test.ts"
    "tests/attributor.test.ts"
)

for test in "${tests[@]}"; do
    if [ -f "$test" ]; then
        echo "  ✓ $test"
    else
        echo "  ✗ $test (MISSING)"
    fi
done

echo ""
echo "✓ Checking documentation..."
docs=(
    "README.md"
    "IMPLEMENTATION_SUMMARY.md"
    "example-request.json"
)

for doc in "${docs[@]}"; do
    if [ -f "$doc" ]; then
        echo "  ✓ $doc"
    else
        echo "  ✗ $doc (MISSING)"
    fi
done

echo ""
echo "========================================="
echo "Structure verification complete!"
echo "========================================="
echo ""
echo "Next steps:"
echo "  1. npm install"
echo "  2. npm test"
echo "  3. npm run build"
echo "  4. npm run deploy"
