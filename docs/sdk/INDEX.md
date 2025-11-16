# LLM-CostOps SDK Documentation Index

Quick navigation to all SDK documentation.

## 📚 Getting Started

Start here if you're new to LLM-CostOps:

1. **[Main SDK Documentation](README.md)** - Overview and feature highlights
2. **[Quickstart Guide](getting-started/quickstart.md)** - Get started in 5 minutes
3. **[Authentication Guide](getting-started/authentication.md)** - API keys, JWT, and security

## 🔌 API Reference

Complete API documentation:

- **[REST API Reference](api-reference/rest-api/README.md)** - Full API reference with all endpoints
- **[Python SDK](api-reference/python/README.md)** - Coming soon
- **[TypeScript SDK](api-reference/typescript/README.md)** - Coming soon
- **[Go SDK](api-reference/go/README.md)** - Coming soon
- **[Java SDK](api-reference/java/README.md)** - Coming soon

## 📖 Guides

In-depth guides for specific features:

- **[Cost Analysis Guide](guides/cost-analysis.md)** - Analyze and optimize LLM costs
- **Forecasting Guide** - Coming soon
- **Budget Management Guide** - Coming soon
- **Export & Reporting Guide** - Coming soon
- **Anomaly Detection Guide** - Coming soon

## 💻 Code Examples

Working code examples in multiple languages:

- **[cURL Examples](examples/curl/README.md)** - 50+ cURL examples and scripts
- **[Python Examples](examples/python/README.md)** - Coming soon
- **[TypeScript Examples](examples/typescript/README.md)** - Coming soon
- **[Go Examples](examples/go/README.md)** - Coming soon
- **[Java Examples](examples/java/README.md)** - Coming soon

## 🔧 Framework Integrations

Integration guides for popular frameworks:

- **[FastAPI Integration](frameworks/fastapi.md)** - Complete FastAPI integration guide
- **Django Integration** - Coming soon
- **Flask Integration** - Coming soon
- **React Integration** - Coming soon
- **Next.js Integration** - Coming soon
- **Spring Boot Integration** - Coming soon

## 🆘 Help & Support

Troubleshooting and support resources:

- **[Troubleshooting Guide](troubleshooting.md)** - Solutions to common issues
- **[FAQ](faq.md)** - 38 frequently asked questions
- **[Documentation Website Setup](DOCUMENTATION_SITE.md)** - Set up docs site with Docusaurus

## 📊 Additional Resources

- **[SDK Documentation Report](../SDK_DOCUMENTATION_REPORT.md)** - Complete documentation report
- **[Main README](../../README.md)** - Project overview
- **[API Specification](../SPECIFICATION.md)** - Technical specification
- **[Deployment Guide](../../k8s/DEPLOYMENT.md)** - Kubernetes deployment

## 🚀 Quick Links

### Most Common Tasks

1. **Track your first LLM usage**
   - [Quickstart Guide](getting-started/quickstart.md#step-3-submit-your-first-usage-record)

2. **Query your costs**
   - [Cost Query Examples](api-reference/rest-api/README.md#get-costs)

3. **Analyze costs by provider**
   - [Cost Analysis Guide](guides/cost-analysis.md#group-by-provider)

4. **Integrate with FastAPI**
   - [FastAPI Integration](frameworks/fastapi.md#basic-integration)

5. **Set up authentication**
   - [Authentication Guide](getting-started/authentication.md#api-key-authentication)

### Quick Reference

```bash
# Submit usage
curl -X POST https://api.llm-cost-ops.dev/api/v1/usage \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "organization_id": "org-123",
    "provider": "openai",
    "model_id": "gpt-4",
    "input_tokens": 1000,
    "output_tokens": 500,
    "total_tokens": 1500
  }'

# Get costs
curl -X GET "https://api.llm-cost-ops.dev/api/v1/costs?organization_id=org-123" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

## 📞 Support

- **GitHub Issues**: https://github.com/llm-devops/llm-cost-ops/issues
- **Discord**: https://discord.gg/llm-cost-ops
- **Email**: support@llm-cost-ops.dev

## 🤝 Contributing

Want to improve the documentation?

1. Fork the repository
2. Make your changes
3. Submit a pull request
4. See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines

---

**Last Updated:** 2025-11-15
**Version:** 1.0.0
**Status:** Production Ready ✅
