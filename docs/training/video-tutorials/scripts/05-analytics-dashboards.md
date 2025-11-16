# Video 05: Analytics Dashboards

## Metadata

- **Duration**: 17-20 minutes
- **Level**: Intermediate
- **Prerequisites**: Videos 01, 02, and either 03 or 04
- **Target Audience**: Analysts, team leads, product managers
- **Video ID**: LLMCO-V05-ANALYTICS
- **Version**: 1.0.0

## Learning Objectives

- Navigate and customize the built-in dashboard
- Create custom views and saved filters
- Build real-time analytics dashboards
- Export data for external analysis
- Set up automated reports
- Integrate with BI tools (Tableau, Looker, Metabase)
- Use the GraphQL API for custom visualizations

## Equipment/Software Needed

- LLM Cost Ops instance with sample data
- Web browser
- GraphQL client (Postman, Insomnia)
- Optional: BI tool for integration demo

## Scene Breakdown

### Scene 1: Dashboard Overview
**Duration**: 0:00-2:00

**Visual**: Tour of main dashboard interface

**Narration**:
"Welcome to analytics dashboards! The built-in dashboard is powerful, but today we'll unlock its full potential and build custom analytics. We'll explore filtering, custom views, real-time updates, data export, and integration with external BI tools."

**On-Screen Text**:
- "Topics:"
  - "Built-in dashboard mastery"
  - "Custom views & filters"
  - "Real-time analytics"
  - "Data export & reporting"
  - "BI tool integration"

---

### Scene 2: Advanced Filtering
**Duration**: 2:00-5:00

**Visual**: Dashboard with complex filter demonstrations

**Code/Demo**:
- Date range selection
- Multi-tag filtering
- Model and provider filtering
- Cost threshold filters
- Regex pattern matching for tags

**Narration**:
"Let's master filtering. You can combine multiple dimensions—filter by date range, specific tags, cost thresholds, and even regex patterns. Watch as I find all GPT-4 requests over $1 made by premium users in the last week."

**Highlight**: "Combine multiple filters • Save filter presets • Share filtered views"

---

### Scene 3: Custom Dashboards with GraphQL
**Duration**: 5:00-9:00

**Code/Demo**:
```graphql
query CustomDashboard($projectId: ID!, $startDate: DateTime!, $endDate: DateTime!) {
  project(id: $projectId) {
    costs(startDate: $startDate, endDate: $endDate) {
      totalCost
      totalTokens
      requestCount

      byModel {
        model
        cost
        percentage
      }

      byTag(tagKey: "feature") {
        tagValue
        cost
        requestCount
      }

      timeSeries(interval: HOUR) {
        timestamp
        cost
        tokens
      }
    }

    topCostDrivers(limit: 10) {
      dimension
      value
      cost
      trend
    }
  }
}
```

**React Component:**
```typescript
import { useQuery, gql } from '@apollo/client';
import { LineChart, PieChart } from 'recharts';

const DASHBOARD_QUERY = gql`...`;

function CustomDashboard() {
  const { data, loading } = useQuery(DASHBOARD_QUERY, {
    variables: {
      projectId: 'my-project',
      startDate: startOfWeek(new Date()),
      endDate: new Date()
    },
    pollInterval: 5000  // Real-time updates
  });

  if (loading) return <Loader />;

  return (
    <div className="dashboard">
      <CostSummary data={data.project.costs} />
      <LineChart data={data.project.costs.timeSeries} />
      <PieChart data={data.project.costs.byModel} />
      <TopDrivers data={data.project.topCostDrivers} />
    </div>
  );
}
```

**Highlight**: "GraphQL API • Real-time updates • Custom components"

---

### Scene 4: Data Export & Reporting
**Duration**: 9:00-12:00

**Visual**: Export interfaces and automated report setup

**Code/Demo**:
```typescript
// Programmatic data export
import { CostTracker } from 'llm-cost-ops';

const tracker = new CostTracker({
  apiKey: process.env.LCOPS_API_KEY!
});

// Export to CSV
const csvData = await tracker.exportCosts({
  format: 'csv',
  startDate: new Date('2025-01-01'),
  endDate: new Date('2025-01-31'),
  groupBy: ['date', 'model', 'feature_tag']
});

await writeFile('costs-january.csv', csvData);

// Export to JSON for custom processing
const jsonData = await tracker.exportCosts({
  format: 'json',
  filters: {
    models: ['gpt-4', 'claude-3-opus'],
    minCost: 0.10
  }
});

// Generate PDF report
const report = await tracker.generateReport({
  template: 'executive-summary',
  period: 'monthly',
  includeCharts: true,
  recipients: ['finance@company.com']
});
```

**Scheduled Reports:**
```typescript
// Set up automated weekly reports
await tracker.scheduleReport({
  name: 'Weekly Cost Summary',
  schedule: '0 9 * * 1',  // Every Monday at 9 AM
  template: 'weekly-summary',
  recipients: ['team@company.com'],
  filters: {
    projects: ['production']
  }
});
```

**Highlight**: "CSV/JSON export • PDF reports • Automated scheduling"

---

### Scene 5: BI Tool Integration
**Duration**: 12:00-15:00

**Visual**: Tableau/Looker connection setup

**Code/Demo**:

**PostgreSQL Direct Connection:**
```sql
-- Connect BI tool directly to database
-- Read-only user for security

SELECT
  DATE(timestamp) as date,
  model,
  provider,
  SUM(input_tokens + output_tokens) as total_tokens,
  SUM(cost) as total_cost,
  COUNT(*) as request_count
FROM tracked_requests
WHERE project_id = 'your-project-id'
  AND timestamp >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY DATE(timestamp), model, provider
ORDER BY date DESC;
```

**REST API Integration:**
```javascript
// Tableau Web Data Connector
(function() {
  var myConnector = tableau.makeConnector();

  myConnector.getSchema = function(schemaCallback) {
    var cols = [
      { id: "date", dataType: tableau.dataTypeEnum.date },
      { id: "model", dataType: tableau.dataTypeEnum.string },
      { id: "cost", dataType: tableau.dataTypeEnum.float },
      { id: "tokens", dataType: tableau.dataTypeEnum.int }
    ];

    var tableSchema = {
      id: "llmCostOps",
      alias: "LLM Cost Ops Data",
      columns: cols
    };

    schemaCallback([tableSchema]);
  };

  myConnector.getData = function(table, doneCallback) {
    fetch('https://your-instance.com/api/costs', {
      headers: { 'Authorization': `Bearer ${apiKey}` }
    })
    .then(r => r.json())
    .then(data => {
      table.appendRows(data.rows);
      doneCallback();
    });
  };

  tableau.registerConnector(myConnector);
})();
```

**Highlight**: "Direct database access • REST API • Pre-built connectors"

---

### Scene 6: Real-Time Monitoring
**Duration**: 15:00-17:30

**Code/Demo**:
```typescript
// WebSocket for real-time updates
import { CostTrackerClient } from 'llm-cost-ops/client';

const client = new CostTrackerClient({
  apiKey: process.env.LCOPS_API_KEY!,
  realtime: true
});

// Subscribe to cost events
client.subscribe('costs', {
  filters: { projects: ['production'] },
  onUpdate: (event) => {
    console.log('New cost event:', event);
    updateDashboard(event);
  },
  onThreshold: (alert) => {
    console.warn('Cost threshold exceeded!', alert);
    notifyTeam(alert);
  }
});

// React component with real-time data
function RealtimeCosts() {
  const [costs, setCosts] = useState([]);

  useEffect(() => {
    const subscription = client.subscribe('costs', {
      onUpdate: (event) => {
        setCosts(prev => [...prev, event]);
      }
    });

    return () => subscription.unsubscribe();
  }, []);

  return (
    <div>
      <h2>Live Costs</h2>
      {costs.map(c => (
        <CostItem key={c.id} cost={c} />
      ))}
    </div>
  );
}
```

**Highlight**: "WebSocket updates • Live dashboards • Real-time alerts"

---

### Scene 7: Best Practices & Recap
**Duration**: 17:30-19:00

**Narration**:
"You now know how to build powerful analytics dashboards! Use the built-in dashboard for quick insights, GraphQL for custom views, and BI tools for deep analysis. Set up automated reports to keep stakeholders informed. Next video: budget management and alerts!"

**On-Screen Text**:
- "Best Practices:"
  - "Start with built-in dashboard"
  - "Use GraphQL for custom needs"
  - "Export regularly for backup"
  - "Connect BI tools for deep analysis"
  - "Automate reporting"
- "Next: Video 06 - Budget Management"

---

## Post-Production Notes

### Chapter Markers
- 0:00 - Introduction
- 2:00 - Advanced Filtering
- 5:00 - Custom Dashboards
- 9:00 - Data Export
- 12:00 - BI Integration
- 15:00 - Real-Time Monitoring
- 17:30 - Best Practices

### Graphics Needed
- Dashboard screenshots
- GraphQL query visualizations
- BI tool integration diagrams
- Real-time update animations

**Script Version**: 1.0.0
**Last Updated**: 2025-11-16
