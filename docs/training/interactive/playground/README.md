# LLM Cost Ops - Code Playground

Welcome to the LLM Cost Ops interactive code playground! This directory contains templates and examples for quickly experimenting with the LLM Cost Ops API.

## Overview

The code playground provides pre-configured environments and starter templates to help you:
- Experiment with API integrations
- Test cost tracking implementations
- Build proof-of-concept applications
- Learn the platform through hands-on coding

## Quick Start Options

### Option 1: Local Development

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/llm-cost-ops.git
   cd llm-cost-ops/docs/training/interactive/playground
   ```

2. **Choose your language template**
   - `python/` - Python examples and templates
   - `javascript/` - Node.js/JavaScript examples
   - `typescript/` - TypeScript examples
   - `rust/` - Rust examples

3. **Install dependencies**
   ```bash
   # Python
   cd python
   pip install -r requirements.txt

   # JavaScript/TypeScript
   cd javascript
   npm install
   ```

4. **Configure environment**
   ```bash
   cp .env.example .env
   # Edit .env with your API credentials
   ```

5. **Run examples**
   ```bash
   # Python
   python basic_tracking.py

   # JavaScript
   node basic-tracking.js
   ```

### Option 2: Online Playgrounds

#### CodeSandbox

[![Open in CodeSandbox](https://codesandbox.io/static/img/play-codesandbox.svg)](https://codesandbox.io/s/github/yourusername/llm-cost-ops/tree/main/docs/training/interactive/playground/javascript)

**JavaScript/TypeScript Sandbox:**
- Pre-configured environment with all dependencies
- Live code editing and preview
- No local setup required
- Perfect for quick experiments

**Getting Started:**
1. Click the CodeSandbox button above
2. Fork the sandbox to your account
3. Update the `.env` file with your API key
4. Start coding!

#### StackBlitz

[![Open in StackBlitz](https://developer.stackblitz.com/img/open_in_stackblitz.svg)](https://stackblitz.com/github/yourusername/llm-cost-ops/tree/main/docs/training/interactive/playground)

**Full-Stack Development Environment:**
- VS Code-like interface in the browser
- Full Node.js environment
- Git integration
- Terminal access

**Getting Started:**
1. Click the StackBlitz button above
2. Wait for the environment to load
3. Configure your API credentials
4. Run examples from the integrated terminal

#### Replit

[![Run on Replit](https://replit.com/badge/github/yourusername/llm-cost-ops)](https://replit.com/github/yourusername/llm-cost-ops)

**Multi-Language Support:**
- Python, JavaScript, TypeScript, and more
- Collaborative coding features
- Built-in hosting
- Database integration

**Getting Started:**
1. Click the Replit button above
2. Fork the repl to your account
3. Set up environment secrets
4. Click "Run" to start

### Option 3: GitHub Codespaces

[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://github.com/codespaces/new?repo=yourusername/llm-cost-ops)

**Full Development Environment:**
- Complete VS Code environment
- Pre-configured with all dependencies
- 60 hours free per month
- Full Git integration

**Getting Started:**
1. Click the Codespaces button above
2. Wait for the environment to provision
3. Open the playground directory
4. Start coding!

## Example Projects

### 1. Basic Cost Tracking

**Location:** `examples/01-basic-tracking/`

A simple example showing how to track LLM API costs.

```python
# Python example
from llm_cost_ops import CostTracker

tracker = CostTracker(api_key="your-api-key")

# Track a single request
tracker.track_cost(
    model="gpt-4",
    input_tokens=1500,
    output_tokens=500,
    total_cost=0.045
)
```

```javascript
// JavaScript example
const { CostTracker } = require('llm-cost-ops');

const tracker = new CostTracker({ apiKey: 'your-api-key' });

// Track a single request
await tracker.trackCost({
  model: 'gpt-4',
  inputTokens: 1500,
  outputTokens: 500,
  totalCost: 0.045
});
```

### 2. Real-time Analytics Dashboard

**Location:** `examples/02-analytics-dashboard/`

Build a real-time cost analytics dashboard.

**Features:**
- Live cost monitoring
- Interactive charts
- Model comparison
- Budget tracking

**Technologies:**
- Frontend: React + Chart.js
- Backend: Express.js
- Real-time: WebSockets

### 3. Budget Alert System

**Location:** `examples/03-budget-alerts/`

Implement automated budget monitoring and alerts.

**Features:**
- Budget threshold monitoring
- Email/Slack notifications
- Custom alert rules
- Historical tracking

### 4. Cost Optimization Tool

**Location:** `examples/04-cost-optimizer/`

Analyze and optimize LLM API costs.

**Features:**
- Model cost comparison
- Token efficiency analysis
- Optimization recommendations
- ROI calculator

### 5. Multi-Project Tracker

**Location:** `examples/05-multi-project/`

Track costs across multiple projects and teams.

**Features:**
- Project-based cost allocation
- Team usage analytics
- Cross-project comparisons
- Custom tagging

## Templates

### Quick Start Templates

#### Minimal Template
```python
# minimal_tracker.py
import os
from llm_cost_ops import CostTracker

def main():
    tracker = CostTracker(api_key=os.getenv('LLM_COST_OPS_API_KEY'))

    # Your code here
    tracker.track_cost(
        model="gpt-3.5-turbo",
        input_tokens=100,
        output_tokens=50,
        total_cost=0.0025
    )

if __name__ == "__main__":
    main()
```

#### Express.js API Template
```javascript
// app.js
const express = require('express');
const { CostTracker } = require('llm-cost-ops');

const app = express();
const tracker = new CostTracker({ apiKey: process.env.API_KEY });

app.use(express.json());

app.post('/api/chat', async (req, res) => {
    // Your LLM API call here
    const response = await callLLM(req.body);

    // Track the cost
    await tracker.trackCost({
        model: response.model,
        inputTokens: response.usage.prompt_tokens,
        outputTokens: response.usage.completion_tokens,
        totalCost: calculateCost(response.usage)
    });

    res.json(response);
});

app.listen(3000, () => console.log('Server running on port 3000'));
```

#### React Dashboard Template
```jsx
// Dashboard.jsx
import React, { useState, useEffect } from 'react';
import { CostChart, BudgetWidget } from 'llm-cost-ops-react';

function Dashboard() {
    const [costs, setCosts] = useState([]);

    useEffect(() => {
        fetch('/api/costs')
            .then(res => res.json())
            .then(data => setCosts(data));
    }, []);

    return (
        <div>
            <h1>Cost Dashboard</h1>
            <CostChart data={costs} />
            <BudgetWidget />
        </div>
    );
}

export default Dashboard;
```

## Language-Specific Guides

### Python

**Installation:**
```bash
pip install llm-cost-ops
```

**Basic Usage:**
```python
from llm_cost_ops import CostTracker, Analytics

# Initialize
tracker = CostTracker(api_key="your-api-key")

# Track costs
tracker.track_cost(model="gpt-4", input_tokens=1000, output_tokens=500)

# Get analytics
analytics = Analytics(api_key="your-api-key")
summary = analytics.get_summary(days=30)
```

**See:** `python/README.md` for detailed examples

### JavaScript/Node.js

**Installation:**
```bash
npm install llm-cost-ops
```

**Basic Usage:**
```javascript
const { CostTracker, Analytics } = require('llm-cost-ops');

// Initialize
const tracker = new CostTracker({ apiKey: 'your-api-key' });

// Track costs
await tracker.trackCost({
    model: 'gpt-4',
    inputTokens: 1000,
    outputTokens: 500
});

// Get analytics
const analytics = new Analytics({ apiKey: 'your-api-key' });
const summary = await analytics.getSummary({ days: 30 });
```

**See:** `javascript/README.md` for detailed examples

### TypeScript

**Installation:**
```bash
npm install llm-cost-ops @types/llm-cost-ops
```

**Basic Usage:**
```typescript
import { CostTracker, Analytics, CostEntry } from 'llm-cost-ops';

// Initialize with type safety
const tracker = new CostTracker({ apiKey: 'your-api-key' });

// Track costs
const entry: CostEntry = {
    model: 'gpt-4',
    inputTokens: 1000,
    outputTokens: 500,
    totalCost: 0.045
};

await tracker.trackCost(entry);
```

**See:** `typescript/README.md` for detailed examples

### Rust

**Installation:**
```toml
[dependencies]
llm-cost-ops = "0.1.0"
```

**Basic Usage:**
```rust
use llm_cost_ops::{CostTracker, CostEntry};

#[tokio::main]
async fn main() {
    let tracker = CostTracker::new("your-api-key");

    let entry = CostEntry {
        model: "gpt-4".to_string(),
        input_tokens: 1000,
        output_tokens: 500,
        total_cost: 0.045,
    };

    tracker.track_cost(entry).await.unwrap();
}
```

**See:** `rust/README.md` for detailed examples

## Development Workflow

### 1. Explore Examples
Start with the basic examples to understand the fundamentals.

### 2. Modify Templates
Take a template and customize it for your use case.

### 3. Build Your Application
Use the SDK to integrate cost tracking into your application.

### 4. Test and Iterate
Use the playground to test different approaches.

### 5. Deploy
Move your tested code to production.

## Best Practices

### Security
- Never commit API keys to version control
- Use environment variables for sensitive data
- Rotate API keys regularly
- Use different keys for development/production

### Performance
- Batch cost tracking when possible
- Cache analytics data appropriately
- Use webhooks for real-time updates
- Implement retry logic with exponential backoff

### Code Organization
- Separate configuration from code
- Use dependency injection
- Write testable code
- Document your implementations

### Error Handling
- Always handle API errors gracefully
- Implement proper logging
- Use try-catch blocks
- Provide meaningful error messages

## Troubleshooting

### Common Issues

**Issue:** API authentication fails
```
Error: 401 Unauthorized
```
**Solution:** Check your API key and ensure it's correctly set in environment variables.

**Issue:** Rate limiting
```
Error: 429 Too Many Requests
```
**Solution:** Implement exponential backoff and respect rate limits.

**Issue:** Network timeout
```
Error: ETIMEDOUT
```
**Solution:** Increase timeout settings and check network connectivity.

### Getting Help

- Check the [API Documentation](/docs/api/)
- Review [Frequently Asked Questions](/docs/faq.md)
- Join our [Discord Community](https://discord.gg/llm-cost-ops)
- Open an issue on [GitHub](https://github.com/yourusername/llm-cost-ops/issues)

## Contributing

Want to add your own example or template?

1. Fork the repository
2. Create your example in the appropriate directory
3. Add documentation
4. Submit a pull request

See [CONTRIBUTING.md](/CONTRIBUTING.md) for guidelines.

## Resources

### Documentation
- [API Reference](/docs/api/)
- [SDK Documentation](/docs/sdk/)
- [Integration Guides](/docs/guides/)

### Tutorials
- [Getting Started Tutorial](/docs/tutorials/getting-started.md)
- [Building a Dashboard](/docs/tutorials/dashboard.md)
- [Cost Optimization Guide](/docs/tutorials/optimization.md)

### Community
- [GitHub Discussions](https://github.com/yourusername/llm-cost-ops/discussions)
- [Discord Server](https://discord.gg/llm-cost-ops)
- [Blog](https://blog.llm-cost-ops.example.com)

## License

The code playground examples and templates are licensed under the MIT License. See [LICENSE](/LICENSE) for details.

## Support

Need help? Reach out:
- Email: support@llm-cost-ops.example.com
- Discord: [Join our server](https://discord.gg/llm-cost-ops)
- Twitter: [@llmcostops](https://twitter.com/llmcostops)
