# Video 06: Budget Management & Alerts

## Metadata

- **Duration**: 16-20 minutes
- **Level**: Intermediate
- **Prerequisites**: Videos 01, 02, 05
- **Target Audience**: Finance teams, operations, team leads
- **Video ID**: LLMCO-V06-BUDGETS
- **Version**: 1.0.0

## Learning Objectives

- Set up project and team budgets
- Configure multi-level budget hierarchies
- Create intelligent alerts and notifications
- Implement automatic cost controls
- Use forecasting to predict future costs
- Set up budget workflows and approvals
- Integrate with financial systems

## Scene Breakdown

### Scene 1: Budget Basics
**Duration**: 0:00-3:00

**Visual**: Budget creation interface

**Narration**:
"Budget management keeps your AI costs under control. Today we'll set up budgets, configure alerts, implement automatic controls, and use forecasting to predict future spending. Let's start with creating your first budget."

**Code/Demo**:
```typescript
import { CostTracker } from 'llm-cost-ops';

const tracker = new CostTracker({ apiKey: process.env.LCOPS_API_KEY! });

// Create project budget
const budget = await tracker.createBudget({
  name: 'Q1 2025 Production',
  amount: 10000,  // $10,000
  period: 'monthly',
  scope: {
    projects: ['production'],
    environments: ['prod']
  },
  alerts: [
    { threshold: 0.5, channels: ['email'] },      // 50%
    { threshold: 0.8, channels: ['email', 'slack'] },  // 80%
    { threshold: 0.95, channels: ['email', 'slack', 'pagerduty'] }  // 95%
  ]
});
```

**Highlight**: "Flexible budget scopes • Multi-threshold alerts • Multiple notification channels"

---

### Scene 2: Budget Hierarchies
**Duration**: 3:00-6:30

**Visual**: Org chart showing budget hierarchy

**Code/Demo**:
```typescript
// Company-wide budget
const companyBudget = await tracker.createBudget({
  name: 'Company AI Budget 2025',
  amount: 100000,  // $100k total
  period: 'monthly',
  scope: 'company-wide'
});

// Department budgets (children of company budget)
const engineeringBudget = await tracker.createBudget({
  name: 'Engineering AI Budget',
  amount: 60000,
  period: 'monthly',
  parentBudget: companyBudget.id,
  scope: {
    departments: ['engineering']
  }
});

const productBudget = await tracker.createBudget({
  name: 'Product AI Budget',
  amount: 30000,
  period: 'monthly',
  parentBudget: companyBudget.id,
  scope: {
    departments: ['product']
  }
});

// Team-level budgets
const backendTeamBudget = await tracker.createBudget({
  name: 'Backend Team Budget',
  amount: 30000,
  period: 'monthly',
  parentBudget: engineeringBudget.id,
  scope: {
    teams: ['backend'],
    tags: { team: 'backend' }
  }
});

// Budget rollup automatically tracks:
// - Individual team spending
// - Department totals
// - Company-wide totals
// - Alerts at each level
```

**Highlight**: "Hierarchical budgets • Automatic rollups • Department allocation"

---

### Scene 3: Smart Alerts & Notifications
**Duration**: 6:30-10:00

**Visual**: Alert configuration and notification examples

**Code/Demo**:
```typescript
// Advanced alert configuration
const alertConfig = {
  budgetId: budget.id,

  // Multiple alert types
  alerts: [
    // Threshold alerts
    {
      type: 'threshold',
      threshold: 0.8,  // 80% of budget
      channels: ['slack', 'email'],
      message: '⚠️ 80% of monthly budget consumed',
      recipients: ['finance@company.com', 'engineering-leads@company.com']
    },

    // Velocity alerts (spending rate)
    {
      type: 'velocity',
      condition: 'spending_rate > projected_rate * 1.2',  // 20% faster than projected
      channels: ['slack'],
      message: '🚨 Spending 20% faster than projected!',
      cooldown: 3600  // Don't spam - max 1 per hour
    },

    // Anomaly detection
    {
      type: 'anomaly',
      sensitivity: 'medium',
      condition: 'cost > mean + 2*stddev',  // 2 standard deviations
      channels: ['slack', 'pagerduty'],
      message: '📊 Unusual spending pattern detected'
    },

    // Cost spike alerts
    {
      type: 'spike',
      threshold: 100,  // $100 in single hour
      window: 'hourly',
      channels: ['slack'],
      message: '💥 Cost spike detected: $100+ in 1 hour'
    }
  ],

  // Escalation policy
  escalation: {
    enabled: true,
    levels: [
      {
        delay: 0,
        recipients: ['team-lead@company.com'],
        channels: ['email']
      },
      {
        delay: 1800,  // 30 minutes later if not acknowledged
        recipients: ['engineering-manager@company.com'],
        channels: ['email', 'sms']
      },
      {
        delay: 3600,  // 1 hour later
        recipients: ['cto@company.com'],
        channels: ['email', 'sms', 'phone']
      }
    ]
  }
};

await tracker.configureAlerts(alertConfig);
```

**Slack Notification Setup:**
```typescript
// Slack webhook integration
await tracker.addNotificationChannel({
  type: 'slack',
  name: 'Engineering Alerts',
  config: {
    webhookUrl: process.env.SLACK_WEBHOOK_URL,
    channel: '#ai-costs',
    mentions: ['@engineering-leads'],
    template: {
      text: '{{alert.message}}',
      attachments: [{
        color: '{{alert.severity_color}}',
        fields: [
          { title: 'Current Spend', value: '{{budget.current}}', short: true },
          { title: 'Budget Limit', value: '{{budget.limit}}', short: true },
          { title: 'Percentage', value: '{{budget.percentage}}%', short: true },
          { title: 'Forecast', value: '{{budget.forecast}}', short: true }
        ]
      }]
    }
  }
});
```

**Highlight**: "Multiple alert types • Anomaly detection • Escalation policies"

---

### Scene 4: Automatic Cost Controls
**Duration**: 10:00-13:00

**Visual**: Cost control rules and enforcement

**Code/Demo**:
```typescript
// Automatic cost control policies
await tracker.configureCostControls({
  budgetId: budget.id,

  rules: [
    // Auto-downgrade to cheaper model at 90% budget
    {
      trigger: 'budget_percentage >= 0.9',
      action: 'model_downgrade',
      config: {
        downgrades: {
          'gpt-4': 'gpt-3.5-turbo',
          'claude-3-opus': 'claude-3-sonnet'
        }
      },
      notify: true
    },

    // Block expensive models in development
    {
      trigger: 'environment == "development"',
      action: 'model_blocklist',
      config: {
        blockedModels: ['gpt-4', 'claude-3-opus'],
        allowedModels: ['gpt-3.5-turbo', 'claude-3-haiku']
      }
    },

    // Rate limiting at 95% budget
    {
      trigger: 'budget_percentage >= 0.95',
      action: 'rate_limit',
      config: {
        maxRequestsPerMinute: 10,
        maxCostPerHour: 50
      }
    },

    // Hard stop at 100%
    {
      trigger: 'budget_percentage >= 1.0',
      action: 'block',
      config: {
        message: 'Monthly budget exceeded. Contact finance team.',
        exemptUsers: ['admin@company.com']
      }
    }
  ]
});

// SDK automatically enforces policies
try {
  const response = await tracker.track(
    openai.chat.completions.create({
      model: 'gpt-4',  // May be auto-downgraded or blocked
      messages: [...]
    })
  );
} catch (BudgetExceededError) {
  // Handle budget exceeded
  console.log('Budget exceeded, using fallback response');
}
```

**Highlight**: "Auto model downgrade • Rate limiting • Hard budget stops"

---

### Scene 5: Forecasting & Predictions
**Duration**: 13:00-16:00

**Visual**: Forecast graphs and projection visualizations

**Code/Demo**:
```typescript
// Get cost forecast
const forecast = await tracker.getForecast({
  projectId: 'production',
  forecastPeriod: 'month',
  confidence: 0.95
});

console.log('Forecast Results:');
console.log('- Projected spend:', forecast.projected);
console.log('- Confidence interval:', forecast.confidenceInterval);
console.log('- Days until budget exceeded:', forecast.daysUntilExceeded);
console.log('- Recommended budget:', forecast.recommendedBudget);

// Forecast with different scenarios
const scenarios = await tracker.forecastScenarios({
  projectId: 'production',
  scenarios: [
    {
      name: 'Current Trajectory',
      assumptions: 'current_rate'
    },
    {
      name: 'With New Feature',
      assumptions: {
        additionalRequests: 10000,
        averageCost: 0.05
      }
    },
    {
      name: 'After Optimization',
      assumptions: {
        costReduction: 0.3  // 30% reduction
      }
    }
  ]
});

// Visualize scenarios
scenarios.forEach(scenario => {
  console.log(`${scenario.name}: $${scenario.projectedCost}`);
});
```

**React Component for Forecast Visualization:**
```typescript
import { LineChart, Line, Area, XAxis, YAxis } from 'recharts';

function ForecastChart({ forecast }) {
  return (
    <LineChart data={forecast.daily}>
      <XAxis dataKey="date" />
      <YAxis />

      {/* Actual spending */}
      <Line
        type="monotone"
        dataKey="actual"
        stroke="#3b82f6"
        strokeWidth={2}
      />

      {/* Projected spending */}
      <Line
        type="monotone"
        dataKey="projected"
        stroke="#8b5cf6"
        strokeDasharray="5 5"
      />

      {/* Confidence interval */}
      <Area
        type="monotone"
        dataKey="confidenceLow"
        stroke="none"
        fill="#8b5cf6"
        fillOpacity={0.1}
      />
      <Area
        type="monotone"
        dataKey="confidenceHigh"
        stroke="none"
        fill="#8b5cf6"
        fillOpacity={0.1}
      />

      {/* Budget limit line */}
      <Line
        type="monotone"
        dataKey="budgetLimit"
        stroke="#ef4444"
        strokeDasharray="3 3"
      />
    </LineChart>
  );
}
```

**Highlight**: "AI-powered forecasting • Scenario planning • Budget recommendations"

---

### Scene 6: Budget Workflows & Approvals
**Duration**: 16:00-18:00

**Code/Demo**:
```typescript
// Budget request workflow
const budgetRequest = await tracker.createBudgetRequest({
  requester: 'product-manager@company.com',
  amount: 5000,
  duration: 'monthly',
  justification: 'New AI feature launch - customer demand tool',
  approvers: [
    'engineering-manager@company.com',
    'finance-director@company.com'
  ],
  metadata: {
    project: 'customer-insights',
    expectedROI: '3x',
    businessCase: 'link-to-doc'
  }
});

// Approval flow
await tracker.approveBudget({
  requestId: budgetRequest.id,
  approver: 'engineering-manager@company.com',
  decision: 'approved',
  comments: 'Approved - aligns with Q1 OKRs'
});

// Auto-create budget after all approvals
// Sends notifications to requester
```

**Highlight**: "Approval workflows • Budget requests • Governance"

---

### Scene 7: Recap & Best Practices
**Duration**: 18:00-19:00

**Narration**:
"You're now a budget management expert! Set up hierarchical budgets, configure smart alerts, enable automatic controls, and use forecasting to stay ahead. Next video: advanced cost optimization techniques!"

**On-Screen Text**:
- "Best Practices:"
  - "Start with conservative budgets"
  - "Use multi-level alerts"
  - "Enable gradual controls"
  - "Review forecasts weekly"
  - "Adjust budgets quarterly"
- "Next: Video 07 - Cost Optimization"

---

## Post-Production Notes

### Chapter Markers
- 0:00 - Budget Basics
- 3:00 - Budget Hierarchies
- 6:30 - Smart Alerts
- 10:00 - Automatic Controls
- 13:00 - Forecasting
- 16:00 - Workflows
- 18:00 - Recap

**Script Version**: 1.0.0
**Last Updated**: 2025-11-16
