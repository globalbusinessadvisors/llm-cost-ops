#!/bin/bash
set -e

# LLM-CostOps Database Initialization Script
# This script creates the database, runs migrations, and generates SQLx query cache

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default database URL
DATABASE_URL="${DATABASE_URL:-sqlite:cost-ops.db}"

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}LLM-CostOps Database Initialization${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "Database URL: ${YELLOW}$DATABASE_URL${NC}"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust/Cargo is not installed${NC}"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check if sqlx-cli is installed
if ! command -v sqlx &> /dev/null; then
    echo -e "${YELLOW}sqlx-cli not found. Installing...${NC}"
    cargo install sqlx-cli --no-default-features --features sqlite
    echo -e "${GREEN}✓ sqlx-cli installed successfully${NC}"
else
    echo -e "${GREEN}✓ sqlx-cli is already installed${NC}"
fi

# Export DATABASE_URL for subsequent commands
export DATABASE_URL

# Create database
echo ""
echo -e "${YELLOW}Creating database...${NC}"
if sqlx database create 2>/dev/null; then
    echo -e "${GREEN}✓ Database created successfully${NC}"
else
    echo -e "${YELLOW}! Database already exists, skipping creation${NC}"
fi

# Run migrations
echo ""
echo -e "${YELLOW}Running database migrations...${NC}"
sqlx migrate run
echo -e "${GREEN}✓ Migrations completed successfully${NC}"

# Generate SQLx query cache for offline compilation
echo ""
echo -e "${YELLOW}Generating SQLx query cache...${NC}"
cargo sqlx prepare --workspace
echo -e "${GREEN}✓ Query cache generated successfully${NC}"

# Optional: Load seed data if file exists
SEED_FILE="scripts/seed-data.sql"
if [ -f "$SEED_FILE" ]; then
    echo ""
    echo -e "${YELLOW}Loading seed data...${NC}"
    sqlite3 "$DATABASE_URL" < "$SEED_FILE"
    echo -e "${GREEN}✓ Seed data loaded successfully${NC}"
fi

# Verify setup
echo ""
echo -e "${YELLOW}Verifying database setup...${NC}"
TABLE_COUNT=$(sqlite3 "$DATABASE_URL" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';")
echo -e "  Tables created: ${GREEN}$TABLE_COUNT${NC}"

PRICING_COUNT=$(sqlite3 "$DATABASE_URL" "SELECT COUNT(*) FROM pricing_tables;")
echo -e "  Default pricing entries: ${GREEN}$PRICING_COUNT${NC}"

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Database initialization complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "Next steps:"
echo -e "  1. Build the project: ${YELLOW}cargo build --release${NC}"
echo -e "  2. Run tests: ${YELLOW}cargo test${NC}"
echo -e "  3. Start using: ${YELLOW}cargo run -- --help${NC}"
echo ""
echo -e "Examples:"
echo -e "  - List pricing: ${YELLOW}cargo run -- pricing list${NC}"
echo -e "  - Ingest data: ${YELLOW}cargo run -- ingest --file examples/usage.json${NC}"
echo -e "  - Query costs: ${YELLOW}cargo run -- query --range last-24-hours${NC}"
echo ""
