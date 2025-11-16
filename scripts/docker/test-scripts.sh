#!/usr/bin/env bash
# =============================================================================
# LLM Cost Ops - Script Testing Suite
# =============================================================================
# Description: Test all deployment scripts for functionality and consistency
# Usage: ./test-scripts.sh [OPTIONS]
# Version: 1.0.0
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuration
# -----------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_RESULTS_FILE="${SCRIPT_DIR}/test-results-$(date +%Y%m%d-%H%M%S).log"
PASSED=0
FAILED=0
SKIPPED=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# -----------------------------------------------------------------------------
# Helper Functions
# -----------------------------------------------------------------------------
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" | tee -a "${TEST_RESULTS_FILE}"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $*" | tee -a "${TEST_RESULTS_FILE}"
    ((PASSED++))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $*" | tee -a "${TEST_RESULTS_FILE}"
    ((FAILED++))
}

log_skip() {
    echo -e "${YELLOW}[SKIP]${NC} $*" | tee -a "${TEST_RESULTS_FILE}"
    ((SKIPPED++))
}

show_usage() {
    cat << EOF
Usage: ${0##*/} [OPTIONS]

Test all deployment scripts for functionality.

OPTIONS:
    -h, --help              Show this help message
    -v, --verbose           Verbose output
    --quick                 Quick tests only (skip long-running tests)
    --script NAME           Test specific script only

EXAMPLES:
    # Run all tests
    ${0##*/}

    # Test specific script
    ${0##*/} --script build.sh

    # Quick tests only
    ${0##*/} --quick

EOF
}

# -----------------------------------------------------------------------------
# Test Functions
# -----------------------------------------------------------------------------

test_script_exists() {
    local script="$1"
    if [[ -f "${SCRIPT_DIR}/${script}" ]]; then
        log_success "Script exists: ${script}"
        return 0
    else
        log_fail "Script missing: ${script}"
        return 1
    fi
}

test_script_executable() {
    local script="$1"
    if [[ -x "${SCRIPT_DIR}/${script}" ]]; then
        log_success "Script executable: ${script}"
        return 0
    else
        log_fail "Script not executable: ${script}"
        return 1
    fi
}

test_help_flag() {
    local script="$1"
    log_info "Testing --help flag for ${script}..."

    if "${SCRIPT_DIR}/${script}" --help >/dev/null 2>&1; then
        log_success "${script} --help works"
        return 0
    else
        log_fail "${script} --help failed"
        return 1
    fi
}

test_dry_run_support() {
    local script="$1"
    log_info "Testing --dry-run support for ${script}..."

    # Scripts that support dry-run
    case "${script}" in
        build.sh|push.sh|deploy-*.sh|cleanup.sh|migrate.sh|backup.sh)
            if "${SCRIPT_DIR}/${script}" --dry-run >/dev/null 2>&1; then
                log_success "${script} supports --dry-run"
                return 0
            else
                log_fail "${script} --dry-run failed"
                return 1
            fi
            ;;
        *)
            log_skip "${script} dry-run test (not applicable)"
            return 0
            ;;
    esac
}

test_error_handling() {
    local script="$1"
    log_info "Testing error handling for ${script}..."

    # Test with invalid option
    if "${SCRIPT_DIR}/${script}" --invalid-option >/dev/null 2>&1; then
        log_fail "${script} accepted invalid option"
        return 1
    else
        log_success "${script} rejects invalid options"
        return 0
    fi
}

test_shebang() {
    local script="$1"
    local shebang
    shebang=$(head -n 1 "${SCRIPT_DIR}/${script}")

    if [[ "${shebang}" == "#!/usr/bin/env bash" ]] || [[ "${shebang}" == "#!/bin/bash" ]]; then
        log_success "${script} has valid shebang"
        return 0
    else
        log_fail "${script} has invalid shebang: ${shebang}"
        return 1
    fi
}

test_set_options() {
    local script="$1"

    # Check for set -e (exit on error)
    if grep -q "set -e" "${SCRIPT_DIR}/${script}"; then
        log_success "${script} uses 'set -e'"
    else
        log_fail "${script} missing 'set -e'"
        return 1
    fi

    # Check for set -u (error on undefined variables)
    if grep -q "set -u" "${SCRIPT_DIR}/${script}"; then
        log_success "${script} uses 'set -u'"
    else
        log_fail "${script} missing 'set -u'"
        return 1
    fi

    # Check for set -o pipefail
    if grep -q "set -o pipefail" "${SCRIPT_DIR}/${script}"; then
        log_success "${script} uses 'set -o pipefail'"
    else
        log_fail "${script} missing 'set -o pipefail'"
        return 1
    fi

    return 0
}

test_documentation() {
    local script="$1"

    # Check for header documentation
    if head -n 10 "${SCRIPT_DIR}/${script}" | grep -q "Description:"; then
        log_success "${script} has header documentation"
    else
        log_fail "${script} missing header documentation"
        return 1
    fi

    # Check for usage function
    if grep -q "show_usage()" "${SCRIPT_DIR}/${script}"; then
        log_success "${script} has usage function"
    else
        log_fail "${script} missing usage function"
        return 1
    fi

    return 0
}

test_logging_functions() {
    local script="$1"

    # Check for log functions
    local required_funcs=("log_info" "log_success" "log_warn" "log_error")

    for func in "${required_funcs[@]}"; do
        if grep -q "${func}()" "${SCRIPT_DIR}/${script}"; then
            log_success "${script} has ${func}()"
        else
            log_fail "${script} missing ${func}()"
            return 1
        fi
    done

    return 0
}

test_script_syntax() {
    local script="$1"
    log_info "Testing syntax for ${script}..."

    if bash -n "${SCRIPT_DIR}/${script}" 2>/dev/null; then
        log_success "${script} syntax is valid"
        return 0
    else
        log_fail "${script} has syntax errors"
        return 1
    fi
}

# -----------------------------------------------------------------------------
# Test Suites
# -----------------------------------------------------------------------------

test_all_scripts() {
    local -a scripts=(
        "build.sh"
        "push.sh"
        "deploy-dev.sh"
        "deploy-prod.sh"
        "deploy-k8s.sh"
        "deploy-helm.sh"
        "cleanup.sh"
        "migrate.sh"
        "logs.sh"
        "backup.sh"
    )

    log_info "Testing all scripts..."
    echo "" | tee -a "${TEST_RESULTS_FILE}"

    for script in "${scripts[@]}"; do
        echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${CYAN}Testing: ${script}${NC}"
        echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

        test_script_exists "${script}" || continue
        test_script_executable "${script}"
        test_shebang "${script}"
        test_syntax "${script}"
        test_set_options "${script}"
        test_documentation "${script}"
        test_logging_functions "${script}"
        test_help_flag "${script}"
        test_dry_run_support "${script}"
        test_error_handling "${script}"

        echo "" | tee -a "${TEST_RESULTS_FILE}"
    done
}

test_specific_script() {
    local script="$1"

    log_info "Testing ${script}..."
    echo "" | tee -a "${TEST_RESULTS_FILE}"

    test_script_exists "${script}" || return 1
    test_script_executable "${script}"
    test_shebang "${script}"
    test_syntax "${script}"
    test_set_options "${script}"
    test_documentation "${script}"
    test_logging_functions "${script}"
    test_help_flag "${script}"
    test_dry_run_support "${script}"
    test_error_handling "${script}"
}

test_integration() {
    log_info "Running integration tests..."

    # Test build -> push workflow
    log_info "Testing build -> push workflow..."
    if "${SCRIPT_DIR}/build.sh" --dry-run >/dev/null 2>&1 && \
       "${SCRIPT_DIR}/push.sh" --dry-run >/dev/null 2>&1; then
        log_success "Build -> Push workflow works"
    else
        log_fail "Build -> Push workflow failed"
    fi

    # Test deployment workflows
    log_info "Testing deployment workflows..."
    if "${SCRIPT_DIR}/deploy-dev.sh" --help >/dev/null 2>&1 && \
       "${SCRIPT_DIR}/deploy-prod.sh" --help >/dev/null 2>&1; then
        log_success "Deployment scripts available"
    else
        log_fail "Deployment scripts failed"
    fi
}

test_documentation_files() {
    log_info "Testing documentation files..."

    if [[ -f "${SCRIPT_DIR}/README.md" ]]; then
        log_success "README.md exists"
    else
        log_fail "README.md missing"
    fi

    if [[ -f "${SCRIPT_DIR}/QUICK_REFERENCE.md" ]]; then
        log_success "QUICK_REFERENCE.md exists"
    else
        log_fail "QUICK_REFERENCE.md missing"
    fi
}

# -----------------------------------------------------------------------------
# Summary and Reporting
# -----------------------------------------------------------------------------

show_summary() {
    local total=$((PASSED + FAILED + SKIPPED))

    cat << EOF | tee -a "${TEST_RESULTS_FILE}"

${CYAN}╔════════════════════════════════════════════════════════════════╗
║                      TEST SUMMARY                              ║
╚════════════════════════════════════════════════════════════════╝${NC}

  ${GREEN}Passed:${NC}  ${PASSED}
  ${RED}Failed:${NC}  ${FAILED}
  ${YELLOW}Skipped:${NC} ${SKIPPED}
  ${BLUE}Total:${NC}   ${total}

  ${BLUE}Results saved to:${NC} ${TEST_RESULTS_FILE}

EOF

    if [[ ${FAILED} -eq 0 ]]; then
        echo -e "${GREEN}✓ All tests passed!${NC}" | tee -a "${TEST_RESULTS_FILE}"
        return 0
    else
        echo -e "${RED}✗ Some tests failed${NC}" | tee -a "${TEST_RESULTS_FILE}"
        return 1
    fi
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    local specific_script=""
    local quick_mode="false"

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -v|--verbose)
                set -x
                shift
                ;;
            --quick)
                quick_mode="true"
                shift
                ;;
            --script)
                specific_script="$2"
                shift 2
                ;;
            *)
                echo "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    # Start testing
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║        LLM Cost Ops - Script Testing Suite                    ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
    echo ""

    log_info "Starting tests at $(date)"
    echo "" | tee -a "${TEST_RESULTS_FILE}"

    # Run tests
    if [[ -n "${specific_script}" ]]; then
        test_specific_script "${specific_script}"
    else
        test_all_scripts
        test_documentation_files

        if [[ "${quick_mode}" != "true" ]]; then
            test_integration
        fi
    fi

    # Show summary
    show_summary
}

# Run main function
main "$@"
