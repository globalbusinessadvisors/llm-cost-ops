-- Initial schema for LLM-CostOps
-- PostgreSQL version

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Usage records table
CREATE TABLE IF NOT EXISTS usage_records (
    id UUID PRIMARY KEY NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    provider TEXT NOT NULL,
    model_name TEXT NOT NULL,
    model_version TEXT,
    context_window BIGINT NOT NULL,
    organization_id TEXT NOT NULL,
    project_id TEXT,
    user_id TEXT,
    prompt_tokens BIGINT NOT NULL,
    completion_tokens BIGINT NOT NULL,
    total_tokens BIGINT NOT NULL,
    cached_tokens BIGINT,
    reasoning_tokens BIGINT,
    latency_ms BIGINT,
    time_to_first_token_ms BIGINT,
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    ingested_at TIMESTAMPTZ NOT NULL,
    source_type TEXT NOT NULL,
    source_metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for usage_records
CREATE INDEX IF NOT EXISTS idx_usage_timestamp ON usage_records(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_usage_provider_model ON usage_records(provider, model_name);
CREATE INDEX IF NOT EXISTS idx_usage_organization ON usage_records(organization_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_usage_project ON usage_records(project_id, timestamp DESC) WHERE project_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_usage_tags ON usage_records USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_usage_metadata ON usage_records USING GIN(metadata);

-- Cost records table
CREATE TABLE IF NOT EXISTS cost_records (
    id UUID PRIMARY KEY NOT NULL,
    usage_id UUID NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    provider TEXT NOT NULL,
    model_name TEXT NOT NULL,
    input_cost NUMERIC(20, 10) NOT NULL,
    output_cost NUMERIC(20, 10) NOT NULL,
    total_cost NUMERIC(20, 10) NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    cost_model_id UUID NOT NULL,
    pricing_structure JSONB NOT NULL,
    organization_id TEXT NOT NULL,
    project_id TEXT,
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    calculated_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (usage_id) REFERENCES usage_records(id) ON DELETE CASCADE
);

-- Indexes for cost_records
CREATE INDEX IF NOT EXISTS idx_cost_timestamp ON cost_records(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_cost_organization ON cost_records(organization_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_cost_usage_id ON cost_records(usage_id);
CREATE INDEX IF NOT EXISTS idx_cost_provider_model ON cost_records(provider, model_name);
CREATE INDEX IF NOT EXISTS idx_cost_tags ON cost_records USING GIN(tags);

-- Pricing tables
CREATE TABLE IF NOT EXISTS pricing_tables (
    id UUID PRIMARY KEY NOT NULL,
    provider TEXT NOT NULL,
    model_name TEXT NOT NULL,
    effective_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ,
    pricing_structure JSONB NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    region TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for pricing_tables
CREATE INDEX IF NOT EXISTS idx_pricing_provider_model ON pricing_tables(provider, model_name);
CREATE INDEX IF NOT EXISTS idx_pricing_effective_date ON pricing_tables(effective_date DESC);
CREATE UNIQUE INDEX IF NOT EXISTS idx_pricing_unique ON pricing_tables(provider, model_name, effective_date);
CREATE INDEX IF NOT EXISTS idx_pricing_metadata ON pricing_tables USING GIN(metadata);

-- Insert default pricing for common models (OpenAI GPT-4)
INSERT INTO pricing_tables (
    id, provider, model_name, effective_date, pricing_structure, currency
) VALUES (
    gen_random_uuid(),
    'openai',
    'gpt-4',
    '2024-01-01T00:00:00Z'::timestamptz,
    '{"type":"per_token","input_price_per_million":"10.0","output_price_per_million":"30.0"}'::jsonb,
    'USD'
) ON CONFLICT (provider, model_name, effective_date) DO NOTHING;

-- Insert default pricing for GPT-3.5
INSERT INTO pricing_tables (
    id, provider, model_name, effective_date, pricing_structure, currency
) VALUES (
    gen_random_uuid(),
    'openai',
    'gpt-3.5-turbo',
    '2024-01-01T00:00:00Z'::timestamptz,
    '{"type":"per_token","input_price_per_million":"0.5","output_price_per_million":"1.5"}'::jsonb,
    'USD'
) ON CONFLICT (provider, model_name, effective_date) DO NOTHING;

-- Insert default pricing for Anthropic Claude Opus
INSERT INTO pricing_tables (
    id, provider, model_name, effective_date, pricing_structure, currency
) VALUES (
    gen_random_uuid(),
    'anthropic',
    'claude-3-opus',
    '2024-01-01T00:00:00Z'::timestamptz,
    '{"type":"per_token","input_price_per_million":"15.0","output_price_per_million":"75.0"}'::jsonb,
    'USD'
) ON CONFLICT (provider, model_name, effective_date) DO NOTHING;

-- Insert default pricing for Anthropic Claude Sonnet
INSERT INTO pricing_tables (
    id, provider, model_name, effective_date, pricing_structure, currency
) VALUES (
    gen_random_uuid(),
    'anthropic',
    'claude-3-sonnet',
    '2024-01-01T00:00:00Z'::timestamptz,
    '{"type":"per_token","input_price_per_million":"3.0","output_price_per_million":"15.0"}'::jsonb,
    'USD'
) ON CONFLICT (provider, model_name, effective_date) DO NOTHING;

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger to auto-update updated_at on pricing_tables
CREATE TRIGGER update_pricing_tables_updated_at
    BEFORE UPDATE ON pricing_tables
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create materialized view for cost aggregations (performance optimization)
CREATE MATERIALIZED VIEW IF NOT EXISTS cost_summary_by_org_day AS
SELECT
    organization_id,
    DATE_TRUNC('day', timestamp) as day,
    provider,
    model_name,
    SUM(total_cost) as total_cost,
    SUM(prompt_tokens + completion_tokens) as total_tokens,
    COUNT(*) as request_count,
    currency
FROM cost_records cr
JOIN usage_records ur ON cr.usage_id = ur.id
GROUP BY organization_id, DATE_TRUNC('day', timestamp), provider, model_name, currency;

CREATE INDEX IF NOT EXISTS idx_cost_summary_org_day ON cost_summary_by_org_day(organization_id, day DESC);

-- Function to refresh materialized view
CREATE OR REPLACE FUNCTION refresh_cost_summary()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY cost_summary_by_org_day;
END;
$$ LANGUAGE plpgsql;
