# Interactive Learning Materials

This directory contains comprehensive interactive learning materials for the LLM Cost Ops platform, including Jupyter notebooks, Postman collections, and code playground resources.

## Overview

The interactive materials provide hands-on learning experiences through:
- **Jupyter Notebooks**: Data analysis and machine learning tutorials
- **Postman Collections**: API exploration and testing
- **Code Playground**: Ready-to-use templates and examples

## Directory Structure

```
interactive/
├── notebooks/           # Jupyter notebooks for data analysis
├── postman/            # Postman collections and environments
├── playground/         # Code templates and examples
└── README.md          # This file
```

## Contents Summary

### Jupyter Notebooks (5 notebooks, 126 KB total)

Located in `notebooks/`:

1. **01_cost_analysis_basics.ipynb** (18 KB)
   - Basic cost data loading and exploration
   - Aggregations and summary statistics
   - Time-series visualization
   - Model cost comparison
   - Export results to CSV

2. **02_advanced_analytics.ipynb** (23 KB)
   - Statistical analysis techniques
   - Trend detection and decomposition
   - Anomaly detection (Z-score, IQR, Isolation Forest)
   - Time series forecasting with Prophet
   - Correlation and growth analysis

3. **03_cost_optimization.ipynb** (26 KB)
   - Model cost comparison and efficiency
   - Token usage optimization
   - Cost-performance tradeoff analysis
   - ROI calculations for optimization strategies
   - Actionable recommendations

4. **04_custom_reports.ipynb** (28 KB)
   - Data transformation for reporting
   - Interactive dashboards with Plotly
   - Custom visualizations with Matplotlib
   - Excel export with formatting
   - Automated report generation

5. **05_ml_forecasting.ipynb** (31 KB)
   - Feature engineering for time series
   - ARIMA statistical forecasting
   - Prophet for robust predictions
   - LSTM neural networks
   - Model comparison and deployment

### Postman Collection (25 KB total)

Located in `postman/`:

- **LLM-Cost-Ops.postman_collection.json** (22 KB)
  - 30+ API endpoints organized by category
  - Pre-request scripts for authentication
  - Automated tests for responses
  - Complete request/response examples
  - Categories:
    - Authentication (Login, Register, Refresh)
    - Cost Tracking (Track, Query, Summarize)
    - Analytics (Trends, Forecasts, Comparisons)
    - Budgets (CRUD operations, Status)
    - Alerts (Rules, History)
    - Reports (Generate, Download)
    - Projects (Management)
    - Admin (Stats, Users, Health)

- **Environment Files** (3.6 KB total)
  - Development environment
  - Staging environment
  - Production environment
  - Pre-configured variables

### Code Playground (12 KB)

Located in `playground/`:

- **README.md** - Comprehensive guide including:
  - Quick start options (Local, CodeSandbox, StackBlitz, Replit, Codespaces)
  - 5 example projects
  - Language-specific guides (Python, JavaScript, TypeScript, Rust)
  - Quick start templates
  - Best practices and troubleshooting

## Getting Started

### Option 1: Jupyter Notebooks

**Prerequisites:**
```bash
pip install pandas matplotlib seaborn numpy requests scikit-learn prophet statsmodels tensorflow plotly openpyxl
```

**Usage:**
```bash
cd notebooks/
jupyter notebook
# Open any notebook and follow along
```

**Learning Path:**
1. Start with `01_cost_analysis_basics.ipynb`
2. Progress to `02_advanced_analytics.ipynb`
3. Apply insights in `03_cost_optimization.ipynb`
4. Build reports with `04_custom_reports.ipynb`
5. Master forecasting in `05_ml_forecasting.ipynb`

### Option 2: Postman Collection

**Prerequisites:**
- Postman desktop app or web version
- LLM Cost Ops API access

**Setup:**
1. Import collection: `postman/LLM-Cost-Ops.postman_collection.json`
2. Import environment: `postman/LLM-Cost-Ops.postman_environment.json`
3. Update environment variables (API key, credentials)
4. Start making requests!

**Quick Test:**
1. Run "Login" to authenticate
2. Try "Get Cost Records" to fetch data
3. Explore other endpoints

### Option 3: Code Playground

**Prerequisites:**
- Choose your preferred language (Python, JavaScript, TypeScript, Rust)
- Or use online playgrounds (no setup required)

**Quick Start:**
```bash
cd playground/
# Follow README.md instructions
```

**Online Options:**
- CodeSandbox (JavaScript/TypeScript)
- StackBlitz (Full-stack)
- Replit (Multi-language)
- GitHub Codespaces (Complete environment)

## Features by Material Type

### Jupyter Notebooks Features

- **Interactive Code Execution**: Run code cells and see immediate results
- **Data Visualization**: Rich charts and graphs with Matplotlib, Seaborn, Plotly
- **Machine Learning**: Hands-on ML model training and evaluation
- **Export Capabilities**: Save results to CSV, Excel, PDF
- **Documentation**: Markdown cells with explanations and best practices

**Key Learning Outcomes:**
- Master cost data analysis techniques
- Build predictive models for cost forecasting
- Create custom reports and dashboards
- Optimize LLM usage costs
- Apply statistical and ML methods

### Postman Collection Features

- **Complete API Coverage**: All endpoints documented and tested
- **Authentication Flow**: Automatic token management
- **Environment Variables**: Easy switching between dev/staging/prod
- **Request Templates**: Ready-to-use examples for all operations
- **Response Validation**: Automated tests ensure API reliability
- **Documentation**: Inline descriptions and usage notes

**Key Learning Outcomes:**
- Understand the complete API surface
- Test API integrations before coding
- Debug API issues quickly
- Learn request/response patterns
- Validate API behavior

### Code Playground Features

- **Multiple Languages**: Python, JavaScript, TypeScript, Rust examples
- **Quick Start Templates**: Minimal boilerplate to get started
- **Example Projects**: 5 complete reference implementations
- **Online Playgrounds**: No local setup required options
- **Best Practices**: Security, performance, and code organization guidance

**Key Learning Outcomes:**
- Rapid prototyping and experimentation
- Learn SDK usage patterns
- Build proof-of-concept applications
- Understand integration patterns
- Deploy production-ready code

## Use Cases

### For Data Analysts
- Use Jupyter notebooks for cost analysis
- Build custom reports and dashboards
- Identify cost optimization opportunities
- Present insights to stakeholders

### For Developers
- Use Postman to explore the API
- Use playground templates for quick integration
- Build applications with SDK examples
- Test and debug implementations

### For DevOps Engineers
- Monitor costs programmatically
- Set up automated alerting
- Integrate with CI/CD pipelines
- Track costs across environments

### For Data Scientists
- Build cost forecasting models
- Apply ML to cost optimization
- Analyze usage patterns
- Predict future costs

### For Managers
- Review cost analytics reports
- Understand cost drivers
- Make data-driven decisions
- Plan budgets based on forecasts

## Support and Resources

### Documentation
- [Main Documentation](/docs/)
- [API Reference](/docs/api/)
- [SDK Documentation](/docs/sdk/)

### Tutorials
- [Getting Started Guide](/docs/tutorials/getting-started.md)
- [Integration Examples](/docs/examples/)
- [Best Practices](/docs/best-practices/)

### Community
- [GitHub Discussions](https://github.com/yourusername/llm-cost-ops/discussions)
- [Discord Server](https://discord.gg/llm-cost-ops)
- [Blog and Updates](https://blog.llm-cost-ops.example.com)

### Getting Help
- **Issues**: Report bugs on [GitHub Issues](https://github.com/yourusername/llm-cost-ops/issues)
- **Questions**: Ask on [Stack Overflow](https://stackoverflow.com/questions/tagged/llm-cost-ops)
- **Email**: support@llm-cost-ops.example.com

## Contributing

We welcome contributions to improve the interactive materials!

**How to Contribute:**
1. Fork the repository
2. Add or improve materials
3. Test your changes
4. Submit a pull request

**What to Contribute:**
- New notebook examples
- Additional Postman requests
- Code templates in other languages
- Bug fixes and improvements
- Documentation enhancements

See [CONTRIBUTING.md](/CONTRIBUTING.md) for detailed guidelines.

## License

All interactive materials are licensed under the MIT License. See [LICENSE](/LICENSE) for details.

## Changelog

### Version 1.0.0 (2024-01-15)
- Initial release
- 5 comprehensive Jupyter notebooks
- Complete Postman collection with 30+ endpoints
- Code playground with multi-language support
- 3 environment configurations
- Extensive documentation

## Roadmap

### Planned Additions
- [ ] Video tutorials for each notebook
- [ ] GraphQL Postman collection
- [ ] Docker playground environment
- [ ] More language examples (Go, Java, C#)
- [ ] Advanced ML notebooks (AutoML, ensemble methods)
- [ ] Real-time streaming examples
- [ ] Mobile SDK examples

## Acknowledgments

Special thanks to:
- The Jupyter community for the excellent notebook platform
- Postman for their API development tools
- All contributors to the open-source libraries used

---

**Last Updated**: 2024-01-15  
**Maintainer**: LLM Cost Ops Team  
**Version**: 1.0.0
