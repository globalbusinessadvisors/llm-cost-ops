# Lab 3: Budget Management

## Overview

Master budget management and cost control with the LLM Cost Ops platform. Learn to create budgets, configure alerts, monitor consumption, and implement budget forecasting for proactive cost management.

**Estimated Time:** 90-120 minutes

**Difficulty Level:** Intermediate

## Learning Objectives

- Create and manage budgets for teams and projects
- Configure multi-level budget hierarchies
- Set up budget alerts and notifications
- Monitor budget consumption in real-time
- Implement budget forecasting
- Configure notification channels (email, Slack, webhooks)
- Handle budget overruns
- Perform budget variance analysis
- Create budget reports and dashboards

## Prerequisites

- [ ] Completed Lab 1 and Lab 2
- [ ] API server running
- [ ] Sample cost data in database
- [ ] Understanding of basic cost tracking

## Part 1: Creating Budgets

### Step 1.1: Understanding Budget Structure

Budgets in LLM Cost Ops support:
- **Organization-level budgets**: Overall spending limits
- **Project-level budgets**: Per-project allocations
- **Team-level budgets**: Department or team budgets
- **Time-based periods**: Daily, weekly, monthly, quarterly, annual

### Step 1.2: Create Your First Budget (Python)

Create `budget_manager.py`:

```python
#!/usr/bin/env python3
"""
Budget Management System
"""

from datetime import datetime, timedelta
from typing import Dict, List, Optional
import json

from cost_ops_client import CostOpsClient

class BudgetManager:
    """Manage budgets and alerts"""

    def __init__(self, base_url: str = "http://localhost:8080"):
        self.client = CostOpsClient(base_url=base_url)

    def create_budget(
        self,
        name: str,
        limit: float,
        period: str = "monthly",
        organization_id: Optional[str] = None,
        project_id: Optional[str] = None,
        warning_threshold: float = 0.80,
        critical_threshold: float = 0.95
    ) -> Dict:
        """Create a new budget"""

        # Calculate period dates
        start_date = datetime.utcnow()

        if period == "monthly":
            # First day of current month
            start_date = start_date.replace(day=1, hour=0, minute=0, second=0, microsecond=0)
            # Last day of current month
            if start_date.month == 12:
                end_date = start_date.replace(year=start_date.year + 1, month=1, day=1) - timedelta(seconds=1)
            else:
                end_date = start_date.replace(month=start_date.month + 1, day=1) - timedelta(seconds=1)
        elif period == "weekly":
            # Start of week (Monday)
            start_date = start_date - timedelta(days=start_date.weekday())
            start_date = start_date.replace(hour=0, minute=0, second=0, microsecond=0)
            end_date = start_date + timedelta(days=7) - timedelta(seconds=1)
        elif period == "quarterly":
            # Start of quarter
            quarter = (start_date.month - 1) // 3
            start_date = start_date.replace(month=quarter * 3 + 1, day=1, hour=0, minute=0, second=0, microsecond=0)
            # End of quarter
            end_month = (quarter + 1) * 3
            if end_month > 12:
                end_date = start_date.replace(year=start_date.year + 1, month=1, day=1) - timedelta(seconds=1)
            else:
                end_date = start_date.replace(month=end_month + 1, day=1) - timedelta(seconds=1)
        else:  # daily
            start_date = start_date.replace(hour=0, minute=0, second=0, microsecond=0)
            end_date = start_date + timedelta(days=1) - timedelta(seconds=1)

        budget = {
            'name': name,
            'limit': limit,
            'period': period,
            'start_date': start_date.isoformat(),
            'end_date': end_date.isoformat(),
            'warning_threshold': warning_threshold,
            'critical_threshold': critical_threshold,
            'organization_id': organization_id,
            'project_id': project_id,
            'created_at': datetime.utcnow().isoformat()
        }

        print(f"✓ Budget '{name}' created")
        print(f"  Limit: ${limit:,.2f}")
        print(f"  Period: {period}")
        print(f"  Duration: {start_date.strftime('%Y-%m-%d')} to {end_date.strftime('%Y-%m-%d')}")
        print(f"  Warning at: {warning_threshold * 100}%")
        print(f"  Critical at: {critical_threshold * 100}%")

        return budget

    def check_budget_status(self, budget: Dict) -> Dict:
        """Check current budget status"""

        # Get actual costs for the period
        costs = self.client.get_costs(
            start_date=budget['start_date'],
            end_date=budget['end_date'],
            organization_id=budget.get('organization_id'),
            project_id=budget.get('project_id')
        )

        # Calculate total spend
        total_spend = sum(float(cost['total_cost']) for cost in costs)

        # Calculate utilization
        utilization = (total_spend / budget['limit']) if budget['limit'] > 0 else 0

        # Determine status
        if utilization >= budget['critical_threshold']:
            status = 'CRITICAL'
            severity = '🔴'
        elif utilization >= budget['warning_threshold']:
            status = 'WARNING'
            severity = '🟡'
        else:
            status = 'OK'
            severity = '🟢'

        # Calculate remaining budget
        remaining = budget['limit'] - total_spend

        # Calculate days remaining
        start = datetime.fromisoformat(budget['start_date'].replace('Z', '+00:00'))
        end = datetime.fromisoformat(budget['end_date'].replace('Z', '+00:00'))
        total_days = (end - start).days
        elapsed_days = (datetime.utcnow() - start).days
        remaining_days = (end - datetime.utcnow()).days

        # Calculate burn rate
        burn_rate = total_spend / elapsed_days if elapsed_days > 0 else 0
        projected_spend = burn_rate * total_days

        result = {
            'budget': budget,
            'status': status,
            'severity': severity,
            'total_spend': total_spend,
            'remaining_budget': remaining,
            'utilization_percent': utilization * 100,
            'elapsed_days': elapsed_days,
            'remaining_days': remaining_days,
            'total_days': total_days,
            'burn_rate': burn_rate,
            'projected_spend': projected_spend,
            'projected_overrun': max(0, projected_spend - budget['limit'])
        }

        return result

    def display_budget_status(self, status: Dict):
        """Display budget status"""
        budget = status['budget']

        print("\n" + "=" * 80)
        print(f"BUDGET STATUS: {budget['name']}")
        print("=" * 80)

        print(f"\n{status['severity']} Status: {status['status']}")
        print(f"\n💰 Financial Summary:")
        print(f"  Budget Limit:        ${budget['limit']:,.2f}")
        print(f"  Total Spend:         ${status['total_spend']:,.2f}")
        print(f"  Remaining Budget:    ${status['remaining_budget']:,.2f}")
        print(f"  Utilization:         {status['utilization_percent']:.1f}%")

        # Visual progress bar
        utilization = status['utilization_percent']
        bar_length = 50
        filled = int(utilization / 100 * bar_length)
        bar = '█' * filled + '░' * (bar_length - filled)
        print(f"\n  [{bar}] {utilization:.1f}%")

        print(f"\n📅 Time Analysis:")
        print(f"  Period:              {budget['period']}")
        print(f"  Elapsed Days:        {status['elapsed_days']} / {status['total_days']}")
        print(f"  Remaining Days:      {status['remaining_days']}")
        print(f"  Time Utilization:    {status['elapsed_days'] / status['total_days'] * 100:.1f}%")

        print(f"\n📊 Burn Rate Analysis:")
        print(f"  Daily Burn Rate:     ${status['burn_rate']:,.2f}/day")
        print(f"  Projected Spend:     ${status['projected_spend']:,.2f}")

        if status['projected_overrun'] > 0:
            print(f"  ⚠️  Projected Overrun: ${status['projected_overrun']:,.2f}")
            print(f"  Recommended Action:  Reduce daily spend to ${budget['limit'] / status['total_days']:,.2f}/day")
        else:
            remaining_daily = status['remaining_budget'] / status['remaining_days'] if status['remaining_days'] > 0 else 0
            print(f"  ✓ Budget on Track")
            print(f"  Available Daily:     ${remaining_daily:,.2f}/day")

        print("\n" + "=" * 80)

    def create_budget_alert(self, status: Dict) -> Optional[Dict]:
        """Create alert if budget thresholds exceeded"""
        if status['status'] in ['WARNING', 'CRITICAL']:
            alert = {
                'type': 'budget_threshold',
                'severity': status['status'],
                'budget_name': status['budget']['name'],
                'limit': status['budget']['limit'],
                'current_spend': status['total_spend'],
                'utilization': status['utilization_percent'],
                'message': f"Budget '{status['budget']['name']}' is at {status['utilization_percent']:.1f}% utilization",
                'timestamp': datetime.utcnow().isoformat()
            }
            return alert
        return None

    def forecast_budget(self, budget: Dict, days_ahead: int = 30) -> Dict:
        """Forecast budget for future period"""

        # Get historical data
        historical_days = 30
        end_date = datetime.utcnow()
        start_date = end_date - timedelta(days=historical_days)

        costs = self.client.get_costs(
            start_date=start_date.isoformat(),
            end_date=end_date.isoformat(),
            organization_id=budget.get('organization_id'),
            project_id=budget.get('project_id')
        )

        # Calculate daily average
        total_cost = sum(float(cost['total_cost']) for cost in costs)
        daily_avg = total_cost / historical_days if historical_days > 0 else 0

        # Project forward
        forecasted_cost = daily_avg * days_ahead

        # Calculate confidence based on variance
        from statistics import stdev

        # Group by day
        from collections import defaultdict
        daily_costs = defaultdict(float)

        for cost in costs:
            timestamp = datetime.fromisoformat(cost['timestamp'].replace('Z', '+00:00'))
            date_key = timestamp.strftime('%Y-%m-%d')
            daily_costs[date_key] += float(cost['total_cost'])

        daily_values = list(daily_costs.values())
        std_dev = stdev(daily_values) if len(daily_values) > 1 else 0

        # Confidence intervals (95%)
        confidence_margin = 1.96 * std_dev * (days_ahead ** 0.5)

        forecast = {
            'forecast_days': days_ahead,
            'forecasted_cost': forecasted_cost,
            'daily_average': daily_avg,
            'confidence_lower': max(0, forecasted_cost - confidence_margin),
            'confidence_upper': forecasted_cost + confidence_margin,
            'budget_limit': budget['limit'],
            'projected_overrun': max(0, forecasted_cost - budget['limit']),
            'probability_over_budget': self._calculate_overrun_probability(forecasted_cost, budget['limit'], confidence_margin)
        }

        return forecast

    def _calculate_overrun_probability(self, forecast: float, limit: float, margin: float) -> float:
        """Calculate probability of exceeding budget"""
        if margin == 0:
            return 1.0 if forecast > limit else 0.0

        # Simple normal distribution approximation
        z_score = (limit - forecast) / margin
        # Approximate probability (simplified)
        if z_score > 2:
            return 0.025
        elif z_score > 1:
            return 0.16
        elif z_score > 0:
            return 0.50
        elif z_score > -1:
            return 0.84
        elif z_score > -2:
            return 0.975
        else:
            return 0.999

    def display_forecast(self, forecast: Dict):
        """Display budget forecast"""
        print("\n" + "=" * 80)
        print(f"BUDGET FORECAST ({forecast['forecast_days']} days)")
        print("=" * 80)

        print(f"\n📈 Forecast Summary:")
        print(f"  Forecasted Cost:     ${forecast['forecasted_cost']:,.2f}")
        print(f"  Confidence Range:    ${forecast['confidence_lower']:,.2f} - ${forecast['confidence_upper']:,.2f}")
        print(f"  Daily Average:       ${forecast['daily_average']:,.2f}")
        print(f"  Budget Limit:        ${forecast['budget_limit']:,.2f}")

        if forecast['projected_overrun'] > 0:
            print(f"\n⚠️  BUDGET OVERRUN PROJECTED:")
            print(f"  Overrun Amount:      ${forecast['projected_overrun']:,.2f}")
            print(f"  Probability:         {forecast['probability_over_budget'] * 100:.1f}%")
            print(f"\n  Recommended Actions:")
            print(f"  • Reduce daily spend by ${forecast['projected_overrun'] / forecast['forecast_days']:,.2f}/day")
            print(f"  • Review and optimize high-cost operations")
            print(f"  • Consider switching to more cost-effective models")
        else:
            buffer = forecast['budget_limit'] - forecast['forecasted_cost']
            print(f"\n✓ BUDGET ON TRACK:")
            print(f"  Projected Buffer:    ${buffer:,.2f}")
            print(f"  Probability Under:   {(1 - forecast['probability_over_budget']) * 100:.1f}%")

        print("\n" + "=" * 80)


# Example usage
if __name__ == "__main__":
    manager = BudgetManager()

    # Create monthly budget
    print("Creating Monthly Budget...")
    print("=" * 80)

    budget = manager.create_budget(
        name="Engineering Team - January 2025",
        limit=5000.00,
        period="monthly",
        organization_id="org-acme-corp",
        warning_threshold=0.75,
        critical_threshold=0.90
    )

    # Check status
    print("\n\nChecking Budget Status...")
    status = manager.check_budget_status(budget)
    manager.display_budget_status(status)

    # Create alert if needed
    alert = manager.create_budget_alert(status)
    if alert:
        print(f"\n🚨 ALERT GENERATED:")
        print(json.dumps(alert, indent=2))

    # Forecast
    print("\n\nGenerating Budget Forecast...")
    forecast = manager.forecast_budget(budget, days_ahead=30)
    manager.display_forecast(forecast)
```

Run the budget manager:

```bash
python budget_manager.py
```

## Part 2: Multi-Level Budget Hierarchies

Create hierarchical budgets for complex organizations:

```python
#!/usr/bin/env python3
"""
Hierarchical Budget Management
"""

from typing import Dict, List
from budget_manager import BudgetManager

class HierarchicalBudgetManager(BudgetManager):
    """Manage hierarchical budgets"""

    def __init__(self, base_url: str = "http://localhost:8080"):
        super().__init__(base_url)
        self.budgets = {}

    def create_budget_hierarchy(self):
        """Create organization -> project -> team budget hierarchy"""

        # Level 1: Organization budget
        org_budget = self.create_budget(
            name="Acme Corp - Monthly",
            limit=50000.00,
            period="monthly",
            organization_id="org-acme-corp"
        )
        self.budgets['organization'] = org_budget

        # Level 2: Department budgets
        eng_budget = self.create_budget(
            name="Engineering Department",
            limit=30000.00,
            period="monthly",
            organization_id="org-acme-corp",
            project_id="dept-engineering"
        )
        self.budgets['dept_engineering'] = eng_budget

        sales_budget = self.create_budget(
            name="Sales Department",
            limit=15000.00,
            period="monthly",
            organization_id="org-acme-corp",
            project_id="dept-sales"
        )
        self.budgets['dept_sales'] = sales_budget

        # Level 3: Team budgets
        ml_budget = self.create_budget(
            name="ML Team",
            limit=15000.00,
            period="monthly",
            organization_id="org-acme-corp",
            project_id="team-ml"
        )
        self.budgets['team_ml'] = ml_budget

        backend_budget = self.create_budget(
            name="Backend Team",
            limit=10000.00,
            period="monthly",
            organization_id="org-acme-corp",
            project_id="team-backend"
        )
        self.budgets['team_backend'] = backend_budget

        print("\n✓ Budget hierarchy created:")
        print("  Organization ($50,000)")
        print("    └─ Engineering Dept ($30,000)")
        print("        ├─ ML Team ($15,000)")
        print("        └─ Backend Team ($10,000)")
        print("    └─ Sales Dept ($15,000)")

    def check_all_budgets(self):
        """Check status of all budgets in hierarchy"""
        print("\n" + "=" * 80)
        print("HIERARCHICAL BUDGET STATUS")
        print("=" * 80)

        for name, budget in self.budgets.items():
            status = self.check_budget_status(budget)
            print(f"\n{status['severity']} {budget['name']}")
            print(f"  Spend: ${status['total_spend']:,.2f} / ${budget['limit']:,.2f}")
            print(f"  Utilization: {status['utilization_percent']:.1f}%")

    def detect_budget_conflicts(self):
        """Detect conflicting budget allocations"""
        print("\n" + "=" * 80)
        print("BUDGET CONFLICT ANALYSIS")
        print("=" * 80)

        # Check if sub-budgets exceed parent budget
        org_limit = self.budgets['organization']['limit']
        dept_total = self.budgets['dept_engineering']['limit'] + self.budgets['dept_sales']['limit']

        print(f"\nOrganization Limit:    ${org_limit:,.2f}")
        print(f"Department Total:      ${dept_total:,.2f}")

        if dept_total > org_limit:
            print(f"⚠️  CONFLICT: Departments allocated ${dept_total - org_limit:,.2f} more than organization limit")
        else:
            print(f"✓ No conflicts: ${org_limit - dept_total:,.2f} unallocated")


if __name__ == "__main__":
    manager = HierarchicalBudgetManager()
    manager.create_budget_hierarchy()
    manager.check_all_budgets()
    manager.detect_budget_conflicts()
```

## Part 3: Budget Alerts and Notifications

Configure notification channels:

```python
#!/usr/bin/env python3
"""
Budget Alert System
"""

import json
import smtplib
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
from typing import Dict, List
import requests

class AlertNotifier:
    """Send budget alerts through various channels"""

    def send_email_alert(self, alert: Dict, to_email: str, smtp_config: Dict):
        """Send email notification"""
        subject = f"Budget Alert: {alert['severity']} - {alert['budget_name']}"

        body = f"""
        Budget Alert

        Severity: {alert['severity']}
        Budget: {alert['budget_name']}
        Limit: ${alert['limit']:,.2f}
        Current Spend: ${alert['current_spend']:,.2f}
        Utilization: {alert['utilization']:.1f}%

        Message: {alert['message']}

        Timestamp: {alert['timestamp']}

        Please review your spending and take appropriate action.
        """

        msg = MIMEMultipart()
        msg['From'] = smtp_config['from_email']
        msg['To'] = to_email
        msg['Subject'] = subject
        msg.attach(MIMEText(body, 'plain'))

        try:
            with smtplib.SMTP(smtp_config['host'], smtp_config['port']) as server:
                if smtp_config.get('use_tls'):
                    server.starttls()
                if smtp_config.get('username'):
                    server.login(smtp_config['username'], smtp_config['password'])
                server.send_message(msg)

            print(f"✓ Email alert sent to {to_email}")
        except Exception as e:
            print(f"✗ Failed to send email: {e}")

    def send_slack_alert(self, alert: Dict, webhook_url: str):
        """Send Slack notification"""
        severity_emoji = {
            'WARNING': ':warning:',
            'CRITICAL': ':rotating_light:'
        }

        emoji = severity_emoji.get(alert['severity'], ':information_source:')

        payload = {
            'text': f"{emoji} Budget Alert",
            'blocks': [
                {
                    'type': 'header',
                    'text': {
                        'type': 'plain_text',
                        'text': f"{emoji} Budget Alert: {alert['severity']}"
                    }
                },
                {
                    'type': 'section',
                    'fields': [
                        {
                            'type': 'mrkdwn',
                            'text': f"*Budget:*\n{alert['budget_name']}"
                        },
                        {
                            'type': 'mrkdwn',
                            'text': f"*Utilization:*\n{alert['utilization']:.1f}%"
                        },
                        {
                            'type': 'mrkdwn',
                            'text': f"*Limit:*\n${alert['limit']:,.2f}"
                        },
                        {
                            'type': 'mrkdwn',
                            'text': f"*Current Spend:*\n${alert['current_spend']:,.2f}"
                        }
                    ]
                },
                {
                    'type': 'section',
                    'text': {
                        'type': 'mrkdwn',
                        'text': alert['message']
                    }
                }
            ]
        }

        try:
            response = requests.post(webhook_url, json=payload)
            response.raise_for_status()
            print("✓ Slack alert sent")
        except Exception as e:
            print(f"✗ Failed to send Slack alert: {e}")

    def send_webhook_alert(self, alert: Dict, webhook_url: str):
        """Send generic webhook notification"""
        try:
            response = requests.post(webhook_url, json=alert)
            response.raise_for_status()
            print(f"✓ Webhook alert sent to {webhook_url}")
        except Exception as e:
            print(f"✗ Failed to send webhook: {e}")

    def send_pagerduty_alert(self, alert: Dict, integration_key: str):
        """Send PagerDuty incident"""
        payload = {
            'routing_key': integration_key,
            'event_action': 'trigger',
            'payload': {
                'summary': f"Budget {alert['severity']}: {alert['budget_name']}",
                'severity': 'critical' if alert['severity'] == 'CRITICAL' else 'warning',
                'source': 'llm-cost-ops',
                'custom_details': alert
            }
        }

        try:
            response = requests.post(
                'https://events.pagerduty.com/v2/enqueue',
                json=payload
            )
            response.raise_for_status()
            print("✓ PagerDuty alert sent")
        except Exception as e:
            print(f"✗ Failed to send PagerDuty alert: {e}")


# Example usage
if __name__ == "__main__":
    notifier = AlertNotifier()

    # Sample alert
    alert = {
        'type': 'budget_threshold',
        'severity': 'WARNING',
        'budget_name': 'Engineering Team - January 2025',
        'limit': 5000.00,
        'current_spend': 4200.00,
        'utilization': 84.0,
        'message': 'Budget is at 84% utilization',
        'timestamp': '2025-01-15T14:30:00Z'
    }

    # Example configurations (use environment variables in production)
    # notifier.send_email_alert(alert, 'admin@example.com', {
    #     'host': 'smtp.gmail.com',
    #     'port': 587,
    #     'use_tls': True,
    #     'from_email': 'alerts@example.com',
    #     'username': 'alerts@example.com',
    #     'password': 'your-password'
    # })

    # notifier.send_slack_alert(alert, 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL')

    # notifier.send_webhook_alert(alert, 'https://your-webhook-endpoint.com/alerts')

    print("\nAlert notification examples completed")
```

## Part 4: Budget Variance Analysis

Analyze budget vs actual spending:

```python
#!/usr/bin/env python3
"""
Budget Variance Analysis
"""

from datetime import datetime, timedelta
from typing import Dict, List
from budget_manager import BudgetManager

class VarianceAnalyzer(BudgetManager):
    """Analyze budget variances"""

    def analyze_variance(self, budget: Dict) -> Dict:
        """Perform variance analysis"""

        status = self.check_budget_status(budget)

        # Calculate time-based expected spend
        elapsed_percent = status['elapsed_days'] / status['total_days'] if status['total_days'] > 0 else 0
        expected_spend = budget['limit'] * elapsed_percent

        # Variance calculations
        absolute_variance = status['total_spend'] - expected_spend
        variance_percent = (absolute_variance / expected_spend * 100) if expected_spend > 0 else 0

        # Categorize variance
        if abs(variance_percent) < 5:
            variance_category = 'On Track'
            variance_icon = '✓'
        elif variance_percent > 0:
            variance_category = 'Over Budget' if variance_percent > 15 else 'Slightly Over'
            variance_icon = '⚠️' if variance_percent > 15 else '⚡'
        else:
            variance_category = 'Under Budget'
            variance_icon = '💰'

        return {
            'budget': budget,
            'expected_spend': expected_spend,
            'actual_spend': status['total_spend'],
            'absolute_variance': absolute_variance,
            'variance_percent': variance_percent,
            'variance_category': variance_category,
            'variance_icon': variance_icon,
            'elapsed_percent': elapsed_percent * 100
        }

    def display_variance_report(self, variance: Dict):
        """Display variance analysis"""
        print("\n" + "=" * 80)
        print("BUDGET VARIANCE ANALYSIS")
        print("=" * 80)

        print(f"\n{variance['variance_icon']} Status: {variance['variance_category']}")

        print(f"\n📊 Variance Summary:")
        print(f"  Budget Name:         {variance['budget']['name']}")
        print(f"  Time Elapsed:        {variance['elapsed_percent']:.1f}%")
        print(f"  Expected Spend:      ${variance['expected_spend']:,.2f}")
        print(f"  Actual Spend:        ${variance['actual_spend']:,.2f}")
        print(f"  Variance:            ${variance['absolute_variance']:+,.2f} ({variance['variance_percent']:+.1f}%)")

        print(f"\n📈 Analysis:")
        if variance['absolute_variance'] > 0:
            print(f"  Spending is ${variance['absolute_variance']:,.2f} above expected rate")
            print(f"  Running {variance['variance_percent']:.1f}% over budget")
        elif variance['absolute_variance'] < 0:
            print(f"  Spending is ${abs(variance['absolute_variance']):,.2f} below expected rate")
            print(f"  Running {abs(variance['variance_percent']):.1f}% under budget")
        else:
            print(f"  Spending is exactly on budget")

        print("\n" + "=" * 80)


if __name__ == "__main__":
    analyzer = VarianceAnalyzer()

    # Create sample budget
    budget = analyzer.create_budget(
        name="Q1 2025 Budget",
        limit=15000.00,
        period="quarterly"
    )

    # Analyze variance
    variance = analyzer.analyze_variance(budget)
    analyzer.display_variance_report(variance)
```

## Exercises and Challenges

### Exercise 1: Budget Dashboard
Create a comprehensive budget dashboard showing all active budgets, their status, and alerts.

### Exercise 2: Budget Optimization
Implement a budget recommendation system that suggests optimal budget allocations based on historical data.

### Exercise 3: Multi-Currency Budgets
Extend the budget system to support multiple currencies with automatic conversion.

### Exercise 4: Budget Approval Workflow
Implement a budget approval workflow with multiple approval levels.

### Exercise 5: Budget Rollover
Create functionality to automatically roll over unused budget to the next period.

## Review Questions

1. What are the key components of a budget in LLM Cost Ops?
2. How do warning and critical thresholds help in budget management?
3. What is burn rate and why is it important?
4. How can variance analysis help identify budget issues?
5. What notification channels are most appropriate for critical budget alerts?

## Next Steps

Continue to **Lab 4: Cost Optimization** to learn about:
- Identifying optimization opportunities
- Model selection strategies
- Caching strategies
- Cost anomaly detection

---

**End of Lab 3**
