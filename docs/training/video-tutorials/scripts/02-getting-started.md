# Video 02: Getting Started with LLM Cost Ops

## Metadata

- **Duration**: 18-20 minutes
- **Level**: Beginner
- **Prerequisites**: Video 01 (Introduction)
- **Target Audience**: Developers ready to implement cost tracking
- **Video ID**: LLMCO-V02-SETUP
- **Version**: 1.0.0

## Learning Objectives

By the end of this video, viewers will be able to:
- Install LLM Cost Ops on their local development machine
- Configure the platform with environment variables
- Set up their first project and get API keys
- Instrument a simple application to track LLM costs
- View their first cost data in the dashboard
- Understand basic configuration options

## Equipment/Software Needed

### Recording
- Screen recording software (1920x1080, 30fps)
- Professional microphone
- Clean terminal with syntax highlighting

### Demonstration Environment
- macOS or Linux system (show both options)
- Docker Desktop installed
- Node.js 18+ or Python 3.9+
- Code editor (VS Code with extensions)
- Terminal application
- Web browser

### Required Accounts
- OpenAI API key (for demo)
- Fresh LLM Cost Ops installation

## Scene Breakdown

### Scene 1: Opening & Overview
**Duration**: 0:00-1:00

**Visual**:
- Animated intro with episode title
- Quick recap of Video 01
- Today's agenda displayed

**Narration**:
"Welcome back! In the last video, we introduced LLM Cost Ops and explored what it can do. Today, we're going hands-on. By the end of this tutorial, you'll have LLM Cost Ops running on your machine, tracking costs from a real application.

We'll start with installation, configure the platform, create our first project, and track some actual LLM requests. Let's dive in!"

**On-Screen Text**:
- "Getting Started with LLM Cost Ops"
- "Today's Agenda:"
  - "✓ Installation"
  - "✓ Configuration"
  - "✓ First Project"
  - "✓ Track Your First Request"

**Transition**: Wipe to desktop

---

### Scene 2: System Requirements
**Duration**: 1:00-2:00

**Visual**:
- Clean desktop
- Documentation page showing requirements
- Terminal window ready

**Narration**:
"Before we begin, let's check the requirements. LLM Cost Ops is lightweight and runs on most modern systems.

You'll need Docker and Docker Compose for the easiest setup, or you can install natively. For this tutorial, I'll use Docker because it's the fastest way to get started.

You'll also need Node.js or Python if you want to use the SDKs. And of course, an API key from at least one LLM provider—we'll use OpenAI for our examples."

**On-Screen Text**:
- "Requirements:"
  - "✓ Docker & Docker Compose (recommended)"
  - "✓ OR: PostgreSQL/SQLite for native install"
  - "✓ Node.js 18+ OR Python 3.9+ (for SDKs)"
  - "✓ LLM Provider API Key (OpenAI, Anthropic, etc.)"

**Code/Demo**:
```bash
# Check versions
docker --version
# Docker version 24.0.0

docker compose version
# Docker Compose version v2.20.0

node --version
# v20.10.0

python --version
# Python 3.11.5
```

**Transition**: Terminal takes full screen

---

### Scene 3: Installation - Docker Method
**Duration**: 2:00-5:00

**Visual**:
- Full-screen terminal
- Split view: terminal on left, file editor on right
- Docker logs streaming in real-time

**Narration**:
"Let's install LLM Cost Ops using Docker. First, I'll clone the repository from GitHub.

[Pause for command execution]

Now, let's look at the docker-compose file. This sets up three services: the main application, PostgreSQL database, and Redis for caching.

The defaults are fine for development, but let's configure a few environment variables. I'll copy the example env file and edit it.

[Show .env file editing]

Here are the important settings: Database connection, secret key for sessions, and the port where the web interface will run. I'm keeping the defaults—port 8080 for the web UI and 5432 for PostgreSQL.

Now, let's start it up with docker compose.

[Show startup sequence]

Perfect! You can see the services starting. The database migrations run automatically. The API server is initializing. And there we go—the web interface is ready on port 8080."

**On-Screen Text**:
- Step labels appear as each completes:
  - "Step 1: Clone Repository ✓"
  - "Step 2: Configure Environment ✓"
  - "Step 3: Start Services ✓"

**Code/Demo**:
```bash
# Clone the repository
git clone https://github.com/your-org/llm-cost-ops.git
cd llm-cost-ops

# Copy environment template
cp .env.example .env

# Edit .env file (show in editor)
# Key settings:
# DATABASE_URL=postgresql://postgres:postgres@db:5432/llm_cost_ops
# SECRET_KEY=<generate-random-key>
# WEB_PORT=8080
# API_PORT=8081

# Start services
docker compose up -d

# Watch logs
docker compose logs -f

# Output shows:
# [+] Running 3/3
#  ✔ Container llm-cost-ops-db    Started
#  ✔ Container llm-cost-ops-redis Started
#  ✔ Container llm-cost-ops-api   Started
#
# API server listening on :8081
# Web server listening on :8080
# Database migrations complete
```

**Highlight Callouts**:
- "Docker Compose handles all dependencies"
- "Database migrations run automatically"
- "Web UI: http://localhost:8080"

**Transition**: Browser window opens

---

### Scene 4: First Login & Setup
**Duration**: 5:00-7:00

**Visual**:
- Browser showing LLM Cost Ops login page
- Mouse movements slow and deliberate
- Forms filling out clearly

**Narration**:
"Let's open a browser and navigate to localhost:8080.

Here's the welcome screen. Since this is our first time, we need to create an admin account. I'll use a simple email and password for this demo—in production, use something secure!

[Fill out form]

Great! We're logged in. This is the main dashboard. Right now it's empty because we haven't tracked any requests yet. But notice the navigation: Projects, Analytics, Budgets, and Settings.

Let's create our first project. A project is a logical grouping of LLM usage—maybe one per application or service.

I'll call this 'demo-app' and give it a description. The API key is generated automatically—this is what we'll use in our code to send data to LLM Cost Ops."

**On-Screen Text**:
- "First-Time Setup:"
  - "Create admin account"
  - "Explore dashboard"
  - "Create first project"
  - "Get API key"

**Code/Demo**:
Navigation flow:
1. Open http://localhost:8080
2. Click "Create Account"
3. Fill form:
   - Email: demo@example.com
   - Password: ••••••••
   - Confirm: ••••••••
4. Click "Sign Up"
5. Redirect to dashboard
6. Click "New Project"
7. Fill form:
   - Name: demo-app
   - Description: Demo application for tutorial
8. Click "Create"
9. Copy API key: `lcops_demo_key_abc123...`

**Highlight Callouts**:
- "Save this API key securely!"
- "You can create multiple projects"
- "Each project has its own API key"

**Transition**: Split screen: browser + code editor

---

### Scene 5: SDK Installation (Python)
**Duration**: 7:00-9:30

**Visual**:
- Split screen: Terminal left, code editor right
- Python virtual environment setup
- Package installation progress

**Narration**:
"Now for the fun part—let's write some code. I'll show you Python first, then TypeScript.

I'm creating a new Python project and setting up a virtual environment. Best practice is to always use virtual environments for Python projects.

[Create venv and activate]

Now I'll install the LLM Cost Ops SDK along with the OpenAI library.

[Show pip install]

Perfect. Let's create a simple script that makes an LLM request and tracks the cost. I'm creating a file called 'demo.py'.

[Start typing code]

First, we import the libraries. Then we initialize the cost tracker with our API key—the one we just got from the dashboard. We also need our OpenAI API key.

Now here's the magic: we wrap our OpenAI call with the tracker. Notice the 'track' method takes the LLM call and optional metadata tags. Tags are powerful—we'll use them to categorize costs later.

Let's run it and see what happens!"

**On-Screen Text**:
- "Python Setup:"
  - "Create virtual environment"
  - "Install llm-cost-ops SDK"
  - "Install LLM provider library"
  - "Write tracking code"

**Code/Demo**:
```bash
# Create project directory
mkdir llm-cost-ops-demo
cd llm-cost-ops-demo

# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install packages
pip install llm-cost-ops openai

# Create demo script
touch demo.py
```

**demo.py** (show typing this in editor):
```python
import os
from llm_cost_ops import CostTracker
from openai import OpenAI

# Initialize tracker with API key from dashboard
tracker = CostTracker(
    api_key="lcops_demo_key_abc123...",
    endpoint="http://localhost:8081"  # Local instance
)

# Initialize OpenAI client
client = OpenAI(api_key=os.getenv("OPENAI_API_KEY"))

# Make a tracked LLM request
print("Making tracked LLM request...")

response = tracker.track(
    # The actual LLM call
    client.chat.completions.create(
        model="gpt-3.5-turbo",
        messages=[
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "What is LLM Cost Ops?"}
        ]
    ),
    # Metadata tags for categorization
    tags={
        "environment": "development",
        "feature": "chat",
        "user_id": "demo_user"
    }
)

print(f"\nResponse: {response.choices[0].message.content}")
print("\n✅ Request tracked successfully!")
print("Check the dashboard at http://localhost:8080")
```

**Highlight Callouts**:
- "tracker.track() wraps any LLM call"
- "Tags help you categorize costs"
- "Works with OpenAI, Anthropic, and more"

**Transition**: Terminal shows execution

---

### Scene 6: Running & Viewing Results
**Duration**: 9:30-11:30

**Visual**:
- Terminal showing script execution
- Browser automatically refreshes dashboard
- Split screen showing both

**Narration**:
"Let's run our script. I've set my OpenAI API key as an environment variable.

[Execute script]

Perfect! The request went through. You can see the response from GPT-3.5. But more importantly, notice that success message—the request was tracked.

Now let's refresh the dashboard and see our data.

[Switch to browser, refresh]

Excellent! Here's our first tracked request. Look at all this information: the timestamp, model used, tokens consumed, calculated cost, and the tags we added.

Click on the request to see full details. Here's the complete breakdown: 45 input tokens, 127 output tokens, total cost of $0.00026. You can see the exact messages sent and received, the response time, and all our custom tags.

This is the power of LLM Cost Ops—automatic, detailed tracking with zero performance impact on your application."

**On-Screen Text**:
- "Execution Results:"
  - "✅ LLM request successful"
  - "✅ Cost tracked automatically"
  - "✅ Data visible in dashboard"

**Code/Demo**:
```bash
# Set OpenAI API key
export OPENAI_API_KEY="sk-your-key-here"

# Run the script
python demo.py

# Output:
# Making tracked LLM request...
#
# Response: LLM Cost Ops is an open-source platform...
# (full response shown)
#
# ✅ Request tracked successfully!
# Check the dashboard at http://localhost:8080
```

Dashboard view shows:
- Request list with new entry
- Model: gpt-3.5-turbo
- Tokens: 45 in / 127 out
- Cost: $0.00026
- Tags: environment=development, feature=chat, user_id=demo_user
- Click to expand full details

**Highlight Callouts**:
- "Instant visibility into costs"
- "All metadata preserved"
- "No performance impact"

**Transition**: New code editor window

---

### Scene 7: TypeScript/JavaScript Setup
**Duration**: 11:30-14:00

**Visual**:
- New terminal window
- VS Code with TypeScript project
- npm install progress

**Narration**:
"Now let's do the same thing with TypeScript. The process is nearly identical.

I'm creating a new Node.js project and installing the TypeScript SDK along with OpenAI's library.

[Show npm install]

The SDK is fully typed, so you get excellent autocomplete and type safety in your editor. Watch as I type—see how it suggests the available options?

Here's the TypeScript equivalent of our Python code. The API is almost identical by design. We initialize the tracker, create our OpenAI client, and use the track method to wrap our LLM call.

Notice the type annotations—the SDK knows the exact type of the response, so you get full IntelliSense support.

Let's compile and run this.

[Execute]

Perfect! Another request tracked. If we check the dashboard, we'll now see two requests—one from Python, one from TypeScript. The system doesn't care what language you use; it all flows into the same tracking system."

**On-Screen Text**:
- "TypeScript Setup:"
  - "Initialize Node.js project"
  - "Install llm-cost-ops SDK"
  - "Install OpenAI library"
  - "Write type-safe code"

**Code/Demo**:
```bash
# Create new directory
mkdir llm-cost-ops-demo-ts
cd llm-cost-ops-demo-ts

# Initialize Node.js project
npm init -y

# Install dependencies
npm install llm-cost-ops openai
npm install -D typescript @types/node tsx

# Create TypeScript file
touch demo.ts
```

**demo.ts** (show in editor):
```typescript
import { CostTracker } from 'llm-cost-ops';
import OpenAI from 'openai';

// Initialize tracker
const tracker = new CostTracker({
  apiKey: 'lcops_demo_key_abc123...',
  endpoint: 'http://localhost:8081'
});

// Initialize OpenAI client
const openai = new OpenAI({
  apiKey: process.env.OPENAI_API_KEY
});

async function main() {
  console.log('Making tracked LLM request...');

  // Make tracked request
  const response = await tracker.track(
    openai.chat.completions.create({
      model: 'gpt-3.5-turbo',
      messages: [
        { role: 'system', content: 'You are a helpful assistant.' },
        { role: 'user', content: 'What is LLM Cost Ops?' }
      ]
    }),
    {
      tags: {
        environment: 'development',
        feature: 'chat',
        userId: 'demo_user_ts'
      }
    }
  );

  console.log(`\nResponse: ${response.choices[0].message.content}`);
  console.log('\n✅ Request tracked successfully!');
  console.log('Check the dashboard at http://localhost:8080');
}

main();
```

```bash
# Run with tsx
npx tsx demo.ts

# Output similar to Python version
```

**Highlight Callouts**:
- "Full TypeScript support"
- "Type-safe SDK"
- "Same API across languages"

**Transition**: Dashboard full screen

---

### Scene 8: Exploring the Dashboard
**Duration**: 14:00-16:00

**Visual**:
- Full-screen browser
- Navigate through different dashboard sections
- Highlight key features

**Narration**:
"Let's explore the dashboard in more detail. We've seen the request list, but there's much more.

Click on 'Analytics' and you'll see visual breakdowns of your costs. This graph shows spending over time. These pie charts show distribution by model, by provider, and by custom tags.

Notice how our demo requests are categorized by the tags we added? That 'feature' tag lets us see costs broken down by feature. The 'environment' tag separates development from production spending.

Under 'Projects', you can manage multiple projects, rotate API keys, and set up project-specific budgets. We'll dive deeper into budgets in a later video.

The 'Settings' section is where you configure integrations, set up alerts, and manage users. For now, the defaults are perfect for local development."

**On-Screen Text**:
Key features highlighted:
- "Real-time analytics"
- "Custom tag filtering"
- "Project management"
- "Budget controls"

**Code/Demo**:
Navigate through:
1. Dashboard → Analytics
   - Show time-series cost graph
   - Show model distribution pie chart
   - Show tag-based breakdown
2. Dashboard → Projects
   - List of projects
   - Project settings
   - API key management
3. Dashboard → Settings
   - User management
   - Integration setup
   - Alert configuration

**Highlight Callouts**:
- "Filter by any tag dimension"
- "Export data as CSV or JSON"
- "Set up custom date ranges"

**Transition**: Back to split screen

---

### Scene 9: Configuration Best Practices
**Duration**: 16:00-17:30

**Visual**:
- Code editor showing configuration files
- Terminal with environment variable examples
- Documentation snippets

**Narration**:
"Before we wrap up, let me share some configuration best practices.

Never hardcode your API keys. Use environment variables or a secrets management system. I'm showing you a .env file here for the demo, but in production, use AWS Secrets Manager, HashiCorp Vault, or your platform's secrets service.

The tracker can be configured globally or per-request. Global configuration is good for app-wide settings like the endpoint URL. Per-request configuration is useful for dynamic tags or sampling rates.

Speaking of sampling, if you have extremely high traffic, you can configure the SDK to sample requests—say, track only 10% of calls. This reduces overhead while still giving you statistical accuracy.

For local development, the endpoint is localhost. For production, you'll point to your hosted instance, which we'll cover in the enterprise deployment video."

**On-Screen Text**:
- "Best Practices:"
  - "✓ Use environment variables"
  - "✓ Never commit API keys"
  - "✓ Configure sampling for high traffic"
  - "✓ Use tags strategically"

**Code/Demo**:

**.env file:**
```bash
# LLM Cost Ops Configuration
LCOPS_API_KEY=lcops_demo_key_abc123...
LCOPS_ENDPOINT=http://localhost:8081

# LLM Provider Keys
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
```

**Python configuration example:**
```python
# Global configuration
tracker = CostTracker(
    api_key=os.getenv("LCOPS_API_KEY"),
    endpoint=os.getenv("LCOPS_ENDPOINT"),
    sampling_rate=0.1,  # Track 10% of requests
    default_tags={"app": "my-app", "env": "prod"}
)

# Per-request override
response = tracker.track(
    llm_call,
    tags={"feature": "specific-feature"},  # Merges with defaults
    force_track=True  # Ignore sampling for this request
)
```

**Highlight Callouts**:
- "Environment-specific configuration"
- "Sampling for high-volume apps"
- "Global + per-request tags"

**Transition**: Fade to recap

---

### Scene 10: Recap & Next Steps
**Duration**: 17:30-18:30

**Visual**:
- Split screen showing all the work done
- Checklist of completed items
- Preview of next video

**Narration**:
"Congratulations! You've successfully set up LLM Cost Ops. Let's recap what we accomplished.

We installed the platform using Docker, created our first project, and got our API key. We wrote code in both Python and TypeScript to track LLM requests. And we viewed our cost data in the beautiful web dashboard.

You now have a solid foundation for cost tracking. In the next videos, we'll go deeper into each SDK, showing advanced features like batch tracking, custom pricing, error handling, and optimization strategies.

If you're a Python developer, watch video three next. TypeScript developers, skip to video four. Either way, you're on your way to complete cost visibility for your LLM applications."

**On-Screen Text**:
- "What We Accomplished:"
  - "✅ Installed LLM Cost Ops"
  - "✅ Created first project"
  - "✅ Tracked requests (Python & TypeScript)"
  - "✅ Viewed data in dashboard"
- "Next Steps:"
  - "→ Video 03: Python SDK Deep Dive"
  - "→ Video 04: TypeScript SDK Guide"
  - "→ Video 05: Analytics Dashboards"

**Transition**: Fade to closing

---

### Scene 11: Closing & Call to Action
**Duration**: 18:30-19:00

**Visual**:
- Final screen with resources
- Social media links
- Thumbnail for next video

**Narration**:
"Thanks for following along! If you ran into any issues, check the documentation link in the description or join our Discord community—we're happy to help.

Don't forget to like and subscribe for more LLM Cost Ops tutorials. In the next video, we'll explore the Python SDK in depth with advanced use cases.

Happy cost tracking!"

**On-Screen Text**:
- "Resources:"
  - "📖 Documentation: docs.llm-cost-ops.dev"
  - "💬 Discord: discord.gg/llm-cost-ops"
  - "⭐ GitHub: github.com/llm-cost-ops"
  - "▶️ Next: Python SDK Deep Dive"

**Call-to-Action**:
- Like & Subscribe
- Join Discord
- Star on GitHub
- Continue to Video 03

---

## Post-Production Notes

### Chapter Markers
- 0:00 - Introduction
- 1:00 - System Requirements
- 2:00 - Docker Installation
- 5:00 - First Login & Setup
- 7:00 - Python SDK Installation
- 9:30 - Running & Viewing Results
- 11:30 - TypeScript Setup
- 14:00 - Exploring Dashboard
- 16:00 - Configuration Best Practices
- 17:30 - Recap & Next Steps

### Key Commands to Highlight
All terminal commands should be clearly visible with good contrast and copyable from description.

### Graphics Needed
- Installation flow diagram
- Architecture reminder (callback to Video 01)
- Configuration file examples (syntax highlighted)
- Dashboard screenshots for thumbnails

### Code Samples to Prepare
All code samples should be:
- Syntax highlighted
- Available as downloadable files
- Tested before recording
- Linked in video description

### Common Issues to Address
Prepare solutions for:
- Docker not running
- Port conflicts (8080 already in use)
- API key not working
- Network connectivity issues

### Thumbnail Design
- Screenshot of dashboard with data
- Text: "Get Started in 20 Minutes"
- Episode number: "02"
- High contrast, engaging

---

**Script Version**: 1.0.0
**Last Updated**: 2025-11-16
**Recording Environment**: macOS 14+ or Ubuntu 22.04+
**Estimated Production Time**: 3-4 days
