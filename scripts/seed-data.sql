-- LLM-CostOps Seed Data
-- Sample usage records for testing and demonstration

-- Sample OpenAI GPT-4 usage
INSERT OR IGNORE INTO usage_records (
    id, timestamp, provider, model_name, model_version, context_window,
    organization_id, project_id, user_id, prompt_tokens, completion_tokens,
    total_tokens, cached_tokens, reasoning_tokens, latency_ms, time_to_first_token_ms,
    tags, metadata, ingested_at, source_type, source_metadata
) VALUES
(
    '550e8400-e29b-41d4-a716-446655440001',
    '2025-01-15T10:00:00Z',
    'openai',
    'gpt-4',
    'gpt-4-0613',
    8192,
    'org-demo',
    'proj-demo',
    'user-demo',
    1500,
    800,
    2300,
    NULL,
    NULL,
    3200,
    150,
    '["production", "api"]',
    '{"request_id": "req-demo-001", "endpoint": "/v1/chat/completions"}',
    '2025-01-15T10:00:01Z',
    'api',
    '{"endpoint": "https://api.openai.com"}'
);

-- Sample Anthropic Claude usage
INSERT OR IGNORE INTO usage_records (
    id, timestamp, provider, model_name, model_version, context_window,
    organization_id, project_id, user_id, prompt_tokens, completion_tokens,
    total_tokens, cached_tokens, reasoning_tokens, latency_ms, time_to_first_token_ms,
    tags, metadata, ingested_at, source_type, source_metadata
) VALUES
(
    '550e8400-e29b-41d4-a716-446655440002',
    '2025-01-15T10:05:00Z',
    'anthropic',
    'claude-3-sonnet',
    '20240229',
    200000,
    'org-demo',
    'proj-demo',
    NULL,
    3000,
    1500,
    4500,
    1000,
    NULL,
    4100,
    200,
    '["production", "research"]',
    '{"request_id": "req-demo-002"}',
    '2025-01-15T10:05:01Z',
    'api',
    '{"endpoint": "https://api.anthropic.com"}'
);

-- Sample GPT-3.5-Turbo usage
INSERT OR IGNORE INTO usage_records (
    id, timestamp, provider, model_name, model_version, context_window,
    organization_id, project_id, user_id, prompt_tokens, completion_tokens,
    total_tokens, cached_tokens, reasoning_tokens, latency_ms, time_to_first_token_ms,
    tags, metadata, ingested_at, source_type, source_metadata
) VALUES
(
    '550e8400-e29b-41d4-a716-446655440003',
    '2025-01-15T10:10:00Z',
    'openai',
    'gpt-3.5-turbo',
    NULL,
    4096,
    'org-demo',
    'proj-staging',
    'user-demo',
    800,
    400,
    1200,
    NULL,
    NULL,
    1100,
    80,
    '["staging", "testing"]',
    '{}',
    '2025-01-15T10:10:01Z',
    'api',
    '{"endpoint": "https://api.openai.com"}'
);

-- Calculate and insert cost records for the sample data
INSERT OR IGNORE INTO cost_records (
    id, usage_id, provider, model, input_cost, output_cost, total_cost,
    currency, timestamp, organization_id, project_id
) VALUES
-- GPT-4 cost: 1500 * $10/1M + 800 * $30/1M = $0.015 + $0.024 = $0.039
(
    '650e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    'openai',
    'gpt-4',
    '0.0150000000',
    '0.0240000000',
    '0.0390000000',
    'USD',
    '2025-01-15T10:00:00Z',
    'org-demo',
    'proj-demo'
),
-- Claude-3-Sonnet cost: 3000 * $3/1M + 1500 * $15/1M = $0.009 + $0.0225 = $0.0315
-- With 1000 cached tokens @ 90% discount: 1000 * $3/1M * 0.9 = $0.0027 saved
-- Net: $0.0315 - $0.0027 = $0.0288
(
    '650e8400-e29b-41d4-a716-446655440002',
    '550e8400-e29b-41d4-a716-446655440002',
    'anthropic',
    'claude-3-sonnet',
    '0.0063000000',
    '0.0225000000',
    '0.0288000000',
    'USD',
    '2025-01-15T10:05:00Z',
    'org-demo',
    'proj-demo'
),
-- GPT-3.5-Turbo cost: 800 * $0.5/1M + 400 * $1.5/1M = $0.0004 + $0.0006 = $0.001
(
    '650e8400-e29b-41d4-a716-446655440003',
    '550e8400-e29b-41d4-a716-446655440003',
    'openai',
    'gpt-3.5-turbo',
    '0.0004000000',
    '0.0006000000',
    '0.0010000000',
    'USD',
    '2025-01-15T10:10:00Z',
    'org-demo',
    'proj-staging'
);

-- Verify data insertion
SELECT 'Usage Records Inserted:' AS info, COUNT(*) AS count FROM usage_records WHERE organization_id = 'org-demo'
UNION ALL
SELECT 'Cost Records Inserted:', COUNT(*) FROM cost_records WHERE organization_id = 'org-demo'
UNION ALL
SELECT 'Total Cost (Demo Org):', printf('$%.6f', SUM(CAST(total_cost AS REAL))) FROM cost_records WHERE organization_id = 'org-demo';
