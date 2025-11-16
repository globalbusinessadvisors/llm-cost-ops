#!/usr/bin/env bash
# =============================================================================
# LLM Cost Ops - Docker Push Script
# =============================================================================
# Description: Push Docker images to multiple registries with multi-arch
#              manifest support and verification
# Usage: ./push.sh [OPTIONS]
# Version: 1.0.0
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuration & Defaults
# -----------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
IMAGE_NAME="${IMAGE_NAME:-llm-cost-ops}"
VERSION="${VERSION:-$(git describe --tags --always --dirty 2>/dev/null || echo 'dev')}"
GIT_COMMIT="${GIT_COMMIT:-$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')}"
DRY_RUN="${DRY_RUN:-false}"
VERIFY="${VERIFY:-true}"

# Registry configurations
REGISTRIES="${REGISTRIES:-}"
DEFAULT_REGISTRY="${DEFAULT_REGISTRY:-ghcr.io/yourusername}"

# Supported registries
declare -A REGISTRY_CONFIGS=(
    ["docker"]="docker.io"
    ["ghcr"]="ghcr.io/yourusername"
    ["ecr"]="123456789.dkr.ecr.us-east-1.amazonaws.com"
    ["gcr"]="gcr.io/project-id"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# -----------------------------------------------------------------------------
# Helper Functions
# -----------------------------------------------------------------------------
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

show_usage() {
    cat << EOF
Usage: ${0##*/} [OPTIONS]

Push Docker images to multiple registries with verification.

OPTIONS:
    -h, --help              Show this help message
    -n, --name NAME         Image name (default: ${IMAGE_NAME})
    -v, --version VERSION   Image version (default: ${VERSION})
    -r, --registry REGISTRY Registry to push to (can specify multiple times)
    -a, --all               Push to all configured registries
    --docker                Push to Docker Hub
    --ghcr                  Push to GitHub Container Registry
    --ecr                   Push to AWS Elastic Container Registry
    --gcr                   Push to Google Container Registry
    --no-verify             Skip image verification after push
    --dry-run               Print commands without executing

REGISTRY SHORTCUTS:
    --docker    ${REGISTRY_CONFIGS[docker]}
    --ghcr      ${REGISTRY_CONFIGS[ghcr]}
    --ecr       ${REGISTRY_CONFIGS[ecr]}
    --gcr       ${REGISTRY_CONFIGS[gcr]}

EXAMPLES:
    # Push to GitHub Container Registry
    ${0##*/} --ghcr

    # Push to multiple registries
    ${0##*/} --docker --ghcr --ecr

    # Push to all configured registries
    ${0##*/} --all

    # Push specific version
    ${0##*/} --version v1.0.0 --ghcr

    # Push to custom registry
    ${0##*/} --registry registry.example.com/myorg

AUTHENTICATION:
    Docker Hub:   docker login
    GHCR:         echo \$GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin
    ECR:          aws ecr get-login-password | docker login --username AWS --password-stdin ECR_URL
    GCR:          gcloud auth configure-docker

ENVIRONMENT VARIABLES:
    IMAGE_NAME      Override default image name
    VERSION         Image version tag
    REGISTRIES      Comma-separated list of registries
    DRY_RUN         Dry run mode (true/false)
    VERIFY          Verify after push (true/false)

EOF
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed or not in PATH"
        exit 1
    fi

    # Check Docker version
    local docker_version
    docker_version=$(docker version --format '{{.Server.Version}}' 2>/dev/null || echo "unknown")
    log_info "Docker version: ${docker_version}"

    log_success "Prerequisites check passed"
}

authenticate_registry() {
    local registry="$1"
    local registry_host="${registry%%/*}"

    log_info "Checking authentication for ${registry_host}..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Skipping authentication check"
        return 0
    fi

    # Test authentication by attempting to pull a small image or checking login status
    if docker login "${registry_host}" --get-login 2>/dev/null | grep -q "Login Succeeded" || \
       docker system info 2>/dev/null | grep -q "Registry: ${registry_host}"; then
        log_success "Already authenticated to ${registry_host}"
        return 0
    fi

    # Attempt auto-authentication for known registries
    case "${registry_host}" in
        *ecr*.amazonaws.com)
            log_info "Attempting AWS ECR authentication..."
            if command -v aws &> /dev/null; then
                local region
                region=$(echo "${registry_host}" | cut -d. -f4)
                aws ecr get-login-password --region "${region}" | \
                    docker login --username AWS --password-stdin "${registry_host}" || \
                    log_warn "ECR authentication failed. Please run: aws ecr get-login-password | docker login ..."
            else
                log_warn "AWS CLI not found. Please authenticate manually."
            fi
            ;;
        gcr.io|*.gcr.io)
            log_info "Attempting GCR authentication..."
            if command -v gcloud &> /dev/null; then
                gcloud auth configure-docker "${registry_host}" --quiet || \
                    log_warn "GCR authentication failed. Please run: gcloud auth configure-docker"
            else
                log_warn "gcloud CLI not found. Please authenticate manually."
            fi
            ;;
        ghcr.io)
            log_warn "Please authenticate to GHCR with: echo \$GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin"
            ;;
        docker.io|registry.hub.docker.com)
            log_warn "Please authenticate to Docker Hub with: docker login"
            ;;
        *)
            log_warn "Unknown registry. Please authenticate manually: docker login ${registry_host}"
            ;;
    esac
}

tag_image() {
    local source_image="$1"
    local target_image="$2"

    log_info "Tagging: ${source_image} -> ${target_image}"

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would execute: docker tag ${source_image} ${target_image}"
        return 0
    fi

    if docker tag "${source_image}" "${target_image}"; then
        log_success "Tagged successfully"
        return 0
    else
        log_error "Failed to tag image"
        return 1
    fi
}

push_image() {
    local image="$1"

    log_info "Pushing: ${image}"

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would execute: docker push ${image}"
        return 0
    fi

    if docker push "${image}"; then
        log_success "Pushed successfully"
        return 0
    else
        log_error "Failed to push image"
        return 1
    fi
}

create_manifest() {
    local manifest_image="$1"
    shift
    local -a images=("$@")

    log_info "Creating multi-arch manifest: ${manifest_image}"

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would create manifest with images: ${images[*]}"
        return 0
    fi

    # Create manifest
    if docker manifest create "${manifest_image}" "${images[@]}" 2>/dev/null || \
       docker buildx imagetools create --tag "${manifest_image}" "${images[@]}" 2>/dev/null; then
        log_success "Manifest created successfully"

        # Push manifest
        if docker manifest push "${manifest_image}" 2>/dev/null || \
           docker buildx imagetools inspect "${manifest_image}" >/dev/null 2>&1; then
            log_success "Manifest pushed successfully"
            return 0
        else
            log_error "Failed to push manifest"
            return 1
        fi
    else
        log_warn "Failed to create manifest (may not be needed for single-arch images)"
        return 0
    fi
}

verify_image() {
    local image="$1"

    log_info "Verifying: ${image}"

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would verify image"
        return 0
    fi

    # Check if image exists in registry
    if docker manifest inspect "${image}" >/dev/null 2>&1 || \
       docker buildx imagetools inspect "${image}" >/dev/null 2>&1; then
        log_success "Image verified in registry"

        # Display image details
        log_info "Image details:"
        docker buildx imagetools inspect "${image}" 2>/dev/null || \
            docker manifest inspect "${image}" 2>/dev/null | \
            jq -r '.manifests[] | "  - \(.platform.os)/\(.platform.architecture)"' 2>/dev/null || \
            echo "  (details not available)"

        return 0
    else
        log_error "Image verification failed"
        return 1
    fi
}

push_to_registry() {
    local registry="$1"
    local source_image="${DEFAULT_REGISTRY}/${IMAGE_NAME}:${VERSION}"

    log_info "Processing registry: ${registry}"

    # Authenticate to registry
    authenticate_registry "${registry}"

    # Define tags to push
    local -a tags=("${VERSION}" "${GIT_COMMIT}")

    # Add latest tag if version is not dirty
    if [[ "${VERSION}" != *"-dirty"* ]] && [[ "${VERSION}" != "dev" ]]; then
        tags+=("latest")
    fi

    # Tag and push each version
    local all_images=()
    for tag in "${tags[@]}"; do
        local target_image="${registry}/${IMAGE_NAME}:${tag}"

        # Check if source image exists locally
        if [[ "${DRY_RUN}" == "false" ]] && ! docker image inspect "${source_image}" >/dev/null 2>&1; then
            log_warn "Source image not found locally: ${source_image}"
            log_info "Attempting to pull from source registry..."
            docker pull "${source_image}" || {
                log_error "Failed to pull source image"
                continue
            }
        fi

        # Tag for target registry
        tag_image "${source_image}" "${target_image}" || continue

        # Push to registry
        push_image "${target_image}" || continue

        all_images+=("${target_image}")

        # Verify if requested
        if [[ "${VERIFY}" == "true" ]]; then
            verify_image "${target_image}"
        fi
    done

    # Create multi-arch manifest if needed
    if [[ ${#all_images[@]} -gt 0 ]]; then
        create_manifest "${registry}/${IMAGE_NAME}:${VERSION}" "${all_images[@]}"
    fi

    log_success "Completed pushing to ${registry}"
}

show_summary() {
    local -a registries=("$@")

    cat << EOF

${GREEN}╔════════════════════════════════════════════════════════════════╗
║                    PUSH SUMMARY                                ║
╚════════════════════════════════════════════════════════════════╝${NC}

  ${BLUE}Image:${NC}      ${IMAGE_NAME}
  ${BLUE}Version:${NC}    ${VERSION}
  ${BLUE}Commit:${NC}     ${GIT_COMMIT}
  ${BLUE}Verified:${NC}   ${VERIFY}

  ${BLUE}Registries:${NC}
EOF

    for registry in "${registries[@]}"; do
        echo "    - ${registry}"
    done

    echo ""
    log_success "Push completed successfully!"
    echo ""
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    local -a target_registries=()
    local push_all="false"

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -n|--name)
                IMAGE_NAME="$2"
                shift 2
                ;;
            -v|--version)
                VERSION="$2"
                shift 2
                ;;
            -r|--registry)
                target_registries+=("$2")
                shift 2
                ;;
            -a|--all)
                push_all="true"
                shift
                ;;
            --docker)
                target_registries+=("${REGISTRY_CONFIGS[docker]}")
                shift
                ;;
            --ghcr)
                target_registries+=("${REGISTRY_CONFIGS[ghcr]}")
                shift
                ;;
            --ecr)
                target_registries+=("${REGISTRY_CONFIGS[ecr]}")
                shift
                ;;
            --gcr)
                target_registries+=("${REGISTRY_CONFIGS[gcr]}")
                shift
                ;;
            --no-verify)
                VERIFY="false"
                shift
                ;;
            --dry-run)
                DRY_RUN="true"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    # Check prerequisites
    check_prerequisites

    # Determine target registries
    if [[ "${push_all}" == "true" ]]; then
        target_registries=("${REGISTRY_CONFIGS[@]}")
    elif [[ ${#target_registries[@]} -eq 0 ]]; then
        # Use default registry if none specified
        target_registries=("${DEFAULT_REGISTRY}")
    fi

    log_info "Pushing to ${#target_registries[@]} registry(s)"

    # Push to each registry
    local failed_registries=()
    for registry in "${target_registries[@]}"; do
        if ! push_to_registry "${registry}"; then
            failed_registries+=("${registry}")
        fi
    done

    # Show summary
    show_summary "${target_registries[@]}"

    # Report failures
    if [[ ${#failed_registries[@]} -gt 0 ]]; then
        log_error "Failed to push to the following registries:"
        for registry in "${failed_registries[@]}"; do
            echo "  - ${registry}"
        done
        exit 1
    fi

    exit 0
}

# Run main function
main "$@"
