# Lab 2: Analytics and Reporting

## Overview

In this hands-on lab, you'll learn to build comprehensive analytics dashboards and automated reporting systems using the LLM Cost Ops platform. You'll create custom visualizations, perform time-series analysis, generate comparative reports, and set up scheduled automated reporting.

**Estimated Time:** 90-120 minutes

**Difficulty Level:** Intermediate

## Learning Objectives

By the end of this lab, you will be able to:

- Build custom cost analytics dashboards
- Perform time-series analysis on cost data
- Create comparative analyses (month-over-month, year-over-year)
- Generate charts and data visualizations
- Configure scheduled automated reports
- Set up multi-channel report delivery (email, webhook, S3)
- Export data in multiple formats (CSV, JSON, Excel)
- Create executive summary reports
- Build trend analysis tools
- Implement cost forecasting visualizations

## Prerequisites

Before starting this lab, ensure you have:

- [ ] Completed Lab 1: Basic Cost Tracking
- [ ] API server running on localhost:8080
- [ ] Database populated with sample usage data
- [ ] Python 3.8+ OR Node.js 16+ environment set up
- [ ] Basic understanding of data visualization concepts
- [ ] Familiarity with JSON and CSV formats

### Additional Dependencies

**For Python users:**
```bash
pip install matplotlib pandas seaborn tabulate plotly
```

**For TypeScript users:**
```bash
npm install chart.js d3 date-fns cli-table3 json2csv
```

## Part 1: Custom Dashboard Development

### Step 1.1: Create a Basic Analytics Dashboard

Let's build a comprehensive cost analytics dashboard.

#### For Python Users:

Create `analytics_dashboard.py`:

```python
#!/usr/bin/env python3
"""
Comprehensive Cost Analytics Dashboard
"""

import sys
import os
from datetime import datetime, timedelta
from typing import Dict, List, Any
import json

# Add parent directory to path if needed
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from cost_ops_client import CostOpsClient

class CostAnalyticsDashboard:
    """Interactive cost analytics dashboard"""

    def __init__(self, base_url: str = "http://localhost:8080"):
        self.client = CostOpsClient(base_url=base_url)
        self.end_date = datetime.utcnow()
        self.start_date = self.end_date - timedelta(days=30)

    def display_header(self, title: str):
        """Display formatted header"""
        print("\n" + "=" * 80)
        print(f"  {title}")
        print("=" * 80 + "\n")

    def display_section(self, title: str):
        """Display section header"""
        print(f"\n{title}")
        print("-" * 80)

    def get_overall_summary(self) -> Dict:
        """Get overall cost summary"""
        return self.client.get_summary(
            start_date=self.start_date.isoformat(),
            end_date=self.end_date.isoformat(),
            group_by=['provider', 'model', 'project']
        )

    def display_overview(self, summary: Dict):
        """Display cost overview"""
        self.display_section("📊 COST OVERVIEW")

        total_cost = float(summary.get('total_cost', 0))
        total_requests = summary.get('total_requests', 0)
        avg_cost = float(summary.get('avg_cost_per_request', 0))

        print(f"  Period:              {self.start_date.strftime('%Y-%m-%d')} to {self.end_date.strftime('%Y-%m-%d')}")
        print(f"  Total Cost:          ${total_cost:,.6f}")
        print(f"  Total Requests:      {total_requests:,}")
        print(f"  Avg Cost/Request:    ${avg_cost:.6f}")
        print(f"  Requests/Day:        {total_requests / 30:.1f}")
        print(f"  Cost/Day:            ${total_cost / 30:.6f}")

    def display_provider_breakdown(self, summary: Dict):
        """Display provider-wise cost breakdown"""
        self.display_section("🏢 BREAKDOWN BY PROVIDER")

        by_provider = summary.get('by_provider', {})
        if not by_provider:
            print("  No provider data available")
            return

        # Sort by cost (descending)
        sorted_providers = sorted(
            by_provider.items(),
            key=lambda x: float(x[1]),
            reverse=True
        )

        total = sum(float(cost) for _, cost in sorted_providers)

        print(f"  {'Provider':<20} {'Cost':<15} {'Percentage':<12} {'Visualization'}")
        print(f"  {'-'*20} {'-'*15} {'-'*12} {'-'*30}")

        for provider, cost in sorted_providers:
            cost_float = float(cost)
            percentage = (cost_float / total * 100) if total > 0 else 0
            bar_length = int(percentage / 2)  # Scale to 50 chars max
            bar = '█' * bar_length

            print(f"  {provider:<20} ${cost_float:<14.6f} {percentage:>6.2f}%     {bar}")

    def display_model_breakdown(self, summary: Dict):
        """Display model-wise cost breakdown"""
        self.display_section("🤖 BREAKDOWN BY MODEL")

        by_model = summary.get('by_model', {})
        if not by_model:
            print("  No model data available")
            return

        # Sort by cost (descending)
        sorted_models = sorted(
            by_model.items(),
            key=lambda x: float(x[1]),
            reverse=True
        )

        # Show top 10 models
        top_models = sorted_models[:10]

        print(f"  {'Rank':<6} {'Model':<40} {'Cost':<15}")
        print(f"  {'-'*6} {'-'*40} {'-'*15}")

        for idx, (model, cost) in enumerate(top_models, 1):
            cost_float = float(cost)
            print(f"  {idx:<6} {model:<40} ${cost_float:.6f}")

        if len(sorted_models) > 10:
            remaining_cost = sum(float(cost) for _, cost in sorted_models[10:])
            print(f"  {'...':<6} {'Other models':<40} ${remaining_cost:.6f}")

    def display_project_breakdown(self, summary: Dict):
        """Display project-wise cost breakdown"""
        self.display_section("📁 BREAKDOWN BY PROJECT")

        by_project = summary.get('by_project', {})
        if not by_project:
            print("  No project data available")
            return

        # Sort by cost (descending)
        sorted_projects = sorted(
            by_project.items(),
            key=lambda x: float(x[1]),
            reverse=True
        )

        total = sum(float(cost) for _, cost in sorted_projects)

        print(f"  {'Project':<30} {'Cost':<15} {'Percentage':<12}")
        print(f"  {'-'*30} {'-'*15} {'-'*12}")

        for project, cost in sorted_projects:
            cost_float = float(cost)
            percentage = (cost_float / total * 100) if total > 0 else 0
            print(f"  {project:<30} ${cost_float:<14.6f} {percentage:>6.2f}%")

    def display_cost_efficiency(self, summary: Dict):
        """Display cost efficiency metrics"""
        self.display_section("⚡ COST EFFICIENCY METRICS")

        by_model = summary.get('by_model', {})
        if not by_model:
            print("  No data available for efficiency analysis")
            return

        # Calculate average costs
        model_costs = [(model, float(cost)) for model, cost in by_model.items()]

        if model_costs:
            avg_model_cost = sum(cost for _, cost in model_costs) / len(model_costs)
            min_cost_model = min(model_costs, key=lambda x: x[1])
            max_cost_model = max(model_costs, key=lambda x: x[1])

            print(f"  Average Cost per Model:  ${avg_model_cost:.6f}")
            print(f"  Most Efficient Model:    {min_cost_model[0]} (${min_cost_model[1]:.6f})")
            print(f"  Most Expensive Model:    {max_cost_model[0]} (${max_cost_model[1]:.6f})")
            print(f"  Cost Range:              ${min_cost_model[1]:.6f} - ${max_cost_model[1]:.6f}")

    def display_recommendations(self, summary: Dict):
        """Display cost optimization recommendations"""
        self.display_section("💡 COST OPTIMIZATION RECOMMENDATIONS")

        by_model = summary.get('by_model', {})
        total_cost = float(summary.get('total_cost', 0))

        recommendations = []

        # Check for expensive models
        for model, cost in by_model.items():
            cost_float = float(cost)
            if cost_float > total_cost * 0.5:
                recommendations.append(
                    f"  • Model '{model}' accounts for {cost_float/total_cost*100:.1f}% of total costs. "
                    f"Consider optimizing usage or exploring alternative models."
                )

        # Check for GPT-4 usage
        for model, cost in by_model.items():
            if 'gpt-4' in model.lower() and 'turbo' not in model.lower():
                recommendations.append(
                    f"  • Consider using GPT-4 Turbo instead of GPT-4 for potential cost savings."
                )
                break

        # Check for Claude usage with caching
        has_claude = any('claude' in model.lower() for model in by_model.keys())
        if has_claude:
            recommendations.append(
                f"  • Ensure you're utilizing prompt caching for Claude models to reduce costs by up to 50%."
            )

        if recommendations:
            for rec in recommendations:
                print(rec)
        else:
            print("  ✓ No immediate cost optimization opportunities identified.")
            print("  ✓ Current usage patterns appear efficient.")

    def run(self):
        """Run the complete dashboard"""
        self.display_header("LLM COST ANALYTICS DASHBOARD")

        print(f"  Loading data...")
        summary = self.get_overall_summary()

        self.display_overview(summary)
        self.display_provider_breakdown(summary)
        self.display_model_breakdown(summary)
        self.display_project_breakdown(summary)
        self.display_cost_efficiency(summary)
        self.display_recommendations(summary)

        print("\n" + "=" * 80)
        print(f"  Dashboard generated at {datetime.utcnow().strftime('%Y-%m-%d %H:%M:%S UTC')}")
        print("=" * 80 + "\n")


if __name__ == "__main__":
    dashboard = CostAnalyticsDashboard()
    dashboard.run()
```

Run the dashboard:

```bash
python analytics_dashboard.py
```

#### For TypeScript Users:

Create `analytics-dashboard.ts`:

```typescript
#!/usr/bin/env ts-node
/**
 * Comprehensive Cost Analytics Dashboard
 */

import { CostOpsClient, CostSummary } from './cost-ops-client';

class CostAnalyticsDashboard {
  private client: CostOpsClient;
  private endDate: Date;
  private startDate: Date;

  constructor(baseUrl: string = 'http://localhost:8080') {
    this.client = new CostOpsClient(baseUrl);
    this.endDate = new Date();
    this.startDate = new Date(this.endDate.getTime() - 30 * 24 * 60 * 60 * 1000);
  }

  private displayHeader(title: string): void {
    console.log('\n' + '='.repeat(80));
    console.log(`  ${title}`);
    console.log('='.repeat(80) + '\n');
  }

  private displaySection(title: string): void {
    console.log(`\n${title}`);
    console.log('-'.repeat(80));
  }

  private async getOverallSummary(): Promise<CostSummary> {
    return await this.client.getSummary(
      this.startDate.toISOString(),
      this.endDate.toISOString(),
      undefined,
      ['provider', 'model', 'project']
    );
  }

  private displayOverview(summary: CostSummary): void {
    this.displaySection('📊 COST OVERVIEW');

    const totalCost = parseFloat(summary.total_cost || '0');
    const totalRequests = summary.total_requests || 0;
    const avgCost = parseFloat(summary.avg_cost_per_request || '0');

    console.log(`  Period:              ${this.startDate.toISOString().split('T')[0]} to ${this.endDate.toISOString().split('T')[0]}`);
    console.log(`  Total Cost:          $${totalCost.toFixed(6)}`);
    console.log(`  Total Requests:      ${totalRequests.toLocaleString()}`);
    console.log(`  Avg Cost/Request:    $${avgCost.toFixed(6)}`);
    console.log(`  Requests/Day:        ${(totalRequests / 30).toFixed(1)}`);
    console.log(`  Cost/Day:            $${(totalCost / 30).toFixed(6)}`);
  }

  private displayProviderBreakdown(summary: CostSummary): void {
    this.displaySection('🏢 BREAKDOWN BY PROVIDER');

    const byProvider = summary.by_provider || {};
    if (Object.keys(byProvider).length === 0) {
      console.log('  No provider data available');
      return;
    }

    // Sort by cost (descending)
    const sorted = Object.entries(byProvider)
      .map(([provider, cost]) => ({ provider, cost: parseFloat(cost) }))
      .sort((a, b) => b.cost - a.cost);

    const total = sorted.reduce((sum, item) => sum + item.cost, 0);

    console.log(`  ${'Provider'.padEnd(20)} ${'Cost'.padEnd(15)} ${'Percentage'.padEnd(12)} ${'Visualization'}`);
    console.log(`  ${'-'.repeat(20)} ${'-'.repeat(15)} ${'-'.repeat(12)} ${'-'.repeat(30)}`);

    for (const { provider, cost } of sorted) {
      const percentage = total > 0 ? (cost / total * 100) : 0;
      const barLength = Math.floor(percentage / 2);
      const bar = '█'.repeat(barLength);

      console.log(`  ${provider.padEnd(20)} $${cost.toFixed(6).padEnd(14)} ${percentage.toFixed(2).padStart(6)}%     ${bar}`);
    }
  }

  private displayModelBreakdown(summary: CostSummary): void {
    this.displaySection('🤖 BREAKDOWN BY MODEL');

    const byModel = summary.by_model || {};
    if (Object.keys(byModel).length === 0) {
      console.log('  No model data available');
      return;
    }

    // Sort by cost (descending)
    const sorted = Object.entries(byModel)
      .map(([model, cost]) => ({ model, cost: parseFloat(cost) }))
      .sort((a, b) => b.cost - a.cost);

    // Show top 10
    const top10 = sorted.slice(0, 10);

    console.log(`  ${'Rank'.padEnd(6)} ${'Model'.padEnd(40)} ${'Cost'.padEnd(15)}`);
    console.log(`  ${'-'.repeat(6)} ${'-'.repeat(40)} ${'-'.repeat(15)}`);

    top10.forEach(({ model, cost }, idx) => {
      console.log(`  ${(idx + 1).toString().padEnd(6)} ${model.padEnd(40)} $${cost.toFixed(6)}`);
    });

    if (sorted.length > 10) {
      const remainingCost = sorted.slice(10).reduce((sum, item) => sum + item.cost, 0);
      console.log(`  ${'...'.padEnd(6)} ${'Other models'.padEnd(40)} $${remainingCost.toFixed(6)}`);
    }
  }

  private displayProjectBreakdown(summary: CostSummary): void {
    this.displaySection('📁 BREAKDOWN BY PROJECT');

    const byProject = summary.by_project || {};
    if (Object.keys(byProject).length === 0) {
      console.log('  No project data available');
      return;
    }

    const sorted = Object.entries(byProject)
      .map(([project, cost]) => ({ project, cost: parseFloat(cost) }))
      .sort((a, b) => b.cost - a.cost);

    const total = sorted.reduce((sum, item) => sum + item.cost, 0);

    console.log(`  ${'Project'.padEnd(30)} ${'Cost'.padEnd(15)} ${'Percentage'.padEnd(12)}`);
    console.log(`  ${'-'.repeat(30)} ${'-'.repeat(15)} ${'-'.repeat(12)}`);

    for (const { project, cost } of sorted) {
      const percentage = total > 0 ? (cost / total * 100) : 0;
      console.log(`  ${project.padEnd(30)} $${cost.toFixed(6).padEnd(14)} ${percentage.toFixed(2).padStart(6)}%`);
    }
  }

  private displayCostEfficiency(summary: CostSummary): void {
    this.displaySection('⚡ COST EFFICIENCY METRICS');

    const byModel = summary.by_model || {};
    if (Object.keys(byModel).length === 0) {
      console.log('  No data available for efficiency analysis');
      return;
    }

    const modelCosts = Object.entries(byModel).map(([model, cost]) => ({
      model,
      cost: parseFloat(cost)
    }));

    if (modelCosts.length > 0) {
      const avgCost = modelCosts.reduce((sum, item) => sum + item.cost, 0) / modelCosts.length;
      const minCostModel = modelCosts.reduce((min, item) => item.cost < min.cost ? item : min);
      const maxCostModel = modelCosts.reduce((max, item) => item.cost > max.cost ? item : max);

      console.log(`  Average Cost per Model:  $${avgCost.toFixed(6)}`);
      console.log(`  Most Efficient Model:    ${minCostModel.model} ($${minCostModel.cost.toFixed(6)})`);
      console.log(`  Most Expensive Model:    ${maxCostModel.model} ($${maxCostModel.cost.toFixed(6)})`);
      console.log(`  Cost Range:              $${minCostModel.cost.toFixed(6)} - $${maxCostModel.cost.toFixed(6)}`);
    }
  }

  private displayRecommendations(summary: CostSummary): void {
    this.displaySection('💡 COST OPTIMIZATION RECOMMENDATIONS');

    const byModel = summary.by_model || {};
    const totalCost = parseFloat(summary.total_cost || '0');
    const recommendations: string[] = [];

    // Check for expensive models
    for (const [model, cost] of Object.entries(byModel)) {
      const costFloat = parseFloat(cost);
      if (costFloat > totalCost * 0.5) {
        recommendations.push(
          `  • Model '${model}' accounts for ${(costFloat / totalCost * 100).toFixed(1)}% of total costs. ` +
          `Consider optimizing usage or exploring alternative models.`
        );
      }
    }

    // Check for GPT-4 usage
    for (const model of Object.keys(byModel)) {
      if (model.toLowerCase().includes('gpt-4') && !model.toLowerCase().includes('turbo')) {
        recommendations.push(
          `  • Consider using GPT-4 Turbo instead of GPT-4 for potential cost savings.`
        );
        break;
      }
    }

    // Check for Claude usage
    const hasClaude = Object.keys(byModel).some(model => model.toLowerCase().includes('claude'));
    if (hasClaude) {
      recommendations.push(
        `  • Ensure you're utilizing prompt caching for Claude models to reduce costs by up to 50%.`
      );
    }

    if (recommendations.length > 0) {
      recommendations.forEach(rec => console.log(rec));
    } else {
      console.log('  ✓ No immediate cost optimization opportunities identified.');
      console.log('  ✓ Current usage patterns appear efficient.');
    }
  }

  async run(): Promise<void> {
    this.displayHeader('LLM COST ANALYTICS DASHBOARD');

    console.log('  Loading data...');
    const summary = await this.getOverallSummary();

    this.displayOverview(summary);
    this.displayProviderBreakdown(summary);
    this.displayModelBreakdown(summary);
    this.displayProjectBreakdown(summary);
    this.displayCostEfficiency(summary);
    this.displayRecommendations(summary);

    console.log('\n' + '='.repeat(80));
    console.log(`  Dashboard generated at ${new Date().toISOString()}`);
    console.log('='.repeat(80) + '\n');
  }
}

// Run dashboard
const dashboard = new CostAnalyticsDashboard();
dashboard.run().catch(console.error);
```

Run the dashboard:

```bash
npx ts-node analytics-dashboard.ts
```

## Part 2: Time-Series Analysis

Create time-series analysis tools to understand cost trends over time.

### For Python Users:

Create `time_series_analysis.py`:

```python
#!/usr/bin/env python3
"""
Time-Series Cost Analysis
"""

from datetime import datetime, timedelta
from typing import Dict, List, Tuple
from collections import defaultdict
import json

from cost_ops_client import CostOpsClient

class TimeSeriesAnalyzer:
    """Analyze costs over time"""

    def __init__(self, base_url: str = "http://localhost:8080"):
        self.client = CostOpsClient(base_url=base_url)

    def get_daily_costs(self, days: int = 30) -> List[Tuple[str, float]]:
        """Get daily cost totals"""
        end_date = datetime.utcnow()
        start_date = end_date - timedelta(days=days)

        # Get all costs for the period
        costs = self.client.get_costs(
            start_date=start_date.isoformat(),
            end_date=end_date.isoformat()
        )

        # Group by day
        daily_costs = defaultdict(float)

        for cost in costs:
            # Parse timestamp and extract date
            timestamp = datetime.fromisoformat(cost['timestamp'].replace('Z', '+00:00'))
            date_key = timestamp.strftime('%Y-%m-%d')
            daily_costs[date_key] += float(cost['total_cost'])

        # Sort by date
        sorted_costs = sorted(daily_costs.items())

        return sorted_costs

    def calculate_moving_average(self, daily_costs: List[Tuple[str, float]], window: int = 7) -> List[Tuple[str, float]]:
        """Calculate moving average"""
        if len(daily_costs) < window:
            return daily_costs

        moving_avg = []

        for i in range(len(daily_costs) - window + 1):
            window_costs = [cost for _, cost in daily_costs[i:i+window]]
            avg = sum(window_costs) / window
            date = daily_costs[i + window - 1][0]
            moving_avg.append((date, avg))

        return moving_avg

    def calculate_trend(self, daily_costs: List[Tuple[str, float]]) -> Dict:
        """Calculate cost trend"""
        if len(daily_costs) < 2:
            return {'direction': 'insufficient_data', 'change': 0}

        # Compare first half vs second half
        mid_point = len(daily_costs) // 2
        first_half = [cost for _, cost in daily_costs[:mid_point]]
        second_half = [cost for _, cost in daily_costs[mid_point:]]

        first_avg = sum(first_half) / len(first_half) if first_half else 0
        second_avg = sum(second_half) / len(second_half) if second_half else 0

        if second_avg > first_avg * 1.1:
            direction = 'increasing'
        elif second_avg < first_avg * 0.9:
            direction = 'decreasing'
        else:
            direction = 'stable'

        change_percent = ((second_avg - first_avg) / first_avg * 100) if first_avg > 0 else 0

        return {
            'direction': direction,
            'change_percent': change_percent,
            'first_half_avg': first_avg,
            'second_half_avg': second_avg
        }

    def display_time_series(self, days: int = 30):
        """Display time-series analysis"""
        print("=" * 80)
        print("TIME-SERIES COST ANALYSIS")
        print("=" * 80 + "\n")

        # Get daily costs
        print(f"Fetching {days} days of cost data...")
        daily_costs = self.get_daily_costs(days)

        if not daily_costs:
            print("No cost data available for the specified period.")
            return

        # Display daily costs
        print(f"\n📅 DAILY COSTS (Last {days} Days)")
        print("-" * 80)
        print(f"  {'Date':<12} {'Cost':<15} {'Visualization'}")
        print(f"  {'-'*12} {'-'*15} {'-'*30}")

        max_cost = max(cost for _, cost in daily_costs)

        for date, cost in daily_costs[-14:]:  # Show last 14 days
            bar_length = int((cost / max_cost * 30)) if max_cost > 0 else 0
            bar = '█' * bar_length
            print(f"  {date:<12} ${cost:<14.6f} {bar}")

        # Calculate statistics
        print(f"\n📊 STATISTICS")
        print("-" * 80)

        all_costs = [cost for _, cost in daily_costs]
        total_cost = sum(all_costs)
        avg_cost = total_cost / len(all_costs) if all_costs else 0
        min_cost = min(all_costs) if all_costs else 0
        max_cost = max(all_costs) if all_costs else 0

        print(f"  Total Cost:      ${total_cost:.6f}")
        print(f"  Average/Day:     ${avg_cost:.6f}")
        print(f"  Minimum/Day:     ${min_cost:.6f}")
        print(f"  Maximum/Day:     ${max_cost:.6f}")
        print(f"  Std Deviation:   ${self._std_dev(all_costs):.6f}")

        # Calculate moving average
        print(f"\n📈 7-DAY MOVING AVERAGE")
        print("-" * 80)

        moving_avg = self.calculate_moving_average(daily_costs, window=7)

        if moving_avg:
            for date, avg in moving_avg[-7:]:  # Show last 7 days
                print(f"  {date}:  ${avg:.6f}")

        # Calculate trend
        print(f"\n📉 TREND ANALYSIS")
        print("-" * 80)

        trend = self.calculate_trend(daily_costs)

        direction_icon = {
            'increasing': '📈',
            'decreasing': '📉',
            'stable': '➡️',
            'insufficient_data': '❓'
        }

        icon = direction_icon.get(trend['direction'], '❓')
        direction = trend['direction'].upper()
        change = trend.get('change_percent', 0)

        print(f"  Trend Direction: {icon} {direction}")
        print(f"  Change:          {change:+.2f}%")

        if trend['direction'] != 'insufficient_data':
            print(f"  First Half Avg:  ${trend['first_half_avg']:.6f}")
            print(f"  Second Half Avg: ${trend['second_half_avg']:.6f}")

        print("\n" + "=" * 80)

    def _std_dev(self, values: List[float]) -> float:
        """Calculate standard deviation"""
        if not values:
            return 0.0

        mean = sum(values) / len(values)
        variance = sum((x - mean) ** 2 for x in values) / len(values)
        return variance ** 0.5


if __name__ == "__main__":
    analyzer = TimeSeriesAnalyzer()
    analyzer.display_time_series(days=30)
```

Run the analysis:

```bash
python time_series_analysis.py
```

### For TypeScript Users:

Create `time-series-analysis.ts`:

```typescript
#!/usr/bin/env ts-node
/**
 * Time-Series Cost Analysis
 */

import { CostOpsClient, CostRecord } from './cost-ops-client';

interface DailyCost {
  date: string;
  cost: number;
}

interface TrendAnalysis {
  direction: string;
  change_percent: number;
  first_half_avg: number;
  second_half_avg: number;
}

class TimeSeriesAnalyzer {
  private client: CostOpsClient;

  constructor(baseUrl: string = 'http://localhost:8080') {
    this.client = new CostOpsClient(baseUrl);
  }

  async getDailyCosts(days: number = 30): Promise<DailyCost[]> {
    const endDate = new Date();
    const startDate = new Date(endDate.getTime() - days * 24 * 60 * 60 * 1000);

    const costs = await this.client.getCosts({
      start_date: startDate.toISOString(),
      end_date: endDate.toISOString()
    });

    // Group by day
    const dailyCosts = new Map<string, number>();

    for (const cost of costs) {
      const date = cost.timestamp.split('T')[0];
      const currentCost = dailyCosts.get(date) || 0;
      dailyCosts.set(date, currentCost + parseFloat(cost.total_cost));
    }

    // Sort by date
    const sorted = Array.from(dailyCosts.entries())
      .map(([date, cost]) => ({ date, cost }))
      .sort((a, b) => a.date.localeCompare(b.date));

    return sorted;
  }

  calculateMovingAverage(dailyCosts: DailyCost[], window: number = 7): DailyCost[] {
    if (dailyCosts.length < window) {
      return dailyCosts;
    }

    const movingAvg: DailyCost[] = [];

    for (let i = 0; i <= dailyCosts.length - window; i++) {
      const windowCosts = dailyCosts.slice(i, i + window).map(dc => dc.cost);
      const avg = windowCosts.reduce((sum, cost) => sum + cost, 0) / window;
      const date = dailyCosts[i + window - 1].date;
      movingAvg.push({ date, cost: avg });
    }

    return movingAvg;
  }

  calculateTrend(dailyCosts: DailyCost[]): TrendAnalysis {
    if (dailyCosts.length < 2) {
      return {
        direction: 'insufficient_data',
        change_percent: 0,
        first_half_avg: 0,
        second_half_avg: 0
      };
    }

    const midPoint = Math.floor(dailyCosts.length / 2);
    const firstHalf = dailyCosts.slice(0, midPoint).map(dc => dc.cost);
    const secondHalf = dailyCosts.slice(midPoint).map(dc => dc.cost);

    const firstAvg = firstHalf.reduce((sum, cost) => sum + cost, 0) / firstHalf.length;
    const secondAvg = secondHalf.reduce((sum, cost) => sum + cost, 0) / secondHalf.length;

    let direction: string;
    if (secondAvg > firstAvg * 1.1) {
      direction = 'increasing';
    } else if (secondAvg < firstAvg * 0.9) {
      direction = 'decreasing';
    } else {
      direction = 'stable';
    }

    const changePercent = firstAvg > 0 ? ((secondAvg - firstAvg) / firstAvg * 100) : 0;

    return {
      direction,
      change_percent: changePercent,
      first_half_avg: firstAvg,
      second_half_avg: secondAvg
    };
  }

  private stdDev(values: number[]): number {
    if (values.length === 0) return 0;

    const mean = values.reduce((sum, val) => sum + val, 0) / values.length;
    const variance = values.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / values.length;
    return Math.sqrt(variance);
  }

  async displayTimeSeries(days: number = 30): Promise<void> {
    console.log('='.repeat(80));
    console.log('TIME-SERIES COST ANALYSIS');
    console.log('='.repeat(80) + '\n');

    console.log(`Fetching ${days} days of cost data...`);
    const dailyCosts = await this.getDailyCosts(days);

    if (dailyCosts.length === 0) {
      console.log('No cost data available for the specified period.');
      return;
    }

    // Display daily costs
    console.log(`\n📅 DAILY COSTS (Last ${days} Days)`);
    console.log('-'.repeat(80));
    console.log(`  ${'Date'.padEnd(12)} ${'Cost'.padEnd(15)} ${'Visualization'}`);
    console.log(`  ${'-'.repeat(12)} ${'-'.repeat(15)} ${'-'.repeat(30)}`);

    const maxCost = Math.max(...dailyCosts.map(dc => dc.cost));
    const last14 = dailyCosts.slice(-14);

    for (const { date, cost } of last14) {
      const barLength = maxCost > 0 ? Math.floor((cost / maxCost) * 30) : 0;
      const bar = '█'.repeat(barLength);
      console.log(`  ${date.padEnd(12)} $${cost.toFixed(6).padEnd(14)} ${bar}`);
    }

    // Statistics
    console.log(`\n📊 STATISTICS`);
    console.log('-'.repeat(80));

    const allCosts = dailyCosts.map(dc => dc.cost);
    const totalCost = allCosts.reduce((sum, cost) => sum + cost, 0);
    const avgCost = totalCost / allCosts.length;
    const minCost = Math.min(...allCosts);
    const maxCostValue = Math.max(...allCosts);
    const stdDevValue = this.stdDev(allCosts);

    console.log(`  Total Cost:      $${totalCost.toFixed(6)}`);
    console.log(`  Average/Day:     $${avgCost.toFixed(6)}`);
    console.log(`  Minimum/Day:     $${minCost.toFixed(6)}`);
    console.log(`  Maximum/Day:     $${maxCostValue.toFixed(6)}`);
    console.log(`  Std Deviation:   $${stdDevValue.toFixed(6)}`);

    // Moving average
    console.log(`\n📈 7-DAY MOVING AVERAGE`);
    console.log('-'.repeat(80));

    const movingAvg = this.calculateMovingAverage(dailyCosts, 7);
    const last7MovingAvg = movingAvg.slice(-7);

    for (const { date, cost } of last7MovingAvg) {
      console.log(`  ${date}:  $${cost.toFixed(6)}`);
    }

    // Trend analysis
    console.log(`\n📉 TREND ANALYSIS`);
    console.log('-'.repeat(80));

    const trend = this.calculateTrend(dailyCosts);

    const directionIcons: Record<string, string> = {
      'increasing': '📈',
      'decreasing': '📉',
      'stable': '➡️',
      'insufficient_data': '❓'
    };

    const icon = directionIcons[trend.direction] || '❓';

    console.log(`  Trend Direction: ${icon} ${trend.direction.toUpperCase()}`);
    console.log(`  Change:          ${trend.change_percent >= 0 ? '+' : ''}${trend.change_percent.toFixed(2)}%`);

    if (trend.direction !== 'insufficient_data') {
      console.log(`  First Half Avg:  $${trend.first_half_avg.toFixed(6)}`);
      console.log(`  Second Half Avg: $${trend.second_half_avg.toFixed(6)}`);
    }

    console.log('\n' + '='.repeat(80));
  }
}

// Run analysis
const analyzer = new TimeSeriesAnalyzer();
analyzer.displayTimeSeries(30).catch(console.error);
```

Run the analysis:

```bash
npx ts-node time-series-analysis.ts
```

## Part 3: Comparative Analysis

Create month-over-month and year-over-year comparison tools.

### For Python Users:

Create `comparative_analysis.py`:

```python
#!/usr/bin/env python3
"""
Comparative Cost Analysis (MoM, YoY)
"""

from datetime import datetime, timedelta
from typing import Dict, List, Tuple
from collections import defaultdict

from cost_ops_client import CostOpsClient

class ComparativeAnalyzer:
    """Month-over-Month and Year-over-Year analysis"""

    def __init__(self, base_url: str = "http://localhost:8080"):
        self.client = CostOpsClient(base_url=base_url)

    def get_monthly_summary(self, year: int, month: int) -> Dict:
        """Get summary for a specific month"""
        # Calculate start and end dates for the month
        start_date = datetime(year, month, 1)

        # Calculate last day of month
        if month == 12:
            end_date = datetime(year + 1, 1, 1) - timedelta(seconds=1)
        else:
            end_date = datetime(year, month + 1, 1) - timedelta(seconds=1)

        try:
            summary = self.client.get_summary(
                start_date=start_date.isoformat(),
                end_date=end_date.isoformat(),
                group_by=['provider', 'model']
            )
            return summary
        except Exception as e:
            print(f"Error getting summary for {year}-{month:02d}: {e}")
            return {}

    def month_over_month_analysis(self):
        """Compare this month vs last month"""
        today = datetime.utcnow()

        # Current month
        current_month = today.month
        current_year = today.year

        # Previous month
        if current_month == 1:
            prev_month = 12
            prev_year = current_year - 1
        else:
            prev_month = current_month - 1
            prev_year = current_year

        print("=" * 80)
        print("MONTH-OVER-MONTH COMPARISON")
        print("=" * 80 + "\n")

        # Get summaries
        print(f"Fetching data for {prev_year}-{prev_month:02d} and {current_year}-{current_month:02d}...")

        prev_summary = self.get_monthly_summary(prev_year, prev_month)
        current_summary = self.get_monthly_summary(current_year, current_month)

        # Overall comparison
        print(f"\n📊 OVERALL COMPARISON")
        print("-" * 80)

        prev_cost = float(prev_summary.get('total_cost', 0))
        current_cost = float(current_summary.get('total_cost', 0))

        change = current_cost - prev_cost
        change_percent = (change / prev_cost * 100) if prev_cost > 0 else 0

        print(f"  Previous Month ({prev_year}-{prev_month:02d}):  ${prev_cost:.6f}")
        print(f"  Current Month ({current_year}-{current_month:02d}):   ${current_cost:.6f}")
        print(f"  Change:                      ${change:+.6f} ({change_percent:+.2f}%)")

        if change > 0:
            print(f"  Status:                      📈 INCREASING")
        elif change < 0:
            print(f"  Status:                      📉 DECREASING")
        else:
            print(f"  Status:                      ➡️ STABLE")

        # Provider comparison
        print(f"\n🏢 PROVIDER COMPARISON")
        print("-" * 80)
        print(f"  {'Provider':<15} {'Previous':<15} {'Current':<15} {'Change':<15}")
        print(f"  {'-'*15} {'-'*15} {'-'*15} {'-'*15}")

        all_providers = set()
        all_providers.update(prev_summary.get('by_provider', {}).keys())
        all_providers.update(current_summary.get('by_provider', {}).keys())

        for provider in sorted(all_providers):
            prev = float(prev_summary.get('by_provider', {}).get(provider, 0))
            curr = float(current_summary.get('by_provider', {}).get(provider, 0))
            chg = curr - prev
            chg_pct = (chg / prev * 100) if prev > 0 else (100 if curr > 0 else 0)

            print(f"  {provider:<15} ${prev:<14.6f} ${curr:<14.6f} {chg:+.6f} ({chg_pct:+.1f}%)")

        # Model comparison
        print(f"\n🤖 TOP MODELS COMPARISON")
        print("-" * 80)

        # Get top 5 models from current month
        current_models = current_summary.get('by_model', {})
        top_models = sorted(
            current_models.items(),
            key=lambda x: float(x[1]),
            reverse=True
        )[:5]

        print(f"  {'Model':<35} {'Previous':<15} {'Current':<15} {'Change':<15}")
        print(f"  {'-'*35} {'-'*15} {'-'*15} {'-'*15}")

        for model, _ in top_models:
            prev = float(prev_summary.get('by_model', {}).get(model, 0))
            curr = float(current_models.get(model, 0))
            chg = curr - prev
            chg_pct = (chg / prev * 100) if prev > 0 else (100 if curr > 0 else 0)

            print(f"  {model:<35} ${prev:<14.6f} ${curr:<14.6f} {chg:+.6f} ({chg_pct:+.1f}%)")

        print("\n" + "=" * 80)

    def year_over_year_analysis(self):
        """Compare this year vs last year (same month)"""
        today = datetime.utcnow()

        current_month = today.month
        current_year = today.year
        prev_year = current_year - 1

        print("=" * 80)
        print("YEAR-OVER-YEAR COMPARISON")
        print("=" * 80 + "\n")

        # Get summaries
        print(f"Fetching data for {prev_year}-{current_month:02d} and {current_year}-{current_month:02d}...")

        prev_summary = self.get_monthly_summary(prev_year, current_month)
        current_summary = self.get_monthly_summary(current_year, current_month)

        # Overall comparison
        print(f"\n📊 OVERALL COMPARISON")
        print("-" * 80)

        prev_cost = float(prev_summary.get('total_cost', 0))
        current_cost = float(current_summary.get('total_cost', 0))

        change = current_cost - prev_cost
        change_percent = (change / prev_cost * 100) if prev_cost > 0 else 0

        print(f"  {prev_year}-{current_month:02d}:  ${prev_cost:.6f}")
        print(f"  {current_year}-{current_month:02d}:  ${current_cost:.6f}")
        print(f"  YoY Change:  ${change:+.6f} ({change_percent:+.2f}%)")

        # Annualized projection
        if current_month > 0:
            ytd_cost = current_cost * current_month  # Simplified
            projected_annual = ytd_cost / current_month * 12
            print(f"  Projected Annual Cost (Current Rate): ${projected_annual:.2f}")

        print("\n" + "=" * 80)


if __name__ == "__main__":
    analyzer = ComparativeAnalyzer()

    print("\nRunning Month-over-Month Analysis...\n")
    analyzer.month_over_month_analysis()

    print("\n\nRunning Year-over-Year Analysis...\n")
    analyzer.year_over_year_analysis()
```

Run the analysis:

```bash
python comparative_analysis.py
```

## Part 4: Report Generation and Scheduling

Create automated report generation with scheduling.

### For Python Users:

Create `report_generator.py`:

```python
#!/usr/bin/env python3
"""
Automated Report Generation
"""

from datetime import datetime, timedelta
from typing import Dict, List, Any
import json
import csv
from pathlib import Path

from cost_ops_client import CostOpsClient

class ReportGenerator:
    """Generate automated cost reports"""

    def __init__(self, base_url: str = "http://localhost:8080"):
        self.client = CostOpsClient(base_url=base_url)
        self.reports_dir = Path("reports")
        self.reports_dir.mkdir(exist_ok=True)

    def generate_executive_summary(self, days: int = 30) -> Dict:
        """Generate executive summary report"""
        end_date = datetime.utcnow()
        start_date = end_date - timedelta(days=days)

        summary = self.client.get_summary(
            start_date=start_date.isoformat(),
            end_date=end_date.isoformat(),
            group_by=['provider', 'model', 'project']
        )

        # Get detailed costs
        costs = self.client.get_costs(
            start_date=start_date.isoformat(),
            end_date=end_date.isoformat()
        )

        report = {
            'report_type': 'executive_summary',
            'generated_at': datetime.utcnow().isoformat(),
            'period': {
                'start': start_date.isoformat(),
                'end': end_date.isoformat(),
                'days': days
            },
            'summary': {
                'total_cost': summary.get('total_cost'),
                'total_requests': summary.get('total_requests'),
                'avg_cost_per_request': summary.get('avg_cost_per_request'),
                'daily_average': float(summary.get('total_cost', 0)) / days
            },
            'breakdown': {
                'by_provider': summary.get('by_provider', {}),
                'by_model': summary.get('by_model', {}),
                'by_project': summary.get('by_project', {})
            },
            'details': {
                'total_records': len(costs),
                'unique_models': len(set(c['model'] for c in costs)),
                'unique_providers': len(set(c['provider'] for c in costs))
            }
        }

        return report

    def save_report_json(self, report: Dict, filename: str):
        """Save report as JSON"""
        filepath = self.reports_dir / filename
        with open(filepath, 'w') as f:
            json.dump(report, f, indent=2)
        print(f"✓ Report saved to {filepath}")
        return filepath

    def save_report_csv(self, costs: List[Dict], filename: str):
        """Save cost details as CSV"""
        filepath = self.reports_dir / filename

        if not costs:
            print("No cost data to save")
            return None

        # Write CSV
        with open(filepath, 'w', newline='') as f:
            writer = csv.DictWriter(f, fieldnames=costs[0].keys())
            writer.writeheader()
            writer.writerows(costs)

        print(f"✓ CSV report saved to {filepath}")
        return filepath

    def generate_html_report(self, report: Dict, filename: str):
        """Generate HTML report"""
        html = f"""
<!DOCTYPE html>
<html>
<head>
    <title>LLM Cost Report</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .header {{
            background-color: #2c3e50;
            color: white;
            padding: 20px;
            border-radius: 5px;
        }}
        .section {{
            background-color: white;
            margin: 20px 0;
            padding: 20px;
            border-radius: 5px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .metric {{
            display: inline-block;
            margin: 10px 20px;
        }}
        .metric-value {{
            font-size: 24px;
            font-weight: bold;
            color: #2c3e50;
        }}
        .metric-label {{
            font-size: 14px;
            color: #7f8c8d;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
        }}
        th, td {{
            padding: 10px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }}
        th {{
            background-color: #34495e;
            color: white;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>LLM Cost Operations Report</h1>
        <p>Generated: {report['generated_at']}</p>
        <p>Period: {report['period']['start']} to {report['period']['end']}</p>
    </div>

    <div class="section">
        <h2>Executive Summary</h2>
        <div class="metric">
            <div class="metric-label">Total Cost</div>
            <div class="metric-value">${report['summary']['total_cost']}</div>
        </div>
        <div class="metric">
            <div class="metric-label">Total Requests</div>
            <div class="metric-value">{report['summary']['total_requests']}</div>
        </div>
        <div class="metric">
            <div class="metric-label">Avg Cost/Request</div>
            <div class="metric-value">${report['summary']['avg_cost_per_request']}</div>
        </div>
        <div class="metric">
            <div class="metric-label">Daily Average</div>
            <div class="metric-value">${report['summary']['daily_average']:.6f}</div>
        </div>
    </div>

    <div class="section">
        <h2>Cost by Provider</h2>
        <table>
            <tr>
                <th>Provider</th>
                <th>Total Cost</th>
            </tr>
"""

        for provider, cost in report['breakdown']['by_provider'].items():
            html += f"""
            <tr>
                <td>{provider}</td>
                <td>${cost}</td>
            </tr>
"""

        html += """
        </table>
    </div>

    <div class="section">
        <h2>Top Models by Cost</h2>
        <table>
            <tr>
                <th>Model</th>
                <th>Total Cost</th>
            </tr>
"""

        # Sort models by cost
        sorted_models = sorted(
            report['breakdown']['by_model'].items(),
            key=lambda x: float(x[1]),
            reverse=True
        )[:10]

        for model, cost in sorted_models:
            html += f"""
            <tr>
                <td>{model}</td>
                <td>${cost}</td>
            </tr>
"""

        html += """
        </table>
    </div>
</body>
</html>
"""

        filepath = self.reports_dir / filename
        with open(filepath, 'w') as f:
            f.write(html)

        print(f"✓ HTML report saved to {filepath}")
        return filepath

    def generate_all_reports(self, days: int = 30):
        """Generate all report formats"""
        print("=" * 80)
        print("GENERATING COST REPORTS")
        print("=" * 80 + "\n")

        # Generate executive summary
        print("Generating executive summary...")
        report = self.generate_executive_summary(days)

        # Save in multiple formats
        timestamp = datetime.utcnow().strftime('%Y%m%d_%H%M%S')

        # JSON report
        self.save_report_json(report, f"executive_summary_{timestamp}.json")

        # HTML report
        self.generate_html_report(report, f"executive_summary_{timestamp}.html")

        # CSV export of raw costs
        print("\nExporting raw cost data...")
        end_date = datetime.utcnow()
        start_date = end_date - timedelta(days=days)

        costs = self.client.get_costs(
            start_date=start_date.isoformat(),
            end_date=end_date.isoformat()
        )

        self.save_report_csv(costs, f"cost_details_{timestamp}.csv")

        print("\n" + "=" * 80)
        print(f"All reports generated in: {self.reports_dir.absolute()}")
        print("=" * 80)


if __name__ == "__main__":
    generator = ReportGenerator()
    generator.generate_all_reports(days=30)
```

Run the report generator:

```bash
python report_generator.py
```

## Part 5: Data Visualization

Create visual charts for cost data.

### For Python Users (with matplotlib):

Create `cost_visualizations.py`:

```python
#!/usr/bin/env python3
"""
Cost Data Visualizations
"""

import matplotlib.pyplot as plt
import matplotlib.dates as mdates
from datetime import datetime, timedelta
from typing import Dict, List, Tuple
from collections import defaultdict
from pathlib import Path

from cost_ops_client import CostOpsClient

class CostVisualizer:
    """Create cost visualizations"""

    def __init__(self, base_url: str = "http://localhost:8080"):
        self.client = CostOpsClient(base_url=base_url)
        self.charts_dir = Path("charts")
        self.charts_dir.mkdir(exist_ok=True)

    def plot_daily_costs(self, days: int = 30):
        """Plot daily cost trend"""
        end_date = datetime.utcnow()
        start_date = end_date - timedelta(days=days)

        costs = self.client.get_costs(
            start_date=start_date.isoformat(),
            end_date=end_date.isoformat()
        )

        # Group by day
        daily_costs = defaultdict(float)

        for cost in costs:
            timestamp = datetime.fromisoformat(cost['timestamp'].replace('Z', '+00:00'))
            date_key = timestamp.strftime('%Y-%m-%d')
            daily_costs[date_key] += float(cost['total_cost'])

        # Sort by date
        sorted_data = sorted(daily_costs.items())
        dates = [datetime.strptime(d, '%Y-%m-%d') for d, _ in sorted_data]
        costs_list = [c for _, c in sorted_data]

        # Create plot
        plt.figure(figsize=(12, 6))
        plt.plot(dates, costs_list, marker='o', linestyle='-', linewidth=2)
        plt.title('Daily Cost Trend', fontsize=16, fontweight='bold')
        plt.xlabel('Date', fontsize=12)
        plt.ylabel('Cost ($)', fontsize=12)
        plt.grid(True, alpha=0.3)
        plt.xticks(rotation=45)
        plt.tight_layout()

        # Save chart
        filepath = self.charts_dir / 'daily_costs.png'
        plt.savefig(filepath, dpi=300, bbox_inches='tight')
        print(f"✓ Daily cost chart saved to {filepath}")
        plt.close()

    def plot_provider_pie(self):
        """Plot provider cost distribution"""
        end_date = datetime.utcnow()
        start_date = end_date - timedelta(days=30)

        summary = self.client.get_summary(
            start_date=start_date.isoformat(),
            end_date=end_date.isoformat(),
            group_by=['provider']
        )

        by_provider = summary.get('by_provider', {})

        if not by_provider:
            print("No provider data available")
            return

        providers = list(by_provider.keys())
        costs = [float(c) for c in by_provider.values()]

        # Create pie chart
        plt.figure(figsize=(10, 8))
        plt.pie(costs, labels=providers, autopct='%1.1f%%', startangle=90)
        plt.title('Cost Distribution by Provider', fontsize=16, fontweight='bold')
        plt.axis('equal')

        filepath = self.charts_dir / 'provider_distribution.png'
        plt.savefig(filepath, dpi=300, bbox_inches='tight')
        print(f"✓ Provider distribution chart saved to {filepath}")
        plt.close()

    def plot_model_comparison(self):
        """Plot top models cost comparison"""
        end_date = datetime.utcnow()
        start_date = end_date - timedelta(days=30)

        summary = self.client.get_summary(
            start_date=start_date.isoformat(),
            end_date=end_date.isoformat(),
            group_by=['model']
        )

        by_model = summary.get('by_model', {})

        if not by_model:
            print("No model data available")
            return

        # Get top 10 models
        sorted_models = sorted(
            by_model.items(),
            key=lambda x: float(x[1]),
            reverse=True
        )[:10]

        models = [m[:30] for m, _ in sorted_models]  # Truncate long names
        costs = [float(c) for _, c in sorted_models]

        # Create bar chart
        plt.figure(figsize=(12, 8))
        plt.barh(models, costs)
        plt.title('Top 10 Models by Cost', fontsize=16, fontweight='bold')
        plt.xlabel('Cost ($)', fontsize=12)
        plt.ylabel('Model', fontsize=12)
        plt.grid(True, alpha=0.3, axis='x')
        plt.tight_layout()

        filepath = self.charts_dir / 'model_comparison.png'
        plt.savefig(filepath, dpi=300, bbox_inches='tight')
        print(f"✓ Model comparison chart saved to {filepath}")
        plt.close()

    def generate_all_charts(self):
        """Generate all visualizations"""
        print("=" * 80)
        print("GENERATING COST VISUALIZATIONS")
        print("=" * 80 + "\n")

        self.plot_daily_costs()
        self.plot_provider_pie()
        self.plot_model_comparison()

        print("\n" + "=" * 80)
        print(f"All charts generated in: {self.charts_dir.absolute()}")
        print("=" * 80)


if __name__ == "__main__":
    visualizer = CostVisualizer()
    visualizer.generate_all_charts()
```

Run the visualizer:

```bash
python cost_visualizations.py
```

## Exercises and Challenges

### Exercise 1: Custom Dashboard

Create a dashboard that shows:
- Top 5 most expensive days
- Busiest day of the week
- Hour-of-day cost pattern
- Weekend vs weekday comparison

### Exercise 2: Forecast Report

Create a report that projects next month's costs based on current trends.

### Exercise 3: Anomaly Detection Report

Create a report that identifies:
- Days with unusually high costs
- Sudden changes in provider usage
- New models being used

### Exercise 4: Multi-Organization Report

Generate a comparative report across multiple organizations.

### Exercise 5: Schedule Implementation

Implement a cron-like scheduler to generate reports daily at specific times.

## Solutions

### Solution to Exercise 1:

```python
#!/usr/bin/env python3
"""
Custom Dashboard: Advanced Insights
"""

from datetime import datetime, timedelta
from collections import defaultdict, Counter
from cost_ops_client import CostOpsClient

class AdvancedDashboard:
    def __init__(self):
        self.client = CostOpsClient()

    def get_top_expensive_days(self, days=30, top_n=5):
        """Find top N most expensive days"""
        end_date = datetime.utcnow()
        start_date = end_date - timedelta(days=days)

        costs = self.client.get_costs(
            start_date=start_date.isoformat(),
            end_date=end_date.isoformat()
        )

        daily_costs = defaultdict(float)
        for cost in costs:
            timestamp = datetime.fromisoformat(cost['timestamp'].replace('Z', '+00:00'))
            date_key = timestamp.strftime('%Y-%m-%d')
            daily_costs[date_key] += float(cost['total_cost'])

        # Sort and get top N
        top_days = sorted(daily_costs.items(), key=lambda x: x[1], reverse=True)[:top_n]

        print(f"\n📊 TOP {top_n} MOST EXPENSIVE DAYS")
        print("-" * 60)
        for idx, (date, cost) in enumerate(top_days, 1):
            print(f"  {idx}. {date}:  ${cost:.6f}")

    def analyze_day_of_week(self, days=30):
        """Analyze costs by day of week"""
        end_date = datetime.utcnow()
        start_date = end_date - timedelta(days=days)

        costs = self.client.get_costs(
            start_date=start_date.isoformat(),
            end_date=end_date.isoformat()
        )

        dow_costs = defaultdict(float)
        dow_counts = defaultdict(int)

        for cost in costs:
            timestamp = datetime.fromisoformat(cost['timestamp'].replace('Z', '+00:00'))
            dow = timestamp.strftime('%A')
            dow_costs[dow] += float(cost['total_cost'])
            dow_counts[dow] += 1

        # Calculate averages
        days_order = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday']

        print(f"\n📅 COSTS BY DAY OF WEEK")
        print("-" * 60)
        for day in days_order:
            total = dow_costs.get(day, 0)
            count = dow_counts.get(day, 0)
            avg = total / count if count > 0 else 0
            print(f"  {day:<10}  Total: ${total:<10.6f}  Avg: ${avg:.6f}")

        # Find busiest day
        busiest_day = max(dow_counts.items(), key=lambda x: x[1])
        print(f"\n  Busiest Day: {busiest_day[0]} ({busiest_day[1]} requests)")

    def run(self):
        print("=" * 60)
        print("ADVANCED COST DASHBOARD")
        print("=" * 60)

        self.get_top_expensive_days()
        self.analyze_day_of_week()

        print("\n" + "=" * 60)

if __name__ == "__main__":
    dashboard = AdvancedDashboard()
    dashboard.run()
```

## Troubleshooting Guide

### Common Issues

1. **No data in visualizations**: Ensure you have populated the database with usage records from Lab 1
2. **Import errors**: Install missing dependencies with `pip install matplotlib pandas`
3. **Report generation fails**: Check write permissions in the reports directory
4. **Empty summaries**: Verify the date range includes periods with actual usage data

## Review Questions

1. What is the difference between time-series analysis and comparative analysis?
2. How can moving averages help identify cost trends?
3. What are the benefits of generating reports in multiple formats?
4. How would you detect cost anomalies in your data?
5. What metrics are most important for an executive summary?

## Next Steps

Continue to **Lab 3: Budget Management** to learn about:
- Setting budget limits
- Configuring alerts
- Budget forecasting
- Multi-level budgets

## Additional Resources

- Matplotlib documentation: https://matplotlib.org/
- Pandas for data analysis: https://pandas.pydata.org/
- Chart.js for TypeScript: https://www.chartjs.org/

---

**End of Lab 2**
