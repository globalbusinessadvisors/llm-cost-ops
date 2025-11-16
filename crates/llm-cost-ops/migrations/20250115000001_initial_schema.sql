-- Initial schema for LLM-CostOps
-- SQLite compatible

-- Usage records table
CREATE TABLE IF NOT EXISTS usage_records (
    id TEXT PRIMARY KEY NOT NULL,
    timestamp TEXT NOT NULL,
    provider TEXT NOT NULL,
    model_name TEXT NOT NULL,
    model_version TEXT,
    context_window INTEGER NOT NULL,
    organization_id TEXT NOT NULL,
    project_id TEXT,
    user_id TEXT,
    prompt_tokens INTEGER NOT NULL,
    completion_tokens INTEGER NOT NULL,
    total_tokens INTEGER NOT NULL,
    cached_tokens INTEGER,
    reasoning_tokens INTEGER,
    latency_ms INTEGER,
    time_to_first_token_ms INTEGER,
    tags TEXT NOT NULL DEFAULT '[]',
    metadata TEXT NOT NULL DEFAULT '{}',
    ingested_at TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for usage_records
CREATE INDEX IF NOT EXISTS idx_usage_timestamp ON usage_records(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_usage_provider_model ON usage_records(provider, model_name);
CREATE INDEX IF NOT EXISTS idx_usage_organization ON usage_records(organization_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_usage_project ON usage_records(project_id, timestamp DESC);

-- Cost records table
CREATE TABLE IF NOT EXISTS cost_records (
    id TEXT PRIMARY KEY NOT NULL,
    usage_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    provider TEXT NOT NULL,
    model_name TEXT NOT NULL,
    input_cost TEXT NOT NULL,
    output_cost TEXT NOT NULL,
    total_cost TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    cost_model_id TEXT NOT NULL,
    pricing_structure TEXT NOT NULL,
    organization_id TEXT NOT NULL,
    project_id TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    calculated_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (usage_id) REFERENCES usage_records(id)
);

-- Indexes for cost_records
CREATE INDEX IF NOT EXISTS idx_cost_timestamp ON cost_records(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_cost_organization ON cost_records(organization_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_cost_usage_id ON cost_records(usage_id);
CREATE INDEX IF NOT EXISTS idx_cost_provider_model ON cost_records(provider, model_name);

-- Pricing tables
CREATE TABLE IF NOT EXISTS pricing_tables (
    id TEXT PRIMARY KEY NOT NULL,
    provider TEXT NOT NULL,
    model_name TEXT NOT NULL,
    effective_date TEXT NOT NULL,
    end_date TEXT,
    pricing_structure TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    region TEXT,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for pricing_tables
CREATE INDEX IF NOT EXISTS idx_pricing_provider_model ON pricing_tables(provider, model_name);
CREATE INDEX IF NOT EXISTS idx_pricing_effective_date ON pricing_tables(effective_date DESC);
CREATE UNIQUE INDEX IF NOT EXISTS idx_pricing_unique ON pricing_tables(provider, model_name, effective_date);

-- Insert default pricing for common models (OpenAI GPT-4)
INSERT OR IGNORE INTO pricing_tables (
    id, provider, model_name, effective_date, pricing_structure, currency
) VALUES (
    lower(hex(randomblob(16))),
    'openai',
    'gpt-4',
    '2024-01-01T00:00:00Z',
    '{"type":"per_token","input_price_per_million":"10.0","output_price_per_million":"30.0"}',
    'USD'
);

-- Insert default pricing for GPT-3.5
INSERT OR IGNORE INTO pricing_tables (
    id, provider, model_name, effective_date, pricing_structure, currency
) VALUES (
    lower(hex(randomblob(16))),
    'openai',
    'gpt-3.5-turbo',
    '2024-01-01T00:00:00Z',
    '{"type":"per_token","input_price_per_million":"0.5","output_price_per_million":"1.5"}',
    'USD'
);

-- Insert default pricing for Anthropic Claude
INSERT OR IGNORE INTO pricing_tables (
    id, provider, model_name, effective_date, pricing_structure, currency
) VALUES (
    lower(hex(randomblob(16))),
    'anthropic',
    'claude-3-opus',
    '2024-01-01T00:00:00Z',
    '{"type":"per_token","input_price_per_million":"15.0","output_price_per_million":"75.0"}',
    'USD'
);

INSERT OR IGNORE INTO pricing_tables (
    id, provider, model_name, effective_date, pricing_structure, currency
) VALUES (
    lower(hex(randomblob(16))),
    'anthropic',
    'claude-3-sonnet',
    '2024-01-01T00:00:00Z',
    '{"type":"per_token","input_price_per_million":"3.0","output_price_per_million":"15.0"}',
    'USD'
);
