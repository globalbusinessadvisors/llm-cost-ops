.PHONY: all build test test-race test-coverage vet fmt lint clean help

# Variables
GO := go
GOFLAGS := -v
COVERAGE_FILE := coverage.txt
COVERAGE_HTML := coverage.html

# Default target
all: fmt vet test

# Build the SDK
build:
	@echo "Building Go SDK..."
	$(GO) build $(GOFLAGS) ./...

# Run tests
test:
	@echo "Running tests..."
	$(GO) test $(GOFLAGS) ./...

# Run tests with race detector
test-race:
	@echo "Running tests with race detector..."
	$(GO) test -race $(GOFLAGS) ./...

# Run tests with coverage
test-coverage:
	@echo "Running tests with coverage..."
	$(GO) test -race -coverprofile=$(COVERAGE_FILE) -covermode=atomic ./...
	@echo "Generating coverage report..."
	$(GO) tool cover -html=$(COVERAGE_FILE) -o $(COVERAGE_HTML)
	@echo "Coverage report generated: $(COVERAGE_HTML)"

# Run go vet
vet:
	@echo "Running go vet..."
	$(GO) vet ./...

# Format code
fmt:
	@echo "Formatting code..."
	$(GO) fmt ./...

# Run golangci-lint (if available)
lint:
	@echo "Running linter..."
	@if command -v golangci-lint >/dev/null 2>&1; then \
		golangci-lint run ./...; \
	else \
		echo "golangci-lint not installed. Install with: go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest"; \
	fi

# Clean build artifacts
clean:
	@echo "Cleaning..."
	rm -f $(COVERAGE_FILE) $(COVERAGE_HTML)
	$(GO) clean

# Update dependencies
deps:
	@echo "Updating dependencies..."
	$(GO) mod tidy
	$(GO) mod download

# Run all quality checks
quality: fmt vet lint test-race

# Display help
help:
	@echo "Available targets:"
	@echo "  all           - Run fmt, vet, and test (default)"
	@echo "  build         - Build the SDK"
	@echo "  test          - Run tests"
	@echo "  test-race     - Run tests with race detector"
	@echo "  test-coverage - Run tests with coverage and generate HTML report"
	@echo "  vet           - Run go vet"
	@echo "  fmt           - Format code"
	@echo "  lint          - Run golangci-lint"
	@echo "  clean         - Clean build artifacts"
	@echo "  deps          - Update dependencies"
	@echo "  quality       - Run all quality checks"
	@echo "  help          - Display this help message"
