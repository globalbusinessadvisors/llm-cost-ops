# Frequently Asked Questions (FAQ)

**Version:** 1.0.0
**Last Updated:** 2025-11-16

Common questions and answers about the LLM Cost Ops platform, covering general usage, pricing, security, integrations, performance, and troubleshooting.

---

## Table of Contents

- [General Questions](#general-questions)
- [Pricing and Billing](#pricing-and-billing)
- [Security and Compliance](#security-and-compliance)
- [Integration Questions](#integration-questions)
- [Performance Questions](#performance-questions)
- [Feature Questions](#feature-questions)
- [Deployment Questions](#deployment-questions)
- [Troubleshooting](#troubleshooting)
- [Support and Community](#support-and-community)

---

## General Questions

### What is LLM Cost Ops?

LLM Cost Ops is an enterprise-grade platform for tracking, analyzing, and optimizing costs across multiple LLM (Large Language Model) providers. It provides:

- **Multi-provider support**: OpenAI, Anthropic, Google, Azure, AWS Bedrock, Cohere, Mistral
- **Cost tracking**: High-precision cost calculations with 10-decimal accuracy
- **Analytics**: Detailed cost breakdowns and trends
- **Forecasting**: Predict future costs with ML-based models
- **Budgeting**: Set budgets and receive alerts
- **Reporting**: Automated reports and exports

### Which LLM providers are supported?

**Currently Supported:**
- OpenAI (GPT-4, GPT-3.5, GPT-4 Turbo, o1, etc.)
- Anthropic (Claude 3 Opus, Sonnet, Haiku)
- Google Vertex AI (Gemini, PaLM)
- Azure OpenAI
- AWS Bedrock
- Cohere
- Mistral AI

**Coming Soon:**
- Replicate
- Hugging Face
- Local/self-hosted models

### How accurate are the cost calculations?

LLM Cost Ops uses `rust_decimal` for financial calculations, providing:

- **10-decimal precision**: Accurate to $0.0000000001
- **No floating-point errors**: Decimal arithmetic for financial accuracy
- **Verified against provider invoices**: Regular validation
- **Cached token support**: Accurate discounts for providers that support it

### Can I use LLM Cost Ops with my existing infrastructure?

Yes! LLM Cost Ops integrates easily with existing infrastructure:

- **API-based integration**: RESTful API for any language
- **SDKs**: Python, Node.js, Go, Rust
- **Webhooks**: Real-time notifications
- **Export formats**: CSV, JSON, Excel, Parquet
- **Database support**: PostgreSQL, SQLite
- **Container-ready**: Docker and Kubernetes support

### Is there a free tier?

Yes! LLM Cost Ops offers:

- **Free tier**: Up to 100 requests/hour, SQLite database, basic features
- **Developer tier**: $49/month - 1,000 requests/hour, PostgreSQL support
- **Professional tier**: $199/month - 10,000 requests/hour, forecasting, reports
- **Enterprise tier**: Custom pricing - Unlimited requests, dedicated support, SLA

### How do I get started?

**Quick Start:**

1. **Install:**
```bash
curl -L https://github.com/llm-cost-ops/releases/latest/download/cost-ops-linux-amd64 -o cost-ops
chmod +x cost-ops
sudo mv cost-ops /usr/local/bin/
```

2. **Initialize:**
```bash
cost-ops init --database-url sqlite:cost-ops.db
```

3. **Add pricing:**
```bash
cost-ops pricing add --provider openai --model gpt-4 --input-price 10.0 --output-price 30.0
```

4. **Submit usage:**
```bash
cost-ops ingest --file usage.json
```

5. **Query costs:**
```bash
cost-ops query --range last-7-days
```

---

## Pricing and Billing

### How is LLM Cost Ops priced?

**Pricing Tiers:**

| Tier | Price | Requests/Hour | Features |
|------|-------|--------------|----------|
| Free | $0 | 100 | Basic tracking, SQLite |
| Developer | $49/mo | 1,000 | PostgreSQL, API, exports |
| Professional | $199/mo | 10,000 | Forecasting, reports, RBAC |
| Enterprise | Custom | Unlimited | SLA, dedicated support, SSO |

**Additional Costs:**
- Export storage: $0.023/GB/month (S3 pricing)
- Email reports: $0.10/report
- Additional users: $10/user/month (Professional+)

### How does billing work?

- **Monthly billing**: Charged on the 1st of each month
- **Annual billing**: 20% discount when paid annually
- **Usage-based**: Overage charges for exceeding limits
- **Prorated**: Upgrades/downgrades are prorated

### Can I change plans?

Yes! You can:
- **Upgrade** anytime (immediate access to new features)
- **Downgrade** at end of billing cycle
- **Cancel** with 30 days notice (no penalties)

### What payment methods are accepted?

- Credit cards (Visa, MasterCard, Amex)
- ACH/wire transfer (Enterprise only)
- Purchase orders (Enterprise only)

### Is there a refund policy?

- **Free tier**: No refunds (free service)
- **Paid tiers**: 30-day money-back guarantee
- **Annual plans**: Pro-rated refund within 90 days
- **Enterprise**: Custom terms in contract

### Do you offer academic or non-profit discounts?

Yes! We offer:
- **Academic**: 50% discount with .edu email
- **Non-profit**: 40% discount with verified 501(c)(3) status
- **Open source**: Free Professional tier for OSS projects

---

## Security and Compliance

### Is my data secure?

Yes! Security measures include:

- **Encryption at rest**: AES-256 encryption for database
- **Encryption in transit**: TLS 1.3 for all API traffic
- **API key hashing**: SHA-256 with 10,000 iterations
- **JWT tokens**: Secure token-based authentication
- **RBAC**: Role-based access control
- **Audit logs**: Comprehensive audit trail
- **Regular security audits**: Third-party penetration testing

### What compliance standards do you meet?

**Current:**
- SOC 2 Type II (in progress)
- GDPR compliant
- CCPA compliant
- ISO 27001 (in progress)

**Enterprise:**
- HIPAA compliance available
- PCI DSS for payment data
- FedRAMP (roadmap)

### Where is data stored?

**Cloud Regions:**
- US East (Virginia) - Default
- US West (Oregon)
- EU (Frankfurt) - GDPR compliance
- Asia (Singapore)

**Self-Hosted:**
- Deploy on your infrastructure
- Full data control
- Available for Enterprise tier

### Can I export my data?

Yes! You can export:

- **All data**: Complete database export
- **Formats**: CSV, JSON, Excel, Parquet
- **API access**: Retrieve via REST API
- **Scheduled exports**: Automated daily/weekly exports
- **Data portability**: No lock-in, easy migration

### Do you support SSO/SAML?

Yes! Enterprise tier includes:

- **SAML 2.0**: Okta, Azure AD, Google Workspace
- **OAuth 2.0**: Generic OAuth providers
- **LDAP/AD**: Active Directory integration
- **Multi-factor authentication**: TOTP, SMS, hardware keys

### How do you handle PII?

- **Minimal collection**: Only necessary data
- **Anonymization**: Option to anonymize user IDs
- **Data retention**: Configurable retention policies
- **GDPR rights**: Right to access, delete, export
- **Consent management**: Track consent and preferences

---

## Integration Questions

### How do I integrate with my application?

**Methods:**

1. **API Integration:**
```python
import requests

response = requests.post(
    "https://api.llm-cost-ops.example.com/api/v1/usage",
    headers={"Authorization": "Bearer sk_live_..."},
    json={
        "organization_id": "org-123",
        "provider": "openai",
        "model_id": "gpt-4",
        "input_tokens": 1500,
        "output_tokens": 800,
        "total_tokens": 2300
    }
)
```

2. **SDK Integration:**
```python
from llm_cost_ops import CostOpsClient

client = CostOpsClient(api_key="sk_live_...")
client.usage.submit(
    organization_id="org-123",
    provider="openai",
    model_id="gpt-4",
    input_tokens=1500,
    output_tokens=800
)
```

3. **Webhook Integration:**
```python
# Receive real-time cost notifications
@app.route('/webhooks/llm-cost-ops', methods=['POST'])
def webhook():
    event = request.json
    if event['event'] == 'budget.threshold_exceeded':
        send_alert(event['data'])
```

### Can I use LLM Cost Ops with LangChain?

Yes! We provide a LangChain callback:

```python
from langchain.callbacks import CostOpsCallbackHandler
from langchain.llms import OpenAI

llm = OpenAI(
    callbacks=[
        CostOpsCallbackHandler(
            api_key="sk_live_...",
            organization_id="org-123"
        )
    ]
)

response = llm("Hello, world!")
# Cost automatically tracked
```

### Does it work with Azure OpenAI?

Yes! Azure OpenAI is fully supported:

```bash
cost-ops pricing add \
  --provider azure \
  --model gpt-4-turbo \
  --input-price 10.0 \
  --output-price 30.0

# Submit usage
cost-ops ingest --file azure-usage.json
```

### Can I track custom models?

Yes! Add custom pricing for any model:

```bash
cost-ops pricing add \
  --provider custom \
  --model my-local-llm \
  --input-price 0.0 \
  --output-price 0.0
```

### Does it support streaming responses?

Yes! Track streaming responses:

```python
from openai import OpenAI
client = OpenAI()

stream = client.chat.completions.create(
    model="gpt-4",
    messages=[{"role": "user", "content": "Hello"}],
    stream=True
)

total_tokens = 0
for chunk in stream:
    total_tokens += 1  # Track tokens

# Submit to LLM Cost Ops
cost_ops.usage.submit(
    provider="openai",
    model_id="gpt-4",
    total_tokens=total_tokens
)
```

---

## Performance Questions

### What is the expected latency?

**API Response Times:**
- Health check: <10ms
- Submit usage: <50ms
- Query costs: <200ms (typical)
- Export: Variable (depends on size)
- Forecast: <1s

**Factors:**
- Database size
- Query complexity
- Network latency
- Server load

### How many requests can it handle?

**Throughput:**
- Single instance: 1,000 req/s
- With load balancing: 10,000+ req/s
- Database bottleneck at ~50,000 req/s

**Optimization:**
- Use batch submission
- Implement request caching
- Scale horizontally

### Can it handle millions of records?

Yes! Tested with:
- **100M usage records**: Query performance <500ms
- **PostgreSQL optimization**: Partitioning, indexes
- **Archive strategy**: Move old data to cold storage

### How do I optimize performance?

**Database:**
```sql
-- Add indexes for common queries
CREATE INDEX idx_usage_org_time ON usage_records(organization_id, timestamp DESC);
CREATE INDEX idx_usage_provider ON usage_records(provider);

-- Partition by timestamp
CREATE TABLE usage_records_2025_11 PARTITION OF usage_records
FOR VALUES FROM ('2025-11-01') TO ('2025-12-01');
```

**Application:**
```toml
[database]
pool_size = 30  # Increase connection pool

[cache]
enable = true
ttl_secs = 300  # Cache query results
```

**Architecture:**
- Use read replicas for queries
- Implement CDN for static content
- Use Redis for caching

### Does it support horizontal scaling?

Yes! Scale with:

**Load Balancing:**
```yaml
# Kubernetes HPA
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: llm-cost-ops
spec:
  minReplicas: 3
  maxReplicas: 20
  targetCPUUtilizationPercentage: 70
```

**Database:**
- PostgreSQL read replicas
- Connection pooling (PgBouncer)
- Sharding for extreme scale

---

## Feature Questions

### Can I set budget alerts?

Yes! Set budgets and receive alerts:

```bash
# Via API
curl -X POST https://api.llm-cost-ops.example.com/api/v1/budgets \
  -H "Authorization: Bearer sk_live_..." \
  -d '{
    "organization_id": "org-123",
    "amount": 5000.00,
    "period": "monthly",
    "alert_thresholds": [0.50, 0.75, 0.90]
  }'

# Via webhook
POST /webhooks/your-endpoint
{
  "event": "budget.threshold_exceeded",
  "data": {
    "budget_id": "budget-123",
    "threshold": 0.75,
    "current_spend": 3750.00
  }
}
```

### How does forecasting work?

**Forecast Models:**

1. **Linear Regression**: Trend-based forecasting
2. **Moving Average**: Smoothed historical average
3. **Exponential Smoothing**: Weighted recent data
4. **Prophet**: ML-based with seasonality

**Usage:**
```bash
cost-ops forecast generate --horizon 30 --model exponential_smoothing
```

**Accuracy:**
- MAPE (Mean Absolute Percentage Error): 5-10%
- Confidence intervals: 95% default
- Regular retraining: Daily

### Can I detect cost anomalies?

Yes! Anomaly detection identifies unusual spending:

```bash
cost-ops forecast anomalies --sensitivity high
```

**Methods:**
- Z-score detection
- IQR (Interquartile Range)
- Prophet anomaly detection

**Alerts:**
- Email notifications
- Webhook events
- Slack integration

### Does it support multi-tenancy?

Yes! Organization-level isolation:

- **Data isolation**: Each organization's data is separate
- **RBAC**: Per-organization permissions
- **Billing**: Separate billing per organization
- **Quotas**: Per-organization rate limits

```bash
# Query for specific organization
cost-ops query --organization org-123 --range last-30-days
```

### Can I customize reports?

Yes! Custom reports with:

- **Report templates**: HTML/PDF templates
- **Scheduling**: Cron-based scheduling
- **Recipients**: Multiple email recipients
- **Formats**: PDF, Excel, CSV
- **Branding**: Custom logo and colors

```bash
cost-ops report generate \
  --type custom \
  --template executive_summary \
  --period last-month \
  --email finance@example.com
```

---

## Deployment Questions

### What are the system requirements?

**Minimum:**
- CPU: 2 cores
- RAM: 2 GB
- Disk: 10 GB
- OS: Linux, macOS, Windows

**Recommended (Production):**
- CPU: 4+ cores
- RAM: 8 GB
- Disk: 50 GB SSD
- OS: Linux (Ubuntu 22.04, RHEL 8+)

**Database:**
- PostgreSQL 12+
- SQLite 3.35+ (development only)

### Can I run it in Docker?

Yes! Docker image available:

```bash
docker run -d \
  -p 8080:8080 \
  -e COST_OPS_DATABASE_URL=postgresql://... \
  -e COST_OPS_AUTH_JWT_SECRET=... \
  llm-cost-ops/llm-cost-ops:latest
```

**Docker Compose:**
```yaml
version: '3.8'
services:
  cost-ops:
    image: llm-cost-ops/llm-cost-ops:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://db/costops
```

### Does it support Kubernetes?

Yes! Complete Kubernetes manifests provided:

```bash
# Quick deploy
kubectl apply -k k8s/overlays/production/

# Helm chart
helm install llm-cost-ops ./k8s/helm/llm-cost-ops
```

**Features:**
- High availability (3+ replicas)
- Auto-scaling (HPA)
- Rolling updates
- Health checks
- Prometheus monitoring

### Can I run it on AWS/GCP/Azure?

Yes! Cloud-native deployment:

**AWS:**
- ECS/Fargate
- EKS (Kubernetes)
- RDS PostgreSQL
- S3 for exports

**GCP:**
- Cloud Run
- GKE (Kubernetes)
- Cloud SQL
- Cloud Storage

**Azure:**
- Container Instances
- AKS (Kubernetes)
- Azure Database for PostgreSQL
- Blob Storage

### Is there a managed/hosted version?

Yes! We offer fully managed hosting:

- **Shared**: Multi-tenant, Developer/Professional tiers
- **Dedicated**: Single-tenant, Enterprise tier
- **Private cloud**: Your AWS/GCP/Azure account
- **On-premises**: Self-hosted with support

---

## Troubleshooting

### Why are my costs different from provider bills?

**Common Causes:**

1. **Timing differences**: Provider bills monthly, we track real-time
2. **Pricing updates**: Provider changed pricing mid-month
3. **Cached tokens**: Not accounting for cache discounts
4. **Minimum charges**: Some providers have minimum per-request fees
5. **Tax/fees**: Provider bills include tax, we show pre-tax

**Solutions:**
- Update pricing tables regularly
- Enable cached token tracking
- Compare same time period
- Contact support for reconciliation

### Why am I getting rate limited?

**Check current limits:**
```bash
curl -I http://localhost:8080/api/v1/costs
# X-RateLimit-Limit: 1000
# X-RateLimit-Remaining: 0
```

**Solutions:**
- Implement exponential backoff
- Batch requests when possible
- Upgrade to higher tier
- Request limit increase (Enterprise)

### How do I reset my password?

**Self-Service:**
```bash
# Request reset
cost-ops auth reset-password --email user@example.com

# Check email for reset link
```

**Admin Reset:**
Contact support with:
- Email address
- Organization ID
- Verification of identity

### How do I backup my data?

**PostgreSQL:**
```bash
# Backup
pg_dump -U postgres costops > backup.sql

# Restore
psql -U postgres costops < backup.sql
```

**SQLite:**
```bash
# Backup
sqlite3 cost-ops.db ".backup backup.db"

# Restore
cp backup.db cost-ops.db
```

**API Export:**
```bash
cost-ops export --output full-export.csv --format csv --all
```

---

## Support and Community

### How do I get support?

**Community Support (Free):**
- GitHub Issues: Bug reports, feature requests
- Discord: Community chat
- Forum: Long-form discussions
- Documentation: Comprehensive guides

**Paid Support:**
- Email: support@llm-cost-ops.example.com
- Priority support: Professional tier (24hr response)
- Dedicated support: Enterprise tier (4hr response, phone)
- SLA: Enterprise tier (99.9% uptime guarantee)

### Where can I find examples?

**GitHub Repository:**
- https://github.com/llm-cost-ops/llm-cost-ops/tree/main/examples

**Example Projects:**
- LangChain integration
- OpenAI proxy with tracking
- Cost dashboard (React)
- Budget alerting (Python)
- Multi-tenant SaaS example

### How do I contribute?

We welcome contributions!

**Ways to Contribute:**
- Bug reports and feature requests
- Code contributions (PRs)
- Documentation improvements
- Example projects
- Community support

**Process:**
1. Fork repository
2. Create feature branch
3. Make changes with tests
4. Submit pull request
5. Code review and merge

See CONTRIBUTING.md for details.

### Is there a roadmap?

**Q1 2026:**
- Advanced ML-based forecasting
- Custom dashboards
- Cloud billing API integration (AWS, GCP, Azure)

**Q2 2026:**
- ROI correlation engine
- Multi-cloud cost allocation
- Advanced anomaly detection

**Q3 2026:**
- Cost optimization recommendations
- Real-time streaming analytics
- GraphQL API

**Q4 2026:**
- Mobile app (iOS, Android)
- Advanced RBAC with custom policies
- FedRAMP compliance

### How do I request a feature?

**GitHub Issues:**
1. Check existing issues
2. Create new feature request
3. Describe use case and benefits
4. Community votes on features

**Enterprise Customers:**
- Direct feature requests
- Prioritized roadmap items
- Custom development available

### Can I get a demo?

Yes! Request a demo:

- **Self-service demo**: https://demo.llm-cost-ops.example.com
- **Guided demo**: Schedule with sales team
- **POC**: 30-day proof of concept (Enterprise)
- **Sandbox**: Free developer sandbox environment

**Contact:**
- Email: sales@llm-cost-ops.example.com
- Phone: 1-800-LLM-COST
- Form: https://llm-cost-ops.example.com/demo

---

## Additional Resources

### Documentation

- [Getting Started Guide](../tutorials/getting-started.md)
- [API Reference](./api-reference.md)
- [CLI Reference](./cli-reference.md)
- [Configuration Reference](./configuration.md)
- [Troubleshooting Guide](./troubleshooting.md)

### Videos

- Product overview (5 min)
- Installation tutorial (10 min)
- Advanced features (20 min)
- Integration examples (15 min)

### Blog

- Best practices for cost optimization
- Case studies from customers
- Product updates and releases
- Technical deep-dives

### Community

- Discord: https://discord.gg/llm-cost-ops
- Twitter: @llmcostops
- LinkedIn: /company/llm-cost-ops
- YouTube: LLM Cost Ops Channel

---

**Have a question not answered here?**

Contact us:
- Email: support@llm-cost-ops.example.com
- Discord: https://discord.gg/llm-cost-ops
- Forum: https://forum.llm-cost-ops.example.com

**See Also:**

- [API Reference](./api-reference.md)
- [CLI Reference](./cli-reference.md)
- [Configuration Reference](./configuration.md)
- [Troubleshooting Guide](./troubleshooting.md)
