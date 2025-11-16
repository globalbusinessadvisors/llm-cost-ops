#!/usr/bin/env bash
# =============================================================================
# LLM Cost Ops - Docker Build Script
# =============================================================================
# Description: Build Docker images with multi-arch support, caching, and
#              comprehensive tag management
# Usage: ./build.sh [OPTIONS]
# Version: 1.0.0
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuration & Defaults
# -----------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
IMAGE_NAME="${IMAGE_NAME:-llm-cost-ops}"
REGISTRY="${REGISTRY:-ghcr.io/yourusername}"
PLATFORMS="${PLATFORMS:-linux/amd64,linux/arm64}"
DOCKERFILE="${DOCKERFILE:-${PROJECT_ROOT}/Dockerfile}"
BUILD_CONTEXT="${BUILD_CONTEXT:-${PROJECT_ROOT}}"
CACHE_TYPE="${CACHE_TYPE:-inline}"
PUSH_IMAGE="${PUSH_IMAGE:-false}"
DRY_RUN="${DRY_RUN:-false}"

# Build metadata
VERSION="${VERSION:-$(git describe --tags --always --dirty 2>/dev/null || echo 'dev')}"
GIT_COMMIT="${GIT_COMMIT:-$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')}"
BUILD_DATE="${BUILD_DATE:-$(date -u +'%Y-%m-%dT%H:%M:%SZ')}"
VCS_URL="${VCS_URL:-$(git config --get remote.origin.url 2>/dev/null || echo 'unknown')}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

Build Docker images with multi-architecture support.

OPTIONS:
    -h, --help              Show this help message
    -n, --name NAME         Image name (default: ${IMAGE_NAME})
    -r, --registry REGISTRY Registry URL (default: ${REGISTRY})
    -t, --tag TAG           Additional tag (can be specified multiple times)
    -p, --platforms ARCH    Target platforms (default: ${PLATFORMS})
    -f, --file FILE         Dockerfile path (default: ${DOCKERFILE})
    -c, --context PATH      Build context (default: ${BUILD_CONTEXT})
    --push                  Push image after build
    --no-cache              Build without cache
    --cache-type TYPE       Cache type: inline, registry, local (default: ${CACHE_TYPE})
    --dry-run               Print commands without executing
    --build-arg ARG=VALUE   Pass build argument

EXAMPLES:
    # Build for current platform only
    ${0##*/} --platforms linux/amd64

    # Build and push multi-arch image
    ${0##*/} --push --platforms linux/amd64,linux/arm64

    # Build with custom tag
    ${0##*/} --tag v1.0.0 --tag latest

    # Build with build arguments
    ${0##*/} --build-arg RUST_VERSION=1.75 --build-arg FEATURES=full

ENVIRONMENT VARIABLES:
    IMAGE_NAME      Override default image name
    REGISTRY        Override default registry
    VERSION         Version tag (default: git describe)
    PLATFORMS       Target platforms
    PUSH_IMAGE      Auto-push after build (true/false)
    DRY_RUN         Dry run mode (true/false)

EOF
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check for Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed or not in PATH"
        exit 1
    fi

    # Check Docker version
    local docker_version
    docker_version=$(docker version --format '{{.Server.Version}}' 2>/dev/null || echo "unknown")
    log_info "Docker version: ${docker_version}"

    # Check for buildx
    if ! docker buildx version &> /dev/null; then
        log_error "Docker buildx is not available"
        log_info "Install with: docker buildx install"
        exit 1
    fi

    # Check for multi-arch support if building for multiple platforms
    if [[ "${PLATFORMS}" == *","* ]] || [[ "${PLATFORMS}" != *"$(uname -m)"* ]]; then
        if ! docker buildx ls | grep -q "linux/amd64.*linux/arm64" && \
           ! docker buildx ls | grep -q "linux/arm64.*linux/amd64"; then
            log_warn "Multi-arch builder not found. Creating one..."
            if [[ "${DRY_RUN}" == "false" ]]; then
                docker buildx create --name multiarch --driver docker-container --use 2>/dev/null || true
                docker buildx inspect --bootstrap
            fi
        fi
    fi

    # Check if Dockerfile exists
    if [[ ! -f "${DOCKERFILE}" ]]; then
        log_error "Dockerfile not found: ${DOCKERFILE}"
        exit 1
    fi

    # Check if build context exists
    if [[ ! -d "${BUILD_CONTEXT}" ]]; then
        log_error "Build context not found: ${BUILD_CONTEXT}"
        exit 1
    fi

    log_success "Prerequisites check passed"
}

generate_tags() {
    local tags=()
    local full_image="${REGISTRY}/${IMAGE_NAME}"

    # Add version tag
    tags+=("${full_image}:${VERSION}")

    # Add commit tag
    tags+=("${full_image}:${GIT_COMMIT}")

    # Add latest tag if this is a clean version (not dirty)
    if [[ "${VERSION}" != *"-dirty"* ]] && [[ "${VERSION}" != "dev" ]]; then
        tags+=("${full_image}:latest")
    fi

    # Add custom tags
    for tag in "${CUSTOM_TAGS[@]}"; do
        tags+=("${full_image}:${tag}")
    done

    echo "${tags[@]}"
}

build_image() {
    log_info "Starting Docker build..."
    log_info "Image: ${REGISTRY}/${IMAGE_NAME}"
    log_info "Version: ${VERSION}"
    log_info "Commit: ${GIT_COMMIT}"
    log_info "Platforms: ${PLATFORMS}"
    log_info "Build Date: ${BUILD_DATE}"

    # Generate tags
    local -a all_tags
    mapfile -t all_tags < <(generate_tags)

    log_info "Tags: ${all_tags[*]}"

    # Build docker buildx command
    local -a build_cmd=(
        "docker" "buildx" "build"
        "--platform" "${PLATFORMS}"
        "--file" "${DOCKERFILE}"
    )

    # Add tags
    for tag in "${all_tags[@]}"; do
        build_cmd+=("--tag" "${tag}")
    done

    # Add labels
    build_cmd+=(
        "--label" "org.opencontainers.image.created=${BUILD_DATE}"
        "--label" "org.opencontainers.image.version=${VERSION}"
        "--label" "org.opencontainers.image.revision=${GIT_COMMIT}"
        "--label" "org.opencontainers.image.source=${VCS_URL}"
        "--label" "org.opencontainers.image.title=${IMAGE_NAME}"
        "--label" "org.opencontainers.image.description=LLM Cost Operations and Analytics Platform"
    )

    # Add build arguments
    build_cmd+=(
        "--build-arg" "BUILD_DATE=${BUILD_DATE}"
        "--build-arg" "VCS_REF=${GIT_COMMIT}"
        "--build-arg" "VERSION=${VERSION}"
    )

    # Add custom build arguments
    for arg in "${BUILD_ARGS[@]}"; do
        build_cmd+=("--build-arg" "${arg}")
    done

    # Add cache configuration
    if [[ "${NO_CACHE}" == "false" ]]; then
        case "${CACHE_TYPE}" in
            inline)
                build_cmd+=("--cache-from" "type=registry,ref=${REGISTRY}/${IMAGE_NAME}:buildcache")
                build_cmd+=("--cache-to" "type=inline")
                ;;
            registry)
                build_cmd+=("--cache-from" "type=registry,ref=${REGISTRY}/${IMAGE_NAME}:buildcache")
                build_cmd+=("--cache-to" "type=registry,ref=${REGISTRY}/${IMAGE_NAME}:buildcache,mode=max")
                ;;
            local)
                build_cmd+=("--cache-from" "type=local,src=/tmp/.buildx-cache")
                build_cmd+=("--cache-to" "type=local,dest=/tmp/.buildx-cache-new,mode=max")
                ;;
        esac
    else
        build_cmd+=("--no-cache")
    fi

    # Add progress output
    build_cmd+=("--progress" "auto")

    # Push if requested
    if [[ "${PUSH_IMAGE}" == "true" ]]; then
        build_cmd+=("--push")
    else
        # Load to local docker if building for current platform only
        if [[ "${PLATFORMS}" != *","* ]]; then
            build_cmd+=("--load")
        fi
    fi

    # Add build context
    build_cmd+=("${BUILD_CONTEXT}")

    # Execute or print command
    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would execute:"
        echo "${build_cmd[@]}"
        return 0
    fi

    log_info "Executing: ${build_cmd[*]}"

    # Run build with progress
    if "${build_cmd[@]}"; then
        log_success "Build completed successfully!"

        # Move cache if using local cache
        if [[ "${CACHE_TYPE}" == "local" ]] && [[ "${NO_CACHE}" == "false" ]]; then
            rm -rf /tmp/.buildx-cache
            mv /tmp/.buildx-cache-new /tmp/.buildx-cache 2>/dev/null || true
        fi

        # Display image info
        if [[ "${PUSH_IMAGE}" == "false" ]] && [[ "${PLATFORMS}" != *","* ]]; then
            log_info "Image loaded to local Docker:"
            docker images | grep "${IMAGE_NAME}" | head -5
        fi

        return 0
    else
        log_error "Build failed!"
        return 1
    fi
}

show_summary() {
    cat << EOF

${GREEN}╔════════════════════════════════════════════════════════════════╗
║                    BUILD SUMMARY                               ║
╚════════════════════════════════════════════════════════════════╝${NC}

  ${BLUE}Image:${NC}      ${REGISTRY}/${IMAGE_NAME}
  ${BLUE}Version:${NC}    ${VERSION}
  ${BLUE}Commit:${NC}     ${GIT_COMMIT}
  ${BLUE}Platforms:${NC}  ${PLATFORMS}
  ${BLUE}Pushed:${NC}     ${PUSH_IMAGE}

  ${GREEN}✓${NC} Build completed successfully!

EOF
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    # Parse command line arguments
    local -a CUSTOM_TAGS=()
    local -a BUILD_ARGS=()
    local NO_CACHE="false"

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
            -r|--registry)
                REGISTRY="$2"
                shift 2
                ;;
            -t|--tag)
                CUSTOM_TAGS+=("$2")
                shift 2
                ;;
            -p|--platforms)
                PLATFORMS="$2"
                shift 2
                ;;
            -f|--file)
                DOCKERFILE="$2"
                shift 2
                ;;
            -c|--context)
                BUILD_CONTEXT="$2"
                shift 2
                ;;
            --push)
                PUSH_IMAGE="true"
                shift
                ;;
            --no-cache)
                NO_CACHE="true"
                shift
                ;;
            --cache-type)
                CACHE_TYPE="$2"
                shift 2
                ;;
            --dry-run)
                DRY_RUN="true"
                shift
                ;;
            --build-arg)
                BUILD_ARGS+=("$2")
                shift 2
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    # Run build process
    check_prerequisites
    build_image

    if [[ $? -eq 0 ]]; then
        show_summary
        exit 0
    else
        exit 1
    fi
}

# Run main function
main "$@"
