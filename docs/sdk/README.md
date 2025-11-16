# LLM-CostOps SDK Documentation

Welcome to the LLM-CostOps SDK documentation. This guide will help you integrate cost tracking, analysis, and forecasting into your LLM applications across multiple languages and platforms.

## What is LLM-CostOps?

LLM-CostOps is an enterprise-grade cost operations platform for LLM infrastructure. Track, analyze, and optimize costs across multiple LLM providers (OpenAI, Anthropic, Google Vertex AI, Azure OpenAI, AWS Bedrock, and more) with production-ready accuracy.

## Available SDKs

### Status

| Language   | Status       | Documentation | Package Manager |
|------------|--------------|---------------|-----------------|
| REST API   | ✅ Available | [Docs](api-reference/rest-api/README.md) | N/A |
| Python     | 🔄 Planned   | [Docs](api-reference/python/README.md) | PyPI |
| TypeScript | 🔄 Planned   | [Docs](api-reference/typescript/README.md) | npm |
| Go         | 🔄 Planned   | [Docs](api-reference/go/README.md) | Go Modules |
| Java       | 🔄 Planned   | [Docs](api-reference/java/README.md) | Maven Central |

## Quick Start

Choose your preferred integration method:

### 1. REST API (Available Now)

Use the REST API directly from any language:

```bash
curl -X POST https://api.llm-cost-ops.dev/api/v1/usage \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "organization_id": "org-123",
    "provider": "openai",
    "model_id": "gpt-4",
    "input_tokens": 1000,
    "output_tokens": 500,
    "total_tokens": 1500
  }'
```

[Read REST API Documentation →](api-reference/rest-api/README.md)

### 2. Python SDK (Coming Soon)

```python
from llm_cost_ops import CostOpsClient

client = CostOpsClient(api_key="YOUR_API_KEY")

# Track usage
usage = client.usage.submit(
    organization_id="org-123",
    provider="openai",
    model_id="gpt-4",
    input_tokens=1000,
    output_tokens=500
)

# Get costs
costs = client.costs.get(
    organization_id="org-123",
    start_date="2025-01-01",
    end_date="2025-01-31"
)

print(f"Total cost: ${costs.total_cost}")
```

[Read Python Documentation →](api-reference/python/README.md)

### 3. TypeScript SDK (Coming Soon)

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

const client = new CostOpsClient({
  apiKey: 'YOUR_API_KEY'
});

// Track usage
const usage = await client.usage.submit({
  organizationId: 'org-123',
  provider: 'openai',
  modelId: 'gpt-4',
  inputTokens: 1000,
  outputTokens: 500
});

// Get costs
const costs = await client.costs.get({
  organizationId: 'org-123',
  startDate: '2025-01-01',
  endDate: '2025-01-31'
});

console.log(`Total cost: $${costs.totalCost}`);
```

[Read TypeScript Documentation →](api-reference/typescript/README.md)

### 4. Go SDK (Coming Soon)

```go
import "github.com/llm-devops/llm-cost-ops-go"

client := costops.NewClient("YOUR_API_KEY")

// Track usage
usage, err := client.Usage.Submit(ctx, &costops.UsageRequest{
    OrganizationID: "org-123",
    Provider:       costops.ProviderOpenAI,
    ModelID:        "gpt-4",
    InputTokens:    1000,
    OutputTokens:   500,
})

// Get costs
costs, err := client.Costs.Get(ctx, &costops.CostsQuery{
    OrganizationID: "org-123",
    StartDate:      time.Date(2025, 1, 1, 0, 0, 0, 0, time.UTC),
    EndDate:        time.Date(2025, 1, 31, 0, 0, 0, 0, time.UTC),
})

fmt.Printf("Total cost: $%s\n", costs.TotalCost)
```

[Read Go Documentation →](api-reference/go/README.md)

## Key Features

### Cost Tracking
- **Real-time Usage Tracking**: Submit LLM usage data in real-time
- **Multi-Provider Support**: OpenAI, Anthropic, Google, Azure, AWS, and more
- **Flexible Pricing Models**: Per-token, per-request, and tiered pricing
- **High Precision**: 10-decimal precision for accurate financial calculations

### Cost Analysis
- **Aggregated Analytics**: Query costs by provider, model, project, or organization
- **Time-Series Data**: Analyze cost trends over time
- **Custom Breakdowns**: Group costs by any dimension
- **Export Capabilities**: CSV, JSON, Excel, JSON Lines

### Forecasting
- **Cost Predictions**: Forecast future costs using time-series models
- **Budget Alerts**: Get notified before exceeding budgets
- **Anomaly Detection**: Automatically detect unusual cost patterns
- **Trend Analysis**: Identify cost trends and seasonality

### Security & Compliance
- **API Key Authentication**: Secure API key management
- **JWT Support**: Token-based authentication
- **RBAC**: Role-based access control
- **Audit Logging**: Comprehensive audit trail
- **Multi-tenancy**: Organization and project-level isolation

## Getting Started Guides

### By Use Case
- [Track Your First LLM Call](getting-started/quickstart.md)
- [Analyze Costs Across Providers](guides/cost-analysis.md)
- [Set Up Budget Alerts](guides/budget-management.md)
- [Forecast Future Costs](guides/forecasting.md)
- [Export Cost Reports](guides/export-reports.md)

### By Framework
- [Django Integration](frameworks/django.md)
- [Flask Integration](frameworks/flask.md)
- [FastAPI Integration](frameworks/fastapi.md)
- [React Integration](frameworks/react.md)
- [Next.js Integration](frameworks/nextjs.md)
- [Spring Boot Integration](frameworks/spring-boot.md)

### By Language
- [Python Examples](examples/python/)
- [TypeScript Examples](examples/typescript/)
- [Go Examples](examples/go/)
- [Java Examples](examples/java/)
- [cURL Examples](examples/curl/)

## API Reference

### Endpoints

- **Usage**: Submit and query usage data
  - `POST /api/v1/usage` - Submit usage
  - `GET /api/v1/usage/history` - Get usage history

- **Costs**: Query cost data
  - `GET /api/v1/costs` - Get cost summary

- **Pricing**: Manage pricing tables
  - `GET /api/v1/pricing` - List pricing
  - `POST /api/v1/pricing` - Create pricing

- **Analytics**: Get analytics and insights
  - `GET /api/v1/analytics` - Get time-series analytics

- **Forecasting**: Cost predictions (Coming Soon)
  - `POST /api/v1/forecasts` - Generate forecast
  - `GET /api/v1/forecasts/{id}` - Get forecast results

- **Budgets**: Budget management (Coming Soon)
  - `POST /api/v1/budgets` - Create budget
  - `GET /api/v1/budgets` - List budgets
  - `GET /api/v1/budgets/{id}/alerts` - Get budget alerts

[View Full API Reference →](api-reference/rest-api/README.md)

## Support & Resources

- **Documentation**: https://docs.llm-cost-ops.dev
- **GitHub**: https://github.com/llm-devops/llm-cost-ops
- **Issues**: https://github.com/llm-devops/llm-cost-ops/issues
- **Discord**: https://discord.gg/llm-cost-ops
- **Email**: support@llm-cost-ops.dev

## Contributing

We welcome contributions! Please see our [Contributing Guide](../../CONTRIBUTING.md) for details.

## License

Apache 2.0 / MIT dual-licensed. See [LICENSE](../../LICENSE) for details.

## Next Steps

1. [Get Started with the Quickstart Guide](getting-started/quickstart.md)
2. [Set Up Authentication](getting-started/authentication.md)
3. [Submit Your First Usage Record](getting-started/first-usage.md)
4. [Query Costs](getting-started/query-costs.md)
5. [Explore Advanced Features](guides/)
