# Lab 1: Basic Cost Tracking

## Overview

In this hands-on lab, you will learn the fundamentals of cost tracking with the LLM Cost Ops platform. You'll set up your environment, install the SDK, create usage records, track costs from multiple LLM providers, and generate cost summaries.

**Estimated Time:** 60-90 minutes

**Difficulty Level:** Beginner

## Learning Objectives

By the end of this lab, you will be able to:

- Set up the LLM Cost Ops platform in your local environment
- Install and configure the Python or TypeScript SDK
- Create and submit usage records for OpenAI, Anthropic, and Google models
- Track token usage and calculate costs accurately
- Query and filter cost data by provider, model, organization, and time range
- Generate cost summaries and reports
- Group costs by different dimensions (provider, model, project)
- Export cost data in various formats

## Prerequisites

Before starting this lab, ensure you have:

- [ ] Basic knowledge of command-line interfaces
- [ ] Python 3.8+ OR Node.js 16+ installed
- [ ] Git installed on your system
- [ ] A code editor (VS Code, Sublime, etc.)
- [ ] 2GB of available disk space
- [ ] Internet connection for downloading dependencies

### System Requirements

**Hardware:**
- CPU: 2+ cores
- RAM: 4GB minimum, 8GB recommended
- Storage: 2GB free space

**Software:**
- OS: Linux, macOS, or Windows 10+
- Rust 1.75+ (will be installed in setup)
- SQLite 3.35+ (bundled with platform)

## Part 1: Environment Setup

### Step 1.1: Clone the Repository

First, clone the LLM Cost Ops repository:

```bash
# Clone the repository
git clone https://github.com/yourusername/llm-cost-ops.git
cd llm-cost-ops

# Verify you're in the correct directory
pwd
# Expected output: /path/to/llm-cost-ops
```

### Step 1.2: Install Rust

Install Rust if you haven't already:

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts and select default installation

# Reload your shell configuration
source $HOME/.cargo/env

# Verify Rust installation
rustc --version
# Expected output: rustc 1.75.0 (or higher)
```

### Step 1.3: Build the Platform

Build the LLM Cost Ops platform from source:

```bash
# Build in release mode for better performance
cargo build --release

# This may take 5-10 minutes on first build
# Expected output:
#    Compiling llm-cost-ops v0.1.0
#    Finished release [optimized] target(s) in 8m 45s
```

**Troubleshooting:**
- If you get compilation errors, ensure Rust version is 1.75+
- On macOS, you may need to install Xcode command-line tools: `xcode-select --install`
- On Linux, install build essentials: `sudo apt-get install build-essential`

### Step 1.4: Initialize the Database

Initialize the SQLite database with schema and sample data:

```bash
# Run the initialization script
./scripts/init-db.sh

# Or manually if script fails:
cargo install sqlx-cli --no-default-features --features sqlite
sqlx database create --database-url sqlite:cost-ops.db
sqlx migrate run --database-url sqlite:cost-ops.db
```

**Expected Output:**
```
========================================
LLM-CostOps Database Initialization
========================================

Database URL: sqlite:cost-ops.db

✓ sqlx-cli is already installed
✓ Database created successfully
✓ Migrations completed successfully
✓ Query cache generated successfully

Tables created: 5
Default pricing entries: 7

Database initialization complete!
```

### Step 1.5: Verify Installation

Test that everything is working:

```bash
# Check CLI is working
./target/release/cost-ops --help

# Check database is initialized
./target/release/cost-ops pricing list
```

**Expected Output:**
```
ID                                   Provider        Model                Effective Date
---------------------------------------------------------------------------------------------
pricing_openai_gpt4                  openai          gpt-4                2024-01-01
pricing_openai_gpt35                 openai          gpt-3.5-turbo        2024-01-01
pricing_anthropic_sonnet             anthropic       claude-3-sonnet      2024-03-01
pricing_anthropic_opus               anthropic       claude-3-opus        2024-03-01
pricing_google_gemini                google          gemini-pro           2024-02-01

Total pricing tables: 5
```

## Part 2: SDK Installation and Configuration

You can choose either Python or TypeScript for this lab. Pick the one you're most comfortable with.

### Option A: Python SDK Setup

#### Step 2A.1: Create Virtual Environment

```bash
# Create a dedicated directory for your lab work
mkdir lab-01-workspace
cd lab-01-workspace

# Create Python virtual environment
python3 -m venv venv

# Activate virtual environment
# On Linux/macOS:
source venv/bin/activate

# On Windows:
# venv\Scripts\activate

# Verify activation (prompt should show (venv))
```

#### Step 2A.2: Install Python Dependencies

```bash
# Install the LLM Cost Ops Python client
pip install requests python-dateutil

# Create requirements.txt
cat > requirements.txt << EOF
requests>=2.31.0
python-dateutil>=2.8.2
pydantic>=2.5.0
python-dotenv>=1.0.0
EOF

# Install all dependencies
pip install -r requirements.txt

# Verify installation
python -c "import requests; print(f'Requests version: {requests.__version__}')"
```

#### Step 2A.3: Create Python Client Library

Create a simple client library for interacting with the Cost Ops API:

```python
# File: cost_ops_client.py

import os
import requests
from datetime import datetime, timezone
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict
import json

@dataclass
class UsageRecord:
    """Represents a single LLM usage record"""
    id: str
    timestamp: str
    provider: str
    model_name: str
    organization_id: str
    prompt_tokens: int
    completion_tokens: int
    total_tokens: int
    cached_tokens: Optional[int] = None
    reasoning_tokens: Optional[int] = None
    latency_ms: Optional[int] = None
    project_id: Optional[str] = None
    user_id: Optional[str] = None
    tags: Optional[List[str]] = None
    metadata: Optional[Dict[str, Any]] = None

class CostOpsClient:
    """Client for LLM Cost Ops API"""

    def __init__(self, base_url: str = "http://localhost:8080", api_key: Optional[str] = None):
        self.base_url = base_url.rstrip('/')
        self.api_key = api_key or os.getenv('COST_OPS_API_KEY')
        self.session = requests.Session()

        if self.api_key:
            self.session.headers.update({
                'Authorization': f'Bearer {self.api_key}',
                'Content-Type': 'application/json'
            })

    def submit_usage(self, usage: UsageRecord) -> Dict:
        """Submit a usage record"""
        url = f"{self.base_url}/api/v1/usage"
        data = asdict(usage)

        # Remove None values
        data = {k: v for k, v in data.items() if v is not None}

        response = self.session.post(url, json=data)
        response.raise_for_status()
        return response.json()

    def submit_batch_usage(self, usage_records: List[UsageRecord]) -> Dict:
        """Submit multiple usage records"""
        url = f"{self.base_url}/api/v1/usage/batch"
        data = [asdict(u) for u in usage_records]

        # Remove None values from each record
        data = [
            {k: v for k, v in record.items() if v is not None}
            for record in data
        ]

        response = self.session.post(url, json=data)
        response.raise_for_status()
        return response.json()

    def get_costs(self,
                  start_date: Optional[str] = None,
                  end_date: Optional[str] = None,
                  provider: Optional[str] = None,
                  model: Optional[str] = None,
                  organization_id: Optional[str] = None,
                  project_id: Optional[str] = None) -> List[Dict]:
        """Query cost records with filters"""
        url = f"{self.base_url}/api/v1/costs"

        params = {}
        if start_date:
            params['start_date'] = start_date
        if end_date:
            params['end_date'] = end_date
        if provider:
            params['provider'] = provider
        if model:
            params['model'] = model
        if organization_id:
            params['organization_id'] = organization_id
        if project_id:
            params['project_id'] = project_id

        response = self.session.get(url, params=params)
        response.raise_for_status()
        return response.json()

    def get_summary(self,
                    start_date: str,
                    end_date: str,
                    organization_id: Optional[str] = None,
                    group_by: Optional[List[str]] = None) -> Dict:
        """Get cost summary with aggregations"""
        url = f"{self.base_url}/api/v1/analytics/summary"

        params = {
            'start_date': start_date,
            'end_date': end_date
        }

        if organization_id:
            params['organization_id'] = organization_id
        if group_by:
            params['group_by'] = ','.join(group_by)

        response = self.session.get(url, params=params)
        response.raise_for_status()
        return response.json()

    def health_check(self) -> Dict:
        """Check API health"""
        url = f"{self.base_url}/health"
        response = self.session.get(url)
        response.raise_for_status()
        return response.json()

# Helper functions
def generate_usage_id() -> str:
    """Generate a unique usage ID"""
    import uuid
    return str(uuid.uuid4())

def current_timestamp() -> str:
    """Get current timestamp in ISO format"""
    return datetime.now(timezone.utc).isoformat()
```

Save this file as `cost_ops_client.py` in your `lab-01-workspace` directory.

### Option B: TypeScript SDK Setup

#### Step 2B.1: Initialize Node.js Project

```bash
# Create a dedicated directory for your lab work
mkdir lab-01-workspace
cd lab-01-workspace

# Initialize npm project
npm init -y

# Install TypeScript and dependencies
npm install typescript @types/node ts-node axios dotenv
npm install --save-dev @types/axios

# Create tsconfig.json
npx tsc --init --target ES2020 --module commonjs --esModuleInterop
```

#### Step 2B.2: Create TypeScript Client Library

Create a TypeScript client library:

```typescript
// File: cost-ops-client.ts

import axios, { AxiosInstance } from 'axios';

export interface UsageRecord {
  id: string;
  timestamp: string;
  provider: string;
  model_name: string;
  organization_id: string;
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
  cached_tokens?: number;
  reasoning_tokens?: number;
  latency_ms?: number;
  project_id?: string;
  user_id?: string;
  tags?: string[];
  metadata?: Record<string, any>;
}

export interface CostRecord {
  id: string;
  usage_id: string;
  provider: string;
  model: string;
  input_cost: string;
  output_cost: string;
  total_cost: string;
  currency: string;
  timestamp: string;
}

export interface CostSummary {
  total_cost: string;
  total_requests: number;
  avg_cost_per_request: string;
  by_provider: Record<string, string>;
  by_model: Record<string, string>;
  by_project?: Record<string, string>;
}

export class CostOpsClient {
  private client: AxiosInstance;

  constructor(
    private baseUrl: string = 'http://localhost:8080',
    private apiKey?: string
  ) {
    this.apiKey = apiKey || process.env.COST_OPS_API_KEY;

    this.client = axios.create({
      baseURL: this.baseUrl,
      headers: {
        'Content-Type': 'application/json',
        ...(this.apiKey && { Authorization: `Bearer ${this.apiKey}` })
      }
    });
  }

  async submitUsage(usage: UsageRecord): Promise<any> {
    const response = await this.client.post('/api/v1/usage', usage);
    return response.data;
  }

  async submitBatchUsage(usageRecords: UsageRecord[]): Promise<any> {
    const response = await this.client.post('/api/v1/usage/batch', usageRecords);
    return response.data;
  }

  async getCosts(filters?: {
    start_date?: string;
    end_date?: string;
    provider?: string;
    model?: string;
    organization_id?: string;
    project_id?: string;
  }): Promise<CostRecord[]> {
    const response = await this.client.get('/api/v1/costs', { params: filters });
    return response.data;
  }

  async getSummary(
    start_date: string,
    end_date: string,
    organization_id?: string,
    group_by?: string[]
  ): Promise<CostSummary> {
    const params: any = { start_date, end_date };
    if (organization_id) params.organization_id = organization_id;
    if (group_by) params.group_by = group_by.join(',');

    const response = await this.client.get('/api/v1/analytics/summary', { params });
    return response.data;
  }

  async healthCheck(): Promise<any> {
    const response = await this.client.get('/health');
    return response.data;
  }
}

// Helper functions
export function generateUsageId(): string {
  return crypto.randomUUID();
}

export function currentTimestamp(): string {
  return new Date().toISOString();
}
```

Save this file as `cost-ops-client.ts` in your `lab-01-workspace` directory.

## Part 3: Starting the API Server

Before we can submit usage records, we need to start the Cost Ops API server.

### Step 3.1: Start the Server

Open a new terminal window and start the server:

```bash
# Navigate to the llm-cost-ops directory
cd /path/to/llm-cost-ops

# Start the API server
./target/release/cost-ops server \
  --host 0.0.0.0 \
  --port 8080 \
  --database-url sqlite:cost-ops.db

# Expected output:
# 2025-01-15T10:00:00Z INFO llm_cost_ops::api: Starting LLM Cost Ops API server
# 2025-01-15T10:00:00Z INFO llm_cost_ops::api: Listening on http://0.0.0.0:8080
# 2025-01-15T10:00:00Z INFO llm_cost_ops::storage: Database connection established
```

**Keep this terminal window open** - the server needs to keep running for the rest of the lab.

### Step 3.2: Verify Server is Running

In your original terminal (in the lab-01-workspace directory):

```bash
# Test health endpoint
curl http://localhost:8080/health

# Expected output:
# {"status":"healthy","version":"0.1.0","database":"connected"}
```

## Part 4: Creating Your First Usage Records

Now let's create and submit usage records for different LLM providers.

### For Python Users:

#### Step 4.1: Create OpenAI Usage Record

Create a new file `track_openai.py`:

```python
#!/usr/bin/env python3
"""Track OpenAI GPT-4 usage"""

from cost_ops_client import CostOpsClient, UsageRecord, generate_usage_id, current_timestamp

# Initialize client
client = CostOpsClient(base_url="http://localhost:8080")

# Create OpenAI GPT-4 usage record
usage = UsageRecord(
    id=generate_usage_id(),
    timestamp=current_timestamp(),
    provider="openai",
    model_name="gpt-4",
    organization_id="org-acme-corp",
    project_id="proj-chatbot",
    user_id="user-alice",
    prompt_tokens=1500,
    completion_tokens=800,
    total_tokens=2300,
    latency_ms=3200,
    tags=["production", "chatbot"],
    metadata={
        "endpoint": "/v1/chat/completions",
        "temperature": 0.7,
        "max_tokens": 1000
    }
)

# Submit usage record
try:
    result = client.submit_usage(usage)
    print(f"✓ Successfully submitted OpenAI usage record")
    print(f"  Record ID: {usage.id}")
    print(f"  Tokens: {usage.prompt_tokens} prompt + {usage.completion_tokens} completion = {usage.total_tokens} total")
    print(f"  Result: {result}")
except Exception as e:
    print(f"✗ Error submitting usage: {e}")
```

Run the script:

```bash
python track_openai.py
```

**Expected Output:**
```
✓ Successfully submitted OpenAI usage record
  Record ID: 550e8400-e29b-41d4-a716-446655440001
  Tokens: 1500 prompt + 800 completion = 2300 total
  Result: {'status': 'success', 'cost': '0.039000'}
```

#### Step 4.2: Create Anthropic Usage Record

Create a new file `track_anthropic.py`:

```python
#!/usr/bin/env python3
"""Track Anthropic Claude usage"""

from cost_ops_client import CostOpsClient, UsageRecord, generate_usage_id, current_timestamp

client = CostOpsClient(base_url="http://localhost:8080")

# Create Anthropic Claude-3 Sonnet usage record
usage = UsageRecord(
    id=generate_usage_id(),
    timestamp=current_timestamp(),
    provider="anthropic",
    model_name="claude-3-sonnet-20240229",
    organization_id="org-acme-corp",
    project_id="proj-analysis",
    user_id="user-bob",
    prompt_tokens=2000,
    completion_tokens=1200,
    total_tokens=3200,
    cached_tokens=500,  # 500 cached tokens from previous context
    latency_ms=2800,
    tags=["production", "analysis"],
    metadata={
        "endpoint": "/v1/messages",
        "temperature": 1.0,
        "max_tokens": 2000
    }
)

try:
    result = client.submit_usage(usage)
    print(f"✓ Successfully submitted Anthropic usage record")
    print(f"  Record ID: {usage.id}")
    print(f"  Tokens: {usage.prompt_tokens} prompt + {usage.completion_tokens} completion")
    print(f"  Cached: {usage.cached_tokens} tokens (reduced cost)")
    print(f"  Result: {result}")
except Exception as e:
    print(f"✗ Error submitting usage: {e}")
```

Run the script:

```bash
python track_anthropic.py
```

#### Step 4.3: Create Google Usage Record

Create a new file `track_google.py`:

```python
#!/usr/bin/env python3
"""Track Google Gemini usage"""

from cost_ops_client import CostOpsClient, UsageRecord, generate_usage_id, current_timestamp

client = CostOpsClient(base_url="http://localhost:8080")

# Create Google Gemini Pro usage record
usage = UsageRecord(
    id=generate_usage_id(),
    timestamp=current_timestamp(),
    provider="google",
    model_name="gemini-pro",
    organization_id="org-acme-corp",
    project_id="proj-translation",
    user_id="user-charlie",
    prompt_tokens=800,
    completion_tokens=400,
    total_tokens=1200,
    latency_ms=1500,
    tags=["production", "translation"],
    metadata={
        "endpoint": "/v1/models/gemini-pro:generateContent",
        "temperature": 0.5
    }
)

try:
    result = client.submit_usage(usage)
    print(f"✓ Successfully submitted Google usage record")
    print(f"  Record ID: {usage.id}")
    print(f"  Tokens: {usage.prompt_tokens} prompt + {usage.completion_tokens} completion = {usage.total_tokens} total")
    print(f"  Result: {result}")
except Exception as e:
    print(f"✗ Error submitting usage: {e}")
```

Run the script:

```bash
python track_google.py
```

### For TypeScript Users:

#### Step 4.1: Create OpenAI Usage Record

Create a new file `track-openai.ts`:

```typescript
#!/usr/bin/env ts-node
/**
 * Track OpenAI GPT-4 usage
 */

import { CostOpsClient, UsageRecord, generateUsageId, currentTimestamp } from './cost-ops-client';

const client = new CostOpsClient('http://localhost:8080');

// Create OpenAI GPT-4 usage record
const usage: UsageRecord = {
  id: generateUsageId(),
  timestamp: currentTimestamp(),
  provider: 'openai',
  model_name: 'gpt-4',
  organization_id: 'org-acme-corp',
  project_id: 'proj-chatbot',
  user_id: 'user-alice',
  prompt_tokens: 1500,
  completion_tokens: 800,
  total_tokens: 2300,
  latency_ms: 3200,
  tags: ['production', 'chatbot'],
  metadata: {
    endpoint: '/v1/chat/completions',
    temperature: 0.7,
    max_tokens: 1000
  }
};

// Submit usage record
client.submitUsage(usage)
  .then(result => {
    console.log('✓ Successfully submitted OpenAI usage record');
    console.log(`  Record ID: ${usage.id}`);
    console.log(`  Tokens: ${usage.prompt_tokens} prompt + ${usage.completion_tokens} completion = ${usage.total_tokens} total`);
    console.log(`  Result:`, result);
  })
  .catch(error => {
    console.error('✗ Error submitting usage:', error.message);
  });
```

Run the script:

```bash
npx ts-node track-openai.ts
```

#### Step 4.2: Create Anthropic Usage Record

Create `track-anthropic.ts`:

```typescript
#!/usr/bin/env ts-node
/**
 * Track Anthropic Claude usage
 */

import { CostOpsClient, UsageRecord, generateUsageId, currentTimestamp } from './cost-ops-client';

const client = new CostOpsClient('http://localhost:8080');

const usage: UsageRecord = {
  id: generateUsageId(),
  timestamp: currentTimestamp(),
  provider: 'anthropic',
  model_name: 'claude-3-sonnet-20240229',
  organization_id: 'org-acme-corp',
  project_id: 'proj-analysis',
  user_id: 'user-bob',
  prompt_tokens: 2000,
  completion_tokens: 1200,
  total_tokens: 3200,
  cached_tokens: 500,
  latency_ms: 2800,
  tags: ['production', 'analysis'],
  metadata: {
    endpoint: '/v1/messages',
    temperature: 1.0,
    max_tokens: 2000
  }
};

client.submitUsage(usage)
  .then(result => {
    console.log('✓ Successfully submitted Anthropic usage record');
    console.log(`  Record ID: ${usage.id}`);
    console.log(`  Tokens: ${usage.prompt_tokens} prompt + ${usage.completion_tokens} completion`);
    console.log(`  Cached: ${usage.cached_tokens} tokens (reduced cost)`);
    console.log(`  Result:`, result);
  })
  .catch(error => {
    console.error('✗ Error submitting usage:', error.message);
  });
```

Run the script:

```bash
npx ts-node track-anthropic.ts
```

#### Step 4.3: Create Google Usage Record

Create `track-google.ts`:

```typescript
#!/usr/bin/env ts-node
/**
 * Track Google Gemini usage
 */

import { CostOpsClient, UsageRecord, generateUsageId, currentTimestamp } from './cost-ops-client';

const client = new CostOpsClient('http://localhost:8080');

const usage: UsageRecord = {
  id: generateUsageId(),
  timestamp: currentTimestamp(),
  provider: 'google',
  model_name: 'gemini-pro',
  organization_id: 'org-acme-corp',
  project_id: 'proj-translation',
  user_id: 'user-charlie',
  prompt_tokens: 800,
  completion_tokens: 400,
  total_tokens: 1200,
  latency_ms: 1500,
  tags: ['production', 'translation'],
  metadata: {
    endpoint: '/v1/models/gemini-pro:generateContent',
    temperature: 0.5
  }
};

client.submitUsage(usage)
  .then(result => {
    console.log('✓ Successfully submitted Google usage record');
    console.log(`  Record ID: ${usage.id}`);
    console.log(`  Tokens: ${usage.prompt_tokens} prompt + ${usage.completion_tokens} completion = ${usage.total_tokens} total`);
    console.log(`  Result:`, result);
  })
  .catch(error => {
    console.error('✗ Error submitting usage:', error.message);
  });
```

Run the script:

```bash
npx ts-node track-google.ts
```

## Part 5: Querying Cost Data

Now that we've submitted usage records, let's query the cost data.

### For Python Users:

Create a new file `query_costs.py`:

```python
#!/usr/bin/env python3
"""Query cost data with various filters"""

from cost_ops_client import CostOpsClient
from datetime import datetime, timedelta
import json

client = CostOpsClient(base_url="http://localhost:8080")

print("=" * 60)
print("COST DATA QUERIES")
print("=" * 60)

# Query 1: All costs from the last 24 hours
print("\n1. All costs from the last 24 hours:")
print("-" * 60)

end_date = datetime.utcnow()
start_date = end_date - timedelta(days=1)

costs = client.get_costs(
    start_date=start_date.isoformat(),
    end_date=end_date.isoformat()
)

for cost in costs:
    print(f"  Provider: {cost['provider']:<15} Model: {cost['model']:<30} Cost: ${cost['total_cost']}")

print(f"\nTotal records: {len(costs)}")

# Query 2: Filter by provider (OpenAI only)
print("\n2. OpenAI costs only:")
print("-" * 60)

openai_costs = client.get_costs(
    start_date=start_date.isoformat(),
    end_date=end_date.isoformat(),
    provider="openai"
)

total_openai = sum(float(c['total_cost']) for c in openai_costs)
print(f"  Total OpenAI cost: ${total_openai:.6f}")
print(f"  Number of requests: {len(openai_costs)}")

# Query 3: Filter by model
print("\n3. GPT-4 costs:")
print("-" * 60)

gpt4_costs = client.get_costs(
    start_date=start_date.isoformat(),
    end_date=end_date.isoformat(),
    model="gpt-4"
)

for cost in gpt4_costs:
    print(f"  Input: ${cost['input_cost']:<10} Output: ${cost['output_cost']:<10} Total: ${cost['total_cost']}")

# Query 4: Filter by organization
print("\n4. Costs for org-acme-corp:")
print("-" * 60)

org_costs = client.get_costs(
    start_date=start_date.isoformat(),
    end_date=end_date.isoformat(),
    organization_id="org-acme-corp"
)

print(f"  Total cost for organization: ${sum(float(c['total_cost']) for c in org_costs):.6f}")
print(f"  Total requests: {len(org_costs)}")

# Query 5: Filter by project
print("\n5. Costs for chatbot project:")
print("-" * 60)

project_costs = client.get_costs(
    start_date=start_date.isoformat(),
    end_date=end_date.isoformat(),
    project_id="proj-chatbot"
)

for cost in project_costs:
    print(f"  {cost['timestamp']}: ${cost['total_cost']} ({cost['model']})")

print("\n" + "=" * 60)
```

Run the query script:

```bash
python query_costs.py
```

### For TypeScript Users:

Create a new file `query-costs.ts`:

```typescript
#!/usr/bin/env ts-node
/**
 * Query cost data with various filters
 */

import { CostOpsClient } from './cost-ops-client';

const client = new CostOpsClient('http://localhost:8080');

async function queryCosts() {
  console.log('='.repeat(60));
  console.log('COST DATA QUERIES');
  console.log('='.repeat(60));

  const endDate = new Date();
  const startDate = new Date(endDate.getTime() - 24 * 60 * 60 * 1000);

  // Query 1: All costs from the last 24 hours
  console.log('\n1. All costs from the last 24 hours:');
  console.log('-'.repeat(60));

  const costs = await client.getCosts({
    start_date: startDate.toISOString(),
    end_date: endDate.toISOString()
  });

  costs.forEach(cost => {
    console.log(`  Provider: ${cost.provider.padEnd(15)} Model: ${cost.model.padEnd(30)} Cost: $${cost.total_cost}`);
  });

  console.log(`\nTotal records: ${costs.length}`);

  // Query 2: Filter by provider (OpenAI only)
  console.log('\n2. OpenAI costs only:');
  console.log('-'.repeat(60));

  const openaiCosts = await client.getCosts({
    start_date: startDate.toISOString(),
    end_date: endDate.toISOString(),
    provider: 'openai'
  });

  const totalOpenai = openaiCosts.reduce((sum, c) => sum + parseFloat(c.total_cost), 0);
  console.log(`  Total OpenAI cost: $${totalOpenai.toFixed(6)}`);
  console.log(`  Number of requests: ${openaiCosts.length}`);

  // Query 3: Filter by model
  console.log('\n3. GPT-4 costs:');
  console.log('-'.repeat(60));

  const gpt4Costs = await client.getCosts({
    start_date: startDate.toISOString(),
    end_date: endDate.toISOString(),
    model: 'gpt-4'
  });

  gpt4Costs.forEach(cost => {
    console.log(`  Input: $${cost.input_cost.padEnd(10)} Output: $${cost.output_cost.padEnd(10)} Total: $${cost.total_cost}`);
  });

  // Query 4: Filter by organization
  console.log('\n4. Costs for org-acme-corp:');
  console.log('-'.repeat(60));

  const orgCosts = await client.getCosts({
    start_date: startDate.toISOString(),
    end_date: endDate.toISOString(),
    organization_id: 'org-acme-corp'
  });

  const totalOrg = orgCosts.reduce((sum, c) => sum + parseFloat(c.total_cost), 0);
  console.log(`  Total cost for organization: $${totalOrg.toFixed(6)}`);
  console.log(`  Total requests: ${orgCosts.length}`);

  // Query 5: Filter by project
  console.log('\n5. Costs for chatbot project:');
  console.log('-'.repeat(60));

  const projectCosts = await client.getCosts({
    start_date: startDate.toISOString(),
    end_date: endDate.toISOString(),
    project_id: 'proj-chatbot'
  });

  projectCosts.forEach(cost => {
    console.log(`  ${cost.timestamp}: $${cost.total_cost} (${cost.model})`);
  });

  console.log('\n' + '='.repeat(60));
}

queryCosts().catch(console.error);
```

Run the query script:

```bash
npx ts-node query-costs.ts
```

## Part 6: Generating Cost Summaries

Let's create aggregated cost summaries with grouping.

### For Python Users:

Create `generate_summary.py`:

```python
#!/usr/bin/env python3
"""Generate cost summaries with different groupings"""

from cost_ops_client import CostOpsClient
from datetime import datetime, timedelta
import json

client = CostOpsClient(base_url="http://localhost:8080")

end_date = datetime.utcnow()
start_date = end_date - timedelta(days=7)

print("=" * 70)
print("COST SUMMARIES")
print("=" * 70)

# Summary 1: Overall summary
print("\n1. Overall Summary (Last 7 Days):")
print("-" * 70)

summary = client.get_summary(
    start_date=start_date.isoformat(),
    end_date=end_date.isoformat()
)

print(f"  Total Cost: ${summary['total_cost']}")
print(f"  Total Requests: {summary['total_requests']}")
print(f"  Avg Cost/Request: ${summary['avg_cost_per_request']}")

# Summary 2: Group by provider
print("\n2. Grouped by Provider:")
print("-" * 70)

summary_by_provider = client.get_summary(
    start_date=start_date.isoformat(),
    end_date=end_date.isoformat(),
    group_by=['provider']
)

print(f"  {'Provider':<20} {'Total Cost':<15} {'Requests':<10}")
print(f"  {'-'*20} {'-'*15} {'-'*10}")
for provider, cost in summary_by_provider['by_provider'].items():
    print(f"  {provider:<20} ${cost:<14} {'-':<10}")

# Summary 3: Group by model
print("\n3. Grouped by Model:")
print("-" * 70)

summary_by_model = client.get_summary(
    start_date=start_date.isoformat(),
    end_date=end_date.isoformat(),
    group_by=['model']
)

print(f"  {'Model':<35} {'Total Cost':<15}")
print(f"  {'-'*35} {'-'*15}")
for model, cost in summary_by_model['by_model'].items():
    print(f"  {model:<35} ${cost:<14}")

# Summary 4: Group by project
print("\n4. Grouped by Project:")
print("-" * 70)

summary_by_project = client.get_summary(
    start_date=start_date.isoformat(),
    end_date=end_date.isoformat(),
    group_by=['project']
)

if 'by_project' in summary_by_project and summary_by_project['by_project']:
    print(f"  {'Project':<25} {'Total Cost':<15}")
    print(f"  {'-'*25} {'-'*15}")
    for project, cost in summary_by_project['by_project'].items():
        print(f"  {project:<25} ${cost:<14}")
else:
    print("  No project-level data available")

# Summary 5: Organization-specific summary
print("\n5. Summary for org-acme-corp:")
print("-" * 70)

org_summary = client.get_summary(
    start_date=start_date.isoformat(),
    end_date=end_date.isoformat(),
    organization_id="org-acme-corp",
    group_by=['provider', 'model']
)

print(f"  Total Cost: ${org_summary['total_cost']}")
print(f"  Total Requests: {org_summary['total_requests']}")
print(f"\n  Breakdown by Provider:")
for provider, cost in org_summary['by_provider'].items():
    print(f"    {provider}: ${cost}")

print("\n" + "=" * 70)
```

Run the summary script:

```bash
python generate_summary.py
```

### For TypeScript Users:

Create `generate-summary.ts`:

```typescript
#!/usr/bin/env ts-node
/**
 * Generate cost summaries with different groupings
 */

import { CostOpsClient } from './cost-ops-client';

const client = new CostOpsClient('http://localhost:8080');

async function generateSummaries() {
  const endDate = new Date();
  const startDate = new Date(endDate.getTime() - 7 * 24 * 60 * 60 * 1000);

  console.log('='.repeat(70));
  console.log('COST SUMMARIES');
  console.log('='.repeat(70));

  // Summary 1: Overall summary
  console.log('\n1. Overall Summary (Last 7 Days):');
  console.log('-'.repeat(70));

  const summary = await client.getSummary(
    startDate.toISOString(),
    endDate.toISOString()
  );

  console.log(`  Total Cost: $${summary.total_cost}`);
  console.log(`  Total Requests: ${summary.total_requests}`);
  console.log(`  Avg Cost/Request: $${summary.avg_cost_per_request}`);

  // Summary 2: Group by provider
  console.log('\n2. Grouped by Provider:');
  console.log('-'.repeat(70));

  const summaryByProvider = await client.getSummary(
    startDate.toISOString(),
    endDate.toISOString(),
    undefined,
    ['provider']
  );

  console.log(`  ${'Provider'.padEnd(20)} ${'Total Cost'.padEnd(15)} ${'Requests'.padEnd(10)}`);
  console.log(`  ${'-'.repeat(20)} ${'-'.repeat(15)} ${'-'.repeat(10)}`);

  for (const [provider, cost] of Object.entries(summaryByProvider.by_provider)) {
    console.log(`  ${provider.padEnd(20)} $${cost.padEnd(14)} ${'-'.padEnd(10)}`);
  }

  // Summary 3: Group by model
  console.log('\n3. Grouped by Model:');
  console.log('-'.repeat(70));

  const summaryByModel = await client.getSummary(
    startDate.toISOString(),
    endDate.toISOString(),
    undefined,
    ['model']
  );

  console.log(`  ${'Model'.padEnd(35)} ${'Total Cost'.padEnd(15)}`);
  console.log(`  ${'-'.repeat(35)} ${'-'.repeat(15)}`);

  for (const [model, cost] of Object.entries(summaryByModel.by_model)) {
    console.log(`  ${model.padEnd(35)} $${cost.padEnd(14)}`);
  }

  // Summary 4: Group by project
  console.log('\n4. Grouped by Project:');
  console.log('-'.repeat(70));

  const summaryByProject = await client.getSummary(
    startDate.toISOString(),
    endDate.toISOString(),
    undefined,
    ['project']
  );

  if (summaryByProject.by_project && Object.keys(summaryByProject.by_project).length > 0) {
    console.log(`  ${'Project'.padEnd(25)} ${'Total Cost'.padEnd(15)}`);
    console.log(`  ${'-'.repeat(25)} ${'-'.repeat(15)}`);

    for (const [project, cost] of Object.entries(summaryByProject.by_project)) {
      console.log(`  ${project.padEnd(25)} $${cost.padEnd(14)}`);
    }
  } else {
    console.log('  No project-level data available');
  }

  // Summary 5: Organization-specific summary
  console.log('\n5. Summary for org-acme-corp:');
  console.log('-'.repeat(70));

  const orgSummary = await client.getSummary(
    startDate.toISOString(),
    endDate.toISOString(),
    'org-acme-corp',
    ['provider', 'model']
  );

  console.log(`  Total Cost: $${orgSummary.total_cost}`);
  console.log(`  Total Requests: ${orgSummary.total_requests}`);
  console.log(`\n  Breakdown by Provider:`);

  for (const [provider, cost] of Object.entries(orgSummary.by_provider)) {
    console.log(`    ${provider}: $${cost}`);
  }

  console.log('\n' + '='.repeat(70));
}

generateSummaries().catch(console.error);
```

Run the summary script:

```bash
npx ts-node generate-summary.ts
```

## Part 7: Batch Operations

Learn to submit multiple usage records efficiently.

### For Python Users:

Create `batch_submit.py`:

```python
#!/usr/bin/env python3
"""Submit multiple usage records in a batch"""

from cost_ops_client import CostOpsClient, UsageRecord, generate_usage_id, current_timestamp
from datetime import datetime, timedelta
import random

client = CostOpsClient(base_url="http://localhost:8080")

# Generate 10 sample usage records
usage_records = []

providers = ["openai", "anthropic", "google"]
models = {
    "openai": ["gpt-4", "gpt-3.5-turbo", "gpt-4-turbo"],
    "anthropic": ["claude-3-sonnet-20240229", "claude-3-opus-20240229"],
    "google": ["gemini-pro", "gemini-ultra"]
}
projects = ["proj-chatbot", "proj-analysis", "proj-translation", "proj-summarization"]
users = ["user-alice", "user-bob", "user-charlie", "user-diana"]

print("Generating 10 sample usage records...")
print("=" * 60)

for i in range(10):
    provider = random.choice(providers)
    model = random.choice(models[provider])

    # Generate realistic token counts
    prompt_tokens = random.randint(500, 3000)
    completion_tokens = random.randint(200, 2000)

    usage = UsageRecord(
        id=generate_usage_id(),
        timestamp=(datetime.utcnow() - timedelta(hours=random.randint(0, 24))).isoformat(),
        provider=provider,
        model_name=model,
        organization_id="org-acme-corp",
        project_id=random.choice(projects),
        user_id=random.choice(users),
        prompt_tokens=prompt_tokens,
        completion_tokens=completion_tokens,
        total_tokens=prompt_tokens + completion_tokens,
        cached_tokens=random.randint(0, prompt_tokens // 2) if provider == "anthropic" else None,
        latency_ms=random.randint(1000, 5000),
        tags=["production", "batch-test"],
        metadata={"batch_id": "batch-001"}
    )

    usage_records.append(usage)
    print(f"  {i+1}. {provider:<12} {model:<35} Tokens: {usage.total_tokens}")

print("\n" + "=" * 60)
print("Submitting batch...")

try:
    result = client.submit_batch_usage(usage_records)
    print(f"✓ Successfully submitted {len(usage_records)} usage records")
    print(f"  Total cost: ${result.get('total_cost', 'N/A')}")
    print(f"  Processing time: {result.get('processing_time_ms', 'N/A')}ms")
except Exception as e:
    print(f"✗ Error submitting batch: {e}")

print("=" * 60)
```

Run the batch script:

```bash
python batch_submit.py
```

### For TypeScript Users:

Create `batch-submit.ts`:

```typescript
#!/usr/bin/env ts-node
/**
 * Submit multiple usage records in a batch
 */

import { CostOpsClient, UsageRecord, generateUsageId, currentTimestamp } from './cost-ops-client';

const client = new CostOpsClient('http://localhost:8080');

async function batchSubmit() {
  const usageRecords: UsageRecord[] = [];

  const providers = ['openai', 'anthropic', 'google'];
  const models: Record<string, string[]> = {
    openai: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo'],
    anthropic: ['claude-3-sonnet-20240229', 'claude-3-opus-20240229'],
    google: ['gemini-pro', 'gemini-ultra']
  };
  const projects = ['proj-chatbot', 'proj-analysis', 'proj-translation', 'proj-summarization'];
  const users = ['user-alice', 'user-bob', 'user-charlie', 'user-diana'];

  console.log('Generating 10 sample usage records...');
  console.log('='.repeat(60));

  for (let i = 0; i < 10; i++) {
    const provider = providers[Math.floor(Math.random() * providers.length)];
    const model = models[provider][Math.floor(Math.random() * models[provider].length)];

    const promptTokens = Math.floor(Math.random() * 2500) + 500;
    const completionTokens = Math.floor(Math.random() * 1800) + 200;

    const hoursAgo = Math.floor(Math.random() * 24);
    const timestamp = new Date(Date.now() - hoursAgo * 60 * 60 * 1000).toISOString();

    const usage: UsageRecord = {
      id: generateUsageId(),
      timestamp,
      provider,
      model_name: model,
      organization_id: 'org-acme-corp',
      project_id: projects[Math.floor(Math.random() * projects.length)],
      user_id: users[Math.floor(Math.random() * users.length)],
      prompt_tokens: promptTokens,
      completion_tokens: completionTokens,
      total_tokens: promptTokens + completionTokens,
      cached_tokens: provider === 'anthropic' ? Math.floor(Math.random() * promptTokens / 2) : undefined,
      latency_ms: Math.floor(Math.random() * 4000) + 1000,
      tags: ['production', 'batch-test'],
      metadata: { batch_id: 'batch-001' }
    };

    usageRecords.push(usage);
    console.log(`  ${i + 1}. ${provider.padEnd(12)} ${model.padEnd(35)} Tokens: ${usage.total_tokens}`);
  }

  console.log('\n' + '='.repeat(60));
  console.log('Submitting batch...');

  try {
    const result = await client.submitBatchUsage(usageRecords);
    console.log(`✓ Successfully submitted ${usageRecords.length} usage records`);
    console.log(`  Total cost: $${result.total_cost || 'N/A'}`);
    console.log(`  Processing time: ${result.processing_time_ms || 'N/A'}ms`);
  } catch (error: any) {
    console.error(`✗ Error submitting batch: ${error.message}`);
  }

  console.log('='.repeat(60));
}

batchSubmit().catch(console.error);
```

Run the batch script:

```bash
npx ts-node batch-submit.ts
```

## Part 8: Exporting Data

Learn to export cost data in different formats.

Using the CLI (works for both Python and TypeScript users):

```bash
# Export to JSON
cd /path/to/llm-cost-ops

# Last 7 days
./target/release/cost-ops export \
  --output costs-last-7-days.json \
  --format json \
  --period last-7-days

# View the exported file
head -20 costs-last-7-days.json

# Export to CSV
./target/release/cost-ops export \
  --output costs-last-7-days.csv \
  --format csv \
  --period last-7-days

# View the CSV file
head -10 costs-last-7-days.csv

# Export with filters
./target/release/cost-ops export \
  --output openai-costs.csv \
  --format csv \
  --period last-30-days \
  --provider openai

# Export for specific organization
./target/release/cost-ops export \
  --output org-costs.json \
  --format json \
  --period last-30-days \
  --organization org-acme-corp
```

## Exercises and Challenges

Now it's time to practice what you've learned!

### Exercise 1: Multi-Provider Comparison

**Task:** Create a script that submits usage for all three providers (OpenAI, Anthropic, Google) with identical token counts, then compares the costs.

**Requirements:**
- Use 2000 prompt tokens and 1000 completion tokens for each
- Submit to all three providers
- Query the results
- Calculate and display the cost difference

**Expected Skills:**
- Creating usage records
- Batch submissions
- Querying with filters
- Cost analysis

### Exercise 2: Project Cost Analysis

**Task:** Create 20 usage records across 4 different projects, then generate a summary showing which project has the highest costs.

**Requirements:**
- Use at least 2 different providers
- Distribute records across projects: chatbot (8), analysis (6), translation (4), summarization (2)
- Generate a summary grouped by project
- Identify the most expensive project

### Exercise 3: Time-Based Analysis

**Task:** Create usage records with timestamps spread across the last 7 days, then query costs day by day.

**Requirements:**
- Create 14 records (2 per day for 7 days)
- Query costs for each day separately
- Calculate daily totals
- Identify the most expensive day

### Exercise 4: Cost Forecasting Preparation

**Task:** Generate a realistic dataset of 30 usage records over the past 30 days that shows increasing usage over time.

**Requirements:**
- Use a linear growth pattern (start: 500 tokens/day, end: 2000 tokens/day)
- Mix multiple providers
- Tag records appropriately
- Export the data to JSON

### Exercise 5: Custom Cost Dashboard

**Task:** Create a comprehensive dashboard script that shows:
- Total costs by provider
- Most expensive model
- Average cost per request
- Costs by user
- Daily cost trend

**Requirements:**
- Use only API calls (no CLI)
- Format output as a readable dashboard
- Include at least 5 different metrics

## Solution Code

### Solution to Exercise 1 (Python):

```python
#!/usr/bin/env python3
"""Solution: Multi-Provider Comparison"""

from cost_ops_client import CostOpsClient, UsageRecord, generate_usage_id, current_timestamp

client = CostOpsClient(base_url="http://localhost:8080")

# Standard token counts for comparison
PROMPT_TOKENS = 2000
COMPLETION_TOKENS = 1000
TOTAL_TOKENS = PROMPT_TOKENS + COMPLETION_TOKENS

providers = [
    ("openai", "gpt-4"),
    ("anthropic", "claude-3-sonnet-20240229"),
    ("google", "gemini-pro")
]

print("=" * 70)
print("MULTI-PROVIDER COST COMPARISON")
print("=" * 70)
print(f"\nUsing standard workload:")
print(f"  Prompt tokens: {PROMPT_TOKENS}")
print(f"  Completion tokens: {COMPLETION_TOKENS}")
print(f"  Total tokens: {TOTAL_TOKENS}\n")

results = []

for provider, model in providers:
    usage = UsageRecord(
        id=generate_usage_id(),
        timestamp=current_timestamp(),
        provider=provider,
        model_name=model,
        organization_id="org-comparison-test",
        project_id="proj-comparison",
        prompt_tokens=PROMPT_TOKENS,
        completion_tokens=COMPLETION_TOKENS,
        total_tokens=TOTAL_TOKENS
    )

    result = client.submit_usage(usage)
    cost = float(result['cost'])
    results.append((provider, model, cost))
    print(f"✓ {provider:<12} {model:<35} Cost: ${cost:.6f}")

# Find cheapest and most expensive
results.sort(key=lambda x: x[2])
cheapest = results[0]
most_expensive = results[-1]

print("\n" + "-" * 70)
print(f"Cheapest:       {cheapest[0]:<12} {cheapest[1]:<35} ${cheapest[2]:.6f}")
print(f"Most Expensive: {most_expensive[0]:<12} {most_expensive[1]:<35} ${most_expensive[2]:.6f}")
print(f"Difference:     ${most_expensive[2] - cheapest[2]:.6f} ({((most_expensive[2]/cheapest[2])-1)*100:.1f}% more)")
print("=" * 70)
```

## Troubleshooting Guide

### Common Issues and Solutions

#### Issue 1: Connection Refused

**Symptom:**
```
Error: Connection refused (os error 111)
```

**Solution:**
- Ensure the API server is running: `ps aux | grep cost-ops`
- Check the server logs for errors
- Verify the port is correct (default: 8080)
- Try restarting the server

#### Issue 2: No Pricing Found

**Symptom:**
```
Error: No active pricing found for provider=X, model=Y
```

**Solution:**
```bash
# List current pricing
./target/release/cost-ops pricing list

# Add missing pricing
./target/release/cost-ops pricing add \
  --provider PROVIDER \
  --model MODEL \
  --input-price X \
  --output-price Y
```

#### Issue 3: Database Locked

**Symptom:**
```
Error: database is locked
```

**Solution:**
- Only one process can write to SQLite at a time
- Close other instances of cost-ops
- Or switch to PostgreSQL for concurrent access

#### Issue 4: Import Errors (Python)

**Symptom:**
```
ModuleNotFoundError: No module named 'requests'
```

**Solution:**
```bash
# Ensure virtual environment is activated
source venv/bin/activate

# Reinstall dependencies
pip install -r requirements.txt
```

#### Issue 5: TypeScript Compilation Errors

**Symptom:**
```
error TS2307: Cannot find module
```

**Solution:**
```bash
# Reinstall node modules
rm -rf node_modules package-lock.json
npm install

# Verify TypeScript is installed
npx tsc --version
```

## Review Questions

Test your understanding:

1. **Q:** What are the three main token types tracked in usage records?
   **A:** Prompt tokens, completion tokens, and cached tokens (optional).

2. **Q:** How does the Cost Ops platform calculate costs?
   **A:** It multiplies token counts by the pricing rate (per million tokens) from the active pricing table for the provider/model combination.

3. **Q:** What is the benefit of batch submissions over individual submissions?
   **A:** Better performance, reduced network overhead, and atomic operations (all succeed or all fail).

4. **Q:** How can you filter cost queries?
   **A:** By date range, provider, model, organization_id, project_id, and user_id.

5. **Q:** What formats can you export data to?
   **A:** JSON, CSV, Excel (XLSX), and JSON Lines.

6. **Q:** Why would Anthropic costs be lower with cached tokens?
   **A:** Cached tokens have a discounted rate (typically 50% off) because they're reused from previous context.

7. **Q:** What's the difference between input_cost and output_cost?
   **A:** Input_cost is for prompt tokens, output_cost is for completion tokens. Most providers charge different rates for each.

8. **Q:** How do you group costs in a summary?
   **A:** Use the group_by parameter with values like 'provider', 'model', 'project', or combinations.

## Next Steps

Congratulations on completing Lab 1! You now have a solid foundation in cost tracking. Here's what to explore next:

1. **Lab 2: Analytics and Reporting**
   - Build custom dashboards
   - Create automated reports
   - Time-series analysis

2. **Lab 3: Budget Management**
   - Set up budgets
   - Configure alerts
   - Monitor consumption

3. **Advanced Topics:**
   - Setting up PostgreSQL for production
   - Configuring authentication
   - Setting up monitoring

## Additional Resources

- **Documentation:** `/docs` directory in the repository
- **API Reference:** `http://localhost:8080/api/docs` (when server is running)
- **Example Code:** `/examples` directory
- **Community:** GitHub Discussions

## Lab Completion Checklist

Mark off each item as you complete it:

- [ ] Environment setup complete
- [ ] SDK installed and configured
- [ ] API server running successfully
- [ ] Created OpenAI usage records
- [ ] Created Anthropic usage records
- [ ] Created Google usage records
- [ ] Successfully queried cost data
- [ ] Generated cost summaries
- [ ] Performed batch submissions
- [ ] Exported data to JSON
- [ ] Exported data to CSV
- [ ] Completed at least 2 exercises
- [ ] Understood all review questions

**Estimated completion time:** 60-90 minutes

**Your actual time:** __________ minutes

## Feedback

Help us improve this lab:
- What sections were unclear?
- What additional examples would be helpful?
- What topics need more depth?

Submit feedback to: feedback@llm-cost-ops.example.com

---

**End of Lab 1**
