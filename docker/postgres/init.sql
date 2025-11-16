-- PostgreSQL initialization script for LLM Cost Ops
-- This script runs on first container startup

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "btree_gin";

-- Create additional databases for different environments
CREATE DATABASE llm_cost_ops_dev;
CREATE DATABASE llm_cost_ops_test;
CREATE DATABASE grafana;

-- Create application-specific schemas
\c llm_cost_ops_dev;
CREATE SCHEMA IF NOT EXISTS cost_ops;
CREATE SCHEMA IF NOT EXISTS analytics;
CREATE SCHEMA IF NOT EXISTS audit;

-- Grant permissions
GRANT ALL PRIVILEGES ON DATABASE llm_cost_ops_dev TO postgres;
GRANT ALL PRIVILEGES ON DATABASE llm_cost_ops_test TO postgres;
GRANT ALL PRIVILEGES ON DATABASE grafana TO postgres;

-- Create indexes for common queries (will be created by migrations, but prepare the database)
-- The actual tables will be created by sqlx migrations

-- Log initialization
DO $$
BEGIN
  RAISE NOTICE 'LLM Cost Ops database initialized successfully';
END $$;
