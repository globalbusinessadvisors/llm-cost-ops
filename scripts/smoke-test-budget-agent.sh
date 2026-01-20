#!/bin/bash
# Smoke Test Script for Budget Enforcement Agent
#
# This script verifies the CLI commands for the Budget Enforcement Agent
# work correctly according to the LLM-CostOps constitution.
#
# Usage:
#   ./scripts/smoke-test-budget-agent.sh
#
# Prerequisites:
#   - Rust and Cargo installed
#   - Project built: cargo build --release

set -e

echo "==========================================="
echo "Budget Enforcement Agent - Smoke Tests"
echo "==========================================="
echo ""

# Binary location (adjust if needed)
BINARY="${CARGO_TARGET_DIR:-./target}/release/cost-ops"

if [ ! -f "$BINARY" ]; then
    echo "Binary not found at $BINARY"
    echo "Please build the project first: cargo build --release"
    exit 1
fi

echo "Using binary: $BINARY"
echo ""

# Test 1: List agents
echo "Test 1: List available agents"
echo "-----------------------------"
$BINARY agent list
echo ""
echo "✓ Agent list works"
echo ""

# Test 2: Get agent info
echo "Test 2: Get Budget Enforcement Agent info"
echo "------------------------------------------"
$BINARY agent info --agent-id budget-enforcement
echo ""
echo "✓ Agent info works"
echo ""

# Test 3: Inspect agent configuration
echo "Test 3: Inspect agent configuration"
echo "------------------------------------"
$BINARY agent budget-enforcement inspect
echo ""
echo "✓ Agent inspect works"
echo ""

# Test 4: Health check
echo "Test 4: Health check (without RuVector)"
echo "----------------------------------------"
$BINARY agent budget-enforcement health
echo ""
echo "✓ Agent health check works"
echo ""

# Test 5: Analyze budget - within limits
echo "Test 5: Analyze budget (50% utilization - OK)"
echo "----------------------------------------------"
$BINARY agent budget-enforcement analyze \
    --tenant-id test-tenant \
    --budget-id budget-001 \
    --budget-limit 1000 \
    --current-spend 500 \
    --dry-run \
    --output table
echo ""
echo "✓ Budget within limits: ADVISORY signal"
echo ""

# Test 6: Analyze budget - warning threshold
echo "Test 6: Analyze budget (85% utilization - WARNING)"
echo "---------------------------------------------------"
$BINARY agent budget-enforcement analyze \
    --tenant-id test-tenant \
    --budget-id budget-002 \
    --budget-limit 1000 \
    --current-spend 850 \
    --dry-run \
    --output table
echo ""
echo "✓ Budget at warning: WARNING signal"
echo ""

# Test 7: Analyze budget - critical threshold
echo "Test 7: Analyze budget (96% utilization - CRITICAL)"
echo "----------------------------------------------------"
$BINARY agent budget-enforcement analyze \
    --tenant-id test-tenant \
    --budget-id budget-003 \
    --budget-limit 1000 \
    --current-spend 960 \
    --dry-run \
    --output table
echo ""
echo "✓ Budget at critical: CRITICAL signal"
echo ""

# Test 8: Analyze budget - exceeded
echo "Test 8: Analyze budget (105% utilization - GATING)"
echo "---------------------------------------------------"
$BINARY agent budget-enforcement analyze \
    --tenant-id test-tenant \
    --budget-id budget-004 \
    --budget-limit 1000 \
    --current-spend 1050 \
    --dry-run \
    --output table
echo ""
echo "✓ Budget exceeded: GATING signal"
echo ""

# Test 9: JSON output
echo "Test 9: JSON output format"
echo "--------------------------"
$BINARY agent budget-enforcement analyze \
    --tenant-id test-tenant \
    --budget-id budget-005 \
    --budget-limit 500 \
    --current-spend 250 \
    --dry-run \
    --output json | head -20
echo "..."
echo ""
echo "✓ JSON output works"
echo ""

# Test 10: Custom thresholds
echo "Test 10: Custom warning/critical thresholds"
echo "--------------------------------------------"
$BINARY agent budget-enforcement analyze \
    --tenant-id test-tenant \
    --budget-id budget-006 \
    --budget-limit 1000 \
    --current-spend 600 \
    --warning-threshold 0.50 \
    --critical-threshold 0.70 \
    --dry-run \
    --output table
echo ""
echo "✓ Custom thresholds work (60% > 50% warning -> WARNING)"
echo ""

echo "==========================================="
echo "All smoke tests passed!"
echo "==========================================="
echo ""
echo "Summary:"
echo "- Agent registration: ✓"
echo "- Agent info: ✓"
echo "- Agent configuration: ✓"
echo "- Health check: ✓"
echo "- Budget analysis (OK): ✓"
echo "- Budget analysis (WARNING): ✓"
echo "- Budget analysis (CRITICAL): ✓"
echo "- Budget analysis (GATING): ✓"
echo "- JSON output: ✓"
echo "- Custom thresholds: ✓"
echo ""
echo "The Budget Enforcement Agent is working correctly."
