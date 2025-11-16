# LLM Cost Ops Analyst Guide

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
   - [Accessing the Platform](#accessing-the-platform)
   - [Dashboard Overview](#dashboard-overview)
   - [User Interface Navigation](#user-interface-navigation)
3. [Cost Analysis Methodologies](#cost-analysis-methodologies)
   - [Time-Series Analysis](#time-series-analysis)
   - [Comparative Analysis](#comparative-analysis)
   - [Trend Analysis](#trend-analysis)
   - [Anomaly Detection](#anomaly-detection)
4. [Dashboard Creation and Customization](#dashboard-creation-and-customization)
   - [Pre-built Dashboards](#pre-built-dashboards)
   - [Custom Dashboard Builder](#custom-dashboard-builder)
   - [Widget Library](#widget-library)
   - [Dashboard Sharing](#dashboard-sharing)
5. [Report Generation and Scheduling](#report-generation-and-scheduling)
   - [Standard Reports](#standard-reports)
   - [Custom Reports](#custom-reports)
   - [Automated Scheduling](#automated-scheduling)
   - [Report Distribution](#report-distribution)
6. [Data Visualization Best Practices](#data-visualization-best-practices)
   - [Chart Types](#chart-types)
   - [Color Schemes](#color-schemes)
   - [Interactive Elements](#interactive-elements)
7. [Forecasting and Predictive Analysis](#forecasting-and-predictive-analysis)
   - [Cost Forecasting Models](#cost-forecasting-models)
   - [Budget Projections](#budget-projections)
   - [Capacity Planning](#capacity-planning)
8. [Budget Tracking and Variance Analysis](#budget-tracking-and-variance-analysis)
   - [Budget Setup](#budget-setup)
   - [Variance Reporting](#variance-reporting)
   - [Alert Configuration](#alert-configuration)
9. [Cost Allocation Strategies](#cost-allocation-strategies)
   - [Team-Based Allocation](#team-based-allocation)
   - [Project-Based Allocation](#project-based-allocation)
   - [Tag-Based Allocation](#tag-based-allocation)
10. [Chargeback and Showback](#chargeback-and-showback)
    - [Chargeback Models](#chargeback-models)
    - [Showback Reports](#showback-reports)
    - [Cost Center Reporting](#cost-center-reporting)
11. [Data Export and Integration](#data-export-and-integration)
    - [Export Formats](#export-formats)
    - [API Access](#api-access)
    - [BI Tool Integration](#bi-tool-integration)
12. [Advanced SQL Queries](#advanced-sql-queries)
    - [Common Query Patterns](#common-query-patterns)
    - [Custom Analytics](#custom-analytics)
    - [Performance Optimization](#performance-optimization)
13. [Key Performance Indicators (KPIs)](#key-performance-indicators-kpis)
    - [Cost Metrics](#cost-metrics)
    - [Efficiency Metrics](#efficiency-metrics)
    - [Usage Metrics](#usage-metrics)
14. [Cost Optimization](#cost-optimization)
    - [Identifying Opportunities](#identifying-opportunities)
    - [Optimization Recommendations](#optimization-recommendations)
    - [Implementation Tracking](#implementation-tracking)
15. [Case Studies and Examples](#case-studies-and-examples)
16. [Best Practices](#best-practices)
17. [Troubleshooting](#troubleshooting)

---

## Introduction

Welcome to the LLM Cost Ops Analyst Guide. This guide is designed for data analysts, financial analysts, and business intelligence professionals responsible for analyzing, reporting, and optimizing LLM API costs within their organizations.

### Who This Guide Is For

- Financial Analysts
- Data Analysts
- Business Intelligence Analysts
- Cost Optimization Specialists
- Product Managers
- Engineering Managers

### What You'll Learn

This guide covers:
- Comprehensive cost analysis techniques
- Dashboard and report creation
- Data visualization best practices
- Forecasting and budgeting
- Cost optimization strategies
- Integration with BI tools

---

## Getting Started

### Accessing the Platform

#### Web Interface Login

1. Navigate to `https://app.llmcostops.com`
2. Enter your credentials
3. Complete two-factor authentication (if enabled)
4. You'll land on the main dashboard

#### API Access

```python
from llm_cost_ops import CostOpsClient

# Initialize client
client = CostOpsClient(api_key="your-api-key")

# Verify access
user = client.users.me()
print(f"Logged in as: {user.name} ({user.role})")
```

### Dashboard Overview

The main dashboard provides:

- **Cost Overview**: Total spending, current month costs, budget utilization
- **Trend Charts**: Cost trends over time, model usage distribution
- **Top Consumers**: Highest spending users, teams, and projects
- **Recent Activity**: Latest cost entries and significant changes
- **Alerts**: Budget warnings and anomaly notifications

### User Interface Navigation

**Main Navigation Menu:**

- **Dashboards**: Access to all dashboards
- **Reports**: Generate and view reports
- **Analytics**: Advanced analysis tools
- **Budgets**: Budget management
- **Settings**: User preferences and configuration

**Quick Actions:**

- Create new dashboard
- Generate report
- Export data
- Set up alert

---

## Cost Analysis Methodologies

### Time-Series Analysis

#### Daily Cost Analysis

```python
from llm_cost_ops import CostOpsClient
import pandas as pd
import matplotlib.pyplot as plt
from datetime import datetime, timedelta

client = CostOpsClient(api_key="your-api-key")

# Get daily costs for last 30 days
end_date = datetime.now()
start_date = end_date - timedelta(days=30)

costs = client.costs.aggregate(
    start_date=start_date,
    end_date=end_date,
    group_by=['date'],
    metrics=['sum', 'count', 'avg']
)

# Convert to DataFrame
df = pd.DataFrame(costs)
df['date'] = pd.to_datetime(df['date'])
df = df.set_index('date')

# Plot daily costs
plt.figure(figsize=(12, 6))
plt.plot(df.index, df['sum'], marker='o')
plt.title('Daily LLM Costs - Last 30 Days')
plt.xlabel('Date')
plt.ylabel('Cost ($)')
plt.grid(True, alpha=0.3)
plt.xticks(rotation=45)
plt.tight_layout()
plt.savefig('daily_costs.png')

# Calculate statistics
print(f"Average daily cost: ${df['sum'].mean():.2f}")
print(f"Maximum daily cost: ${df['sum'].max():.2f}")
print(f"Total cost: ${df['sum'].sum():.2f}")
```

#### Weekly and Monthly Trends

```python
# Weekly aggregation
weekly_costs = client.costs.aggregate(
    start_date=start_date,
    end_date=end_date,
    group_by=['week'],
    metrics=['sum', 'count', 'avg']
)

df_weekly = pd.DataFrame(weekly_costs)

# Monthly aggregation
monthly_costs = client.costs.aggregate(
    start_date=datetime.now() - timedelta(days=365),
    end_date=datetime.now(),
    group_by=['month'],
    metrics=['sum', 'count', 'avg']
)

df_monthly = pd.DataFrame(monthly_costs)

# Year-over-year comparison
current_year = df_monthly[df_monthly['month'].str.contains('2025')]
previous_year = df_monthly[df_monthly['month'].str.contains('2024')]

# Calculate growth rate
growth_rate = ((current_year['sum'].sum() - previous_year['sum'].sum()) /
               previous_year['sum'].sum() * 100)

print(f"Year-over-year cost growth: {growth_rate:.1f}%")
```

### Comparative Analysis

#### Model Comparison

```python
# Compare costs across different models
model_costs = client.costs.aggregate(
    start_date=start_date,
    end_date=end_date,
    group_by=['model'],
    metrics=['sum', 'count', 'avg'],
    order_by=['-sum']
)

df_models = pd.DataFrame(model_costs)

# Calculate metrics
df_models['cost_per_request'] = df_models['sum'] / df_models['count']
df_models['percentage'] = (df_models['sum'] / df_models['sum'].sum()) * 100

print("\nModel Cost Comparison:")
print(df_models.to_string(index=False))

# Visualize
plt.figure(figsize=(10, 6))
plt.bar(df_models['model'], df_models['sum'])
plt.title('Total Cost by Model')
plt.xlabel('Model')
plt.ylabel('Total Cost ($)')
plt.xticks(rotation=45)
plt.tight_layout()
plt.savefig('model_costs.png')
```

#### Team Comparison

```python
# Compare costs across teams
team_costs = client.costs.aggregate(
    start_date=start_date,
    end_date=end_date,
    group_by=['team_id'],
    metrics=['sum', 'count', 'avg']
)

df_teams = pd.DataFrame(team_costs)

# Get team names
for idx, row in df_teams.iterrows():
    team = client.teams.get(row['team_id'])
    df_teams.at[idx, 'team_name'] = team.name
    df_teams.at[idx, 'budget'] = team.budget

# Calculate budget utilization
df_teams['utilization'] = (df_teams['sum'] / df_teams['budget']) * 100

# Identify teams over budget
over_budget = df_teams[df_teams['utilization'] > 100]
print(f"\nTeams over budget: {len(over_budget)}")
print(over_budget[['team_name', 'sum', 'budget', 'utilization']])
```

### Trend Analysis

#### Moving Averages

```python
import numpy as np

# Calculate moving averages
df['ma_7'] = df['sum'].rolling(window=7).mean()
df['ma_14'] = df['sum'].rolling(window=14).mean()
df['ma_30'] = df['sum'].rolling(window=30).mean()

# Plot with moving averages
plt.figure(figsize=(14, 7))
plt.plot(df.index, df['sum'], label='Daily Cost', alpha=0.5)
plt.plot(df.index, df['ma_7'], label='7-day MA', linewidth=2)
plt.plot(df.index, df['ma_14'], label='14-day MA', linewidth=2)
plt.plot(df.index, df['ma_30'], label='30-day MA', linewidth=2)
plt.title('Cost Trends with Moving Averages')
plt.xlabel('Date')
plt.ylabel('Cost ($)')
plt.legend()
plt.grid(True, alpha=0.3)
plt.tight_layout()
plt.savefig('cost_trends_ma.png')
```

#### Seasonal Decomposition

```python
from statsmodels.tsa.seasonal import seasonal_decompose

# Ensure regular frequency
df_resampled = df.resample('D').sum().fillna(0)

# Perform seasonal decomposition
decomposition = seasonal_decompose(
    df_resampled['sum'],
    model='additive',
    period=7  # Weekly seasonality
)

# Plot components
fig, axes = plt.subplots(4, 1, figsize=(14, 10))

decomposition.observed.plot(ax=axes[0], title='Observed')
decomposition.trend.plot(ax=axes[1], title='Trend')
decomposition.seasonal.plot(ax=axes[2], title='Seasonal')
decomposition.resid.plot(ax=axes[3], title='Residual')

plt.tight_layout()
plt.savefig('seasonal_decomposition.png')
```

### Anomaly Detection

#### Statistical Anomaly Detection

```python
from scipy import stats

# Calculate z-scores
df['z_score'] = np.abs(stats.zscore(df['sum']))

# Identify anomalies (z-score > 3)
anomalies = df[df['z_score'] > 3]

print(f"\nAnomalies detected: {len(anomalies)}")
print(anomalies[['sum', 'z_score']])

# Visualize anomalies
plt.figure(figsize=(14, 7))
plt.plot(df.index, df['sum'], label='Daily Cost')
plt.scatter(anomalies.index, anomalies['sum'],
           color='red', s=100, label='Anomalies', zorder=5)
plt.title('Cost Anomalies Detection')
plt.xlabel('Date')
plt.ylabel('Cost ($)')
plt.legend()
plt.grid(True, alpha=0.3)
plt.tight_layout()
plt.savefig('anomalies.png')
```

#### Isolation Forest Anomaly Detection

```python
from sklearn.ensemble import IsolationForest

# Prepare features
features = df[['sum', 'count', 'avg']].values

# Train isolation forest
clf = IsolationForest(contamination=0.05, random_state=42)
anomaly_labels = clf.fit_predict(features)

# Add to dataframe
df['anomaly'] = anomaly_labels
anomalies_ml = df[df['anomaly'] == -1]

print(f"\nML-detected anomalies: {len(anomalies_ml)}")
print(anomalies_ml[['sum', 'count', 'avg']])
```

---

## Dashboard Creation and Customization

### Pre-built Dashboards

#### Executive Dashboard

The executive dashboard provides high-level metrics:

- **Total Spend**: Current month, quarter, year
- **Budget Status**: Utilization percentage and remaining budget
- **Top Cost Drivers**: Models, teams, projects
- **Trend Chart**: 90-day cost trend with forecast
- **Key Metrics**: Average cost per request, total requests

#### Team Dashboard

Team-specific cost visibility:

- **Team Spend**: Current period spend
- **Team Budget**: Utilization and variance
- **User Breakdown**: Top users within team
- **Model Usage**: Distribution of model usage
- **Historical Trends**: Team spending over time

#### Model Performance Dashboard

Model-specific analytics:

- **Cost per Model**: Total and per-request costs
- **Usage Volume**: Request counts by model
- **Efficiency Metrics**: Cost per token, tokens per request
- **Comparison**: Model performance comparison
- **Trends**: Usage and cost trends by model

### Custom Dashboard Builder

```python
from llm_cost_ops import DashboardBuilder

# Create dashboard
dashboard = DashboardBuilder(client)

# Add widgets
dashboard.add_widget(
    type='metric',
    title='Total Monthly Cost',
    query={
        'metric': 'sum',
        'field': 'cost',
        'time_range': 'month'
    },
    position={'x': 0, 'y': 0, 'w': 3, 'h': 2}
)

dashboard.add_widget(
    type='line_chart',
    title='Daily Cost Trend',
    query={
        'metrics': ['sum'],
        'group_by': ['date'],
        'time_range': '30d'
    },
    position={'x': 3, 'y': 0, 'w': 9, 'h': 4}
)

dashboard.add_widget(
    type='pie_chart',
    title='Cost by Model',
    query={
        'metric': 'sum',
        'group_by': ['model'],
        'time_range': 'month'
    },
    position={'x': 0, 'y': 2, 'w': 6, 'h': 4}
)

dashboard.add_widget(
    type='table',
    title='Top 10 Users',
    query={
        'metrics': ['sum', 'count'],
        'group_by': ['user_id'],
        'order_by': ['-sum'],
        'limit': 10,
        'time_range': 'month'
    },
    position={'x': 6, 'y': 2, 'w': 6, 'h': 4}
)

# Save dashboard
dashboard.save(
    name='My Custom Dashboard',
    description='Custom analysis dashboard',
    is_public=False,
    tags=['custom', 'analysis']
)

print(f"Dashboard created: {dashboard.id}")
```

### Widget Library

#### Available Widget Types

**Metric Widgets:**
- Single Value: Display single metric (total, average, etc.)
- Comparison: Compare two metrics side-by-side
- Gauge: Visual gauge for budget utilization
- Progress Bar: Progress toward goal

**Chart Widgets:**
- Line Chart: Time-series data
- Bar Chart: Categorical comparisons
- Pie Chart: Distribution analysis
- Area Chart: Stacked time-series
- Scatter Plot: Correlation analysis
- Heatmap: Multi-dimensional data

**Table Widgets:**
- Data Table: Sortable, filterable data tables
- Pivot Table: Multi-dimensional analysis
- Comparison Table: Side-by-side comparisons

**Text Widgets:**
- Markdown: Rich text and documentation
- HTML: Custom HTML content

### Dashboard Sharing

```python
# Share dashboard with team
dashboard.share(
    team_ids=['team-123', 'team-456'],
    permissions='view'
)

# Share with specific users
dashboard.share(
    user_ids=['user-789', 'user-012'],
    permissions='edit'
)

# Make dashboard public
dashboard.make_public()

# Generate shareable link
link = dashboard.get_share_link(expires_in=86400)  # 24 hours
print(f"Share link: {link}")

# Export dashboard configuration
config = dashboard.export_config()
with open('dashboard_config.json', 'w') as f:
    json.dump(config, f, indent=2)

# Import dashboard configuration
new_dashboard = DashboardBuilder.import_config(
    client,
    config_file='dashboard_config.json'
)
```

---

## Report Generation and Scheduling

### Standard Reports

#### Monthly Cost Summary Report

```python
from llm_cost_ops import ReportGenerator

report_gen = ReportGenerator(client)

# Generate monthly summary
report = report_gen.generate(
    type='monthly_summary',
    period='2025-01',
    include_sections=[
        'executive_summary',
        'cost_breakdown',
        'budget_analysis',
        'trends',
        'top_consumers',
        'recommendations'
    ],
    format='pdf'
)

print(f"Report generated: {report.url}")

# Download report
report.download('monthly_summary_2025_01.pdf')
```

#### Budget Variance Report

```python
# Generate variance report
variance_report = report_gen.generate(
    type='budget_variance',
    period='2025-01',
    include_teams=['engineering', 'product', 'data-science'],
    thresholds={
        'warning': 0.8,  # 80% budget utilization
        'critical': 1.0  # 100% budget utilization
    },
    format='xlsx'
)

variance_report.download('budget_variance_2025_01.xlsx')
```

#### Cost Optimization Report

```python
# Generate optimization recommendations
optimization_report = report_gen.generate(
    type='optimization',
    analysis_period='90d',
    include_sections=[
        'cost_trends',
        'inefficiencies',
        'model_recommendations',
        'usage_patterns',
        'potential_savings'
    ],
    format='pdf'
)

print(f"Estimated savings: ${optimization_report.estimated_savings:.2f}/month")
```

### Custom Reports

#### Creating Custom Report Templates

```python
# Define custom report template
template = {
    'name': 'Executive Monthly Report',
    'sections': [
        {
            'type': 'cover_page',
            'title': 'LLM Cost Operations Report',
            'subtitle': 'Monthly Analysis - {{ period }}',
            'logo': 'company_logo.png'
        },
        {
            'type': 'executive_summary',
            'metrics': [
                'total_cost',
                'cost_change',
                'budget_utilization',
                'top_3_models',
                'top_3_teams'
            ]
        },
        {
            'type': 'chart',
            'title': 'Monthly Cost Trend',
            'chart_type': 'line',
            'query': {
                'metrics': ['sum'],
                'group_by': ['month'],
                'time_range': '12m'
            }
        },
        {
            'type': 'table',
            'title': 'Team Cost Breakdown',
            'query': {
                'metrics': ['sum', 'count', 'avg'],
                'group_by': ['team_id'],
                'order_by': ['-sum']
            }
        },
        {
            'type': 'analysis',
            'title': 'Key Insights',
            'auto_generate': True
        },
        {
            'type': 'recommendations',
            'title': 'Action Items',
            'categories': [
                'cost_optimization',
                'usage_efficiency',
                'budget_planning'
            ]
        }
    ],
    'styling': {
        'theme': 'corporate',
        'primary_color': '#1E3A8A',
        'font_family': 'Arial',
        'page_numbers': True,
        'table_of_contents': True
    }
}

# Save template
report_gen.save_template(
    name='executive_monthly',
    template=template
)

# Use template
report = report_gen.generate_from_template(
    template_name='executive_monthly',
    period='2025-01',
    format='pdf'
)
```

### Automated Scheduling

```python
# Schedule daily reports
daily_schedule = report_gen.schedule(
    template_name='daily_summary',
    frequency='daily',
    time='09:00',
    timezone='America/New_York',
    recipients=[
        'manager@example.com',
        'analyst@example.com'
    ],
    format='pdf',
    delivery_method='email'
)

# Schedule weekly reports
weekly_schedule = report_gen.schedule(
    template_name='weekly_analysis',
    frequency='weekly',
    day='Monday',
    time='08:00',
    timezone='America/New_York',
    recipients=['team@example.com'],
    format='xlsx',
    delivery_method='slack',
    slack_channel='#cost-analytics'
)

# Schedule monthly reports
monthly_schedule = report_gen.schedule(
    template_name='executive_monthly',
    frequency='monthly',
    day=1,  # First day of month
    time='10:00',
    timezone='America/New_York',
    recipients=['executives@example.com'],
    format='pdf',
    delivery_method='email',
    include_attachments=['data_export.csv']
)

# List scheduled reports
schedules = report_gen.list_schedules()
for schedule in schedules:
    print(f"{schedule.name}: {schedule.frequency} at {schedule.time}")

# Pause schedule
report_gen.pause_schedule(daily_schedule.id)

# Resume schedule
report_gen.resume_schedule(daily_schedule.id)

# Delete schedule
report_gen.delete_schedule(daily_schedule.id)
```

### Report Distribution

```python
# Email distribution
report.send_email(
    to=['manager@example.com', 'analyst@example.com'],
    cc=['director@example.com'],
    subject='Monthly LLM Cost Report - January 2025',
    body='Please find attached the monthly cost report.',
    attachments=['monthly_summary.pdf']
)

# Slack distribution
report.send_slack(
    channel='#cost-analytics',
    message='Monthly cost report is ready!',
    include_preview=True
)

# S3 upload
report.upload_to_s3(
    bucket='company-reports',
    key='llm-costs/2025/01/monthly_summary.pdf',
    make_public=False
)

# Webhook notification
report.send_webhook(
    url='https://internal.example.com/webhooks/reports',
    payload={
        'report_id': report.id,
        'report_type': 'monthly_summary',
        'period': '2025-01',
        'url': report.url
    }
)
```

---

## Data Visualization Best Practices

### Chart Types

#### When to Use Each Chart Type

**Line Charts:**
- Time-series data
- Trends over time
- Multiple series comparison

```python
# Line chart example
import plotly.graph_objects as go

fig = go.Figure()

fig.add_trace(go.Scatter(
    x=df.index,
    y=df['sum'],
    mode='lines+markers',
    name='Daily Cost',
    line=dict(color='#1E3A8A', width=2),
    marker=dict(size=4)
))

fig.update_layout(
    title='Daily Cost Trend',
    xaxis_title='Date',
    yaxis_title='Cost ($)',
    hovermode='x unified',
    template='plotly_white'
)

fig.show()
```

**Bar Charts:**
- Categorical comparisons
- Ranking data
- Period comparisons

```python
# Bar chart example
fig = go.Figure(data=[
    go.Bar(
        x=df_models['model'],
        y=df_models['sum'],
        marker_color='#1E3A8A',
        text=df_models['sum'].round(2),
        textposition='outside'
    )
])

fig.update_layout(
    title='Cost by Model',
    xaxis_title='Model',
    yaxis_title='Total Cost ($)',
    showlegend=False
)

fig.show()
```

**Pie Charts:**
- Part-to-whole relationships
- Distribution analysis
- Limited categories (5-7 max)

```python
# Pie chart example
fig = go.Figure(data=[go.Pie(
    labels=df_models['model'],
    values=df_models['sum'],
    hole=0.3,  # Donut chart
    marker=dict(colors=['#1E3A8A', '#3B82F6', '#60A5FA', '#93C5FD'])
)])

fig.update_layout(
    title='Cost Distribution by Model'
)

fig.show()
```

**Heatmaps:**
- Multi-dimensional data
- Correlation analysis
- Pattern identification

```python
# Heatmap example
import seaborn as sns

# Pivot data
pivot_data = df.pivot_table(
    values='sum',
    index=df.index.hour,
    columns=df.index.dayofweek,
    aggfunc='mean'
)

plt.figure(figsize=(12, 8))
sns.heatmap(
    pivot_data,
    annot=True,
    fmt='.2f',
    cmap='Blues',
    cbar_kws={'label': 'Average Cost ($)'}
)
plt.title('Cost Heatmap by Hour and Day of Week')
plt.xlabel('Day of Week')
plt.ylabel('Hour of Day')
plt.tight_layout()
plt.savefig('cost_heatmap.png')
```

### Color Schemes

#### Recommended Color Palettes

```python
# Corporate color scheme
COLORS = {
    'primary': '#1E3A8A',      # Dark Blue
    'secondary': '#3B82F6',    # Blue
    'accent': '#10B981',       # Green
    'warning': '#F59E0B',      # Amber
    'danger': '#EF4444',       # Red
    'neutral': '#6B7280'       # Gray
}

# Sequential colors (for gradients)
SEQUENTIAL = [
    '#EFF6FF',  # Very light blue
    '#DBEAFE',
    '#BFDBFE',
    '#93C5FD',
    '#60A5FA',
    '#3B82F6',
    '#2563EB',
    '#1D4ED8',
    '#1E40AF',
    '#1E3A8A'   # Very dark blue
]

# Diverging colors (for comparisons)
DIVERGING = [
    '#EF4444',  # Red (negative)
    '#F59E0B',  # Amber
    '#6B7280',  # Gray (neutral)
    '#10B981',  # Green
    '#059669'   # Dark green (positive)
]
```

### Interactive Elements

```python
# Interactive dashboard with Plotly Dash
import dash
from dash import dcc, html
from dash.dependencies import Input, Output

app = dash.Dash(__name__)

app.layout = html.Div([
    html.H1('LLM Cost Analytics Dashboard'),

    dcc.DatePickerRange(
        id='date-range',
        start_date=start_date,
        end_date=end_date
    ),

    dcc.Dropdown(
        id='model-filter',
        options=[
            {'label': 'All Models', 'value': 'all'},
            {'label': 'GPT-4', 'value': 'gpt-4'},
            {'label': 'GPT-3.5', 'value': 'gpt-3.5-turbo'},
            {'label': 'Claude', 'value': 'claude-2'}
        ],
        value='all'
    ),

    dcc.Graph(id='cost-trend'),
    dcc.Graph(id='model-distribution')
])

@app.callback(
    [Output('cost-trend', 'figure'),
     Output('model-distribution', 'figure')],
    [Input('date-range', 'start_date'),
     Input('date-range', 'end_date'),
     Input('model-filter', 'value')]
)
def update_graphs(start_date, end_date, model):
    # Fetch data based on filters
    costs = client.costs.aggregate(
        start_date=start_date,
        end_date=end_date,
        filters={'model': model} if model != 'all' else {},
        group_by=['date', 'model'],
        metrics=['sum']
    )

    df = pd.DataFrame(costs)

    # Create cost trend chart
    trend_fig = go.Figure()
    for model in df['model'].unique():
        model_data = df[df['model'] == model]
        trend_fig.add_trace(go.Scatter(
            x=model_data['date'],
            y=model_data['sum'],
            name=model,
            mode='lines+markers'
        ))

    # Create distribution chart
    dist_df = df.groupby('model')['sum'].sum().reset_index()
    dist_fig = go.Figure(data=[go.Pie(
        labels=dist_df['model'],
        values=dist_df['sum']
    )])

    return trend_fig, dist_fig

if __name__ == '__main__':
    app.run_server(debug=True)
```

---

## Forecasting and Predictive Analysis

### Cost Forecasting Models

#### Linear Regression Forecast

```python
from sklearn.linear_model import LinearRegression
from sklearn.model_selection import train_test_split
import numpy as np

# Prepare data
df_forecast = df.copy()
df_forecast['day_num'] = (df_forecast.index - df_forecast.index[0]).days
X = df_forecast[['day_num']].values
y = df_forecast['sum'].values

# Split data
X_train, X_test, y_train, y_test = train_test_split(
    X, y, test_size=0.2, shuffle=False
)

# Train model
model = LinearRegression()
model.fit(X_train, y_train)

# Predict next 30 days
last_day = df_forecast['day_num'].max()
future_days = np.array([[i] for i in range(last_day + 1, last_day + 31)])
forecast = model.predict(future_days)

print(f"Forecasted cost for next 30 days: ${forecast.sum():.2f}")
print(f"Average daily forecast: ${forecast.mean():.2f}")
```

#### ARIMA Forecast

```python
from statsmodels.tsa.arima.model import ARIMA

# Fit ARIMA model
arima_model = ARIMA(df['sum'], order=(5, 1, 0))
arima_fit = arima_model.fit()

# Forecast next 30 days
forecast = arima_fit.forecast(steps=30)

# Plot forecast
plt.figure(figsize=(14, 7))
plt.plot(df.index, df['sum'], label='Historical')
plt.plot(
    pd.date_range(start=df.index[-1], periods=31)[1:],
    forecast,
    label='Forecast',
    color='red',
    linestyle='--'
)
plt.title('Cost Forecast - Next 30 Days')
plt.xlabel('Date')
plt.ylabel('Cost ($)')
plt.legend()
plt.grid(True, alpha=0.3)
plt.tight_layout()
plt.savefig('cost_forecast.png')

print(f"30-day forecast: ${forecast.sum():.2f}")
```

#### Prophet Forecast

```python
from prophet import Prophet

# Prepare data for Prophet
df_prophet = df.reset_index()
df_prophet.columns = ['ds', 'y']

# Initialize and fit model
prophet_model = Prophet(
    changepoint_prior_scale=0.05,
    seasonality_mode='multiplicative'
)
prophet_model.fit(df_prophet)

# Create future dataframe
future = prophet_model.make_future_dataframe(periods=30)
forecast = prophet_model.predict(future)

# Plot forecast
fig = prophet_model.plot(forecast)
plt.title('Cost Forecast with Prophet')
plt.savefig('prophet_forecast.png')

# Plot components
fig_components = prophet_model.plot_components(forecast)
plt.savefig('prophet_components.png')

# Get forecast summary
forecast_30d = forecast.tail(30)
print(f"30-day forecast: ${forecast_30d['yhat'].sum():.2f}")
print(f"Lower bound: ${forecast_30d['yhat_lower'].sum():.2f}")
print(f"Upper bound: ${forecast_30d['yhat_upper'].sum():.2f}")
```

### Budget Projections

```python
# Project budget requirements
current_monthly_cost = df.tail(30)['sum'].sum()
forecasted_monthly_cost = forecast.sum()

# Calculate required budget
growth_rate = (forecasted_monthly_cost - current_monthly_cost) / current_monthly_cost
recommended_budget = forecasted_monthly_cost * 1.2  # 20% buffer

print(f"Current monthly cost: ${current_monthly_cost:.2f}")
print(f"Forecasted monthly cost: ${forecasted_monthly_cost:.2f}")
print(f"Growth rate: {growth_rate*100:.1f}%")
print(f"Recommended budget: ${recommended_budget:.2f}")

# Quarterly projections
quarterly_forecast = {
    'Q1': forecast[:90].sum(),
    'Q2': forecast[90:180].sum(),
    'Q3': forecast[180:270].sum(),
    'Q4': forecast[270:].sum()
}

print("\nQuarterly Projections:")
for quarter, amount in quarterly_forecast.items():
    print(f"{quarter}: ${amount:.2f}")
```

### Capacity Planning

```python
# Analyze usage patterns for capacity planning
usage_stats = client.costs.aggregate(
    start_date=start_date,
    end_date=end_date,
    group_by=['model'],
    metrics=['count', 'sum', 'avg', 'max']
)

df_usage = pd.DataFrame(usage_stats)

# Calculate capacity metrics
df_usage['requests_per_day'] = df_usage['count'] / 30
df_usage['peak_capacity_needed'] = df_usage['max'] * 1.5  # 50% buffer

# Project future capacity needs
df_usage['projected_requests_per_day'] = df_usage['requests_per_day'] * (1 + growth_rate)
df_usage['projected_capacity'] = df_usage['peak_capacity_needed'] * (1 + growth_rate)

print("\nCapacity Planning:")
print(df_usage[['model', 'requests_per_day', 'projected_requests_per_day', 'projected_capacity']])
```

---

## Budget Tracking and Variance Analysis

### Budget Setup

```python
# Create team budgets
engineering_budget = client.budgets.create(
    name='Engineering - Q1 2025',
    amount=50000,
    period='quarter',
    start_date='2025-01-01',
    end_date='2025-03-31',
    team_id='engineering',
    alert_thresholds=[0.5, 0.75, 0.9, 1.0]
)

# Create model-specific budgets
gpt4_budget = client.budgets.create(
    name='GPT-4 - January 2025',
    amount=10000,
    period='month',
    start_date='2025-01-01',
    end_date='2025-01-31',
    filters={'model': 'gpt-4'},
    alert_thresholds=[0.8, 1.0]
)
```

### Variance Reporting

```python
# Get budget variance
def analyze_budget_variance(budget_id):
    budget = client.budgets.get(budget_id)
    actual = budget.current_spend
    planned = budget.amount
    variance = actual - planned
    variance_pct = (variance / planned) * 100

    return {
        'budget_name': budget.name,
        'planned': planned,
        'actual': actual,
        'variance': variance,
        'variance_pct': variance_pct,
        'utilization': budget.utilization,
        'status': 'over' if variance > 0 else 'under',
        'remaining': budget.remaining
    }

# Analyze all budgets
budgets = client.budgets.list()
variance_report = []

for budget in budgets:
    variance = analyze_budget_variance(budget.id)
    variance_report.append(variance)

df_variance = pd.DataFrame(variance_report)

# Display variance report
print("\nBudget Variance Report:")
print(df_variance.to_string(index=False))

# Visualize variance
fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 6))

# Variance chart
colors = ['red' if v > 0 else 'green' for v in df_variance['variance']]
ax1.barh(df_variance['budget_name'], df_variance['variance'], color=colors)
ax1.set_xlabel('Variance ($)')
ax1.set_title('Budget Variance')
ax1.axvline(x=0, color='black', linestyle='--', linewidth=0.5)

# Utilization chart
ax2.barh(df_variance['budget_name'], df_variance['utilization'])
ax2.set_xlabel('Utilization (%)')
ax2.set_title('Budget Utilization')
ax2.axvline(x=100, color='red', linestyle='--', linewidth=1)

plt.tight_layout()
plt.savefig('budget_variance.png')
```

### Alert Configuration

```python
# Configure budget alerts
alert_config = client.alerts.create(
    name='Budget Warning - Engineering',
    type='budget',
    conditions={
        'budget_id': engineering_budget.id,
        'threshold': 0.8,  # 80% utilization
        'comparison': 'greater_than'
    },
    actions=[
        {
            'type': 'email',
            'recipients': ['manager@example.com', 'finance@example.com']
        },
        {
            'type': 'slack',
            'channel': '#cost-alerts'
        }
    ],
    severity': 'warning'
)

# Configure anomaly alerts
anomaly_alert = client.alerts.create(
    name='Cost Anomaly Detection',
    type='anomaly',
    conditions={
        'metric': 'daily_cost',
        'threshold': 3.0,  # 3 standard deviations
        'window': '7d'
    },
    actions=[
        {
            'type': 'email',
            'recipients': ['analyst@example.com']
        }
    ],
    severity: 'high'
)
```

---

## Cost Allocation Strategies

### Team-Based Allocation

```python
# Allocate costs by team
team_allocation = client.costs.aggregate(
    start_date=start_date,
    end_date=end_date,
    group_by=['team_id'],
    metrics=['sum', 'count']
)

df_teams = pd.DataFrame(team_allocation)

# Calculate allocation percentages
total_cost = df_teams['sum'].sum()
df_teams['allocation_pct'] = (df_teams['sum'] / total_cost) * 100

# Get team details
for idx, row in df_teams.iterrows():
    team = client.teams.get(row['team_id'])
    df_teams.at[idx, 'team_name'] = team.name
    df_teams.at[idx, 'department'] = team.metadata.get('department', 'N/A')

print("\nTeam Cost Allocation:")
print(df_teams[['team_name', 'department', 'sum', 'allocation_pct']])
```

### Project-Based Allocation

```python
# Allocate costs by project
project_allocation = client.costs.aggregate(
    start_date=start_date,
    end_date=end_date,
    group_by=['project_id'],
    metrics=['sum', 'count', 'avg']
)

df_projects = pd.DataFrame(project_allocation)

# Calculate ROI metrics
for idx, row in df_projects.iterrows():
    project = client.projects.get(row['project_id'])
    df_projects.at[idx, 'project_name'] = project.name
    df_projects.at[idx, 'revenue'] = project.revenue
    df_projects.at[idx, 'roi'] = (project.revenue - row['sum']) / row['sum'] * 100

print("\nProject Cost Allocation with ROI:")
print(df_projects[['project_name', 'sum', 'revenue', 'roi']])
```

### Tag-Based Allocation

```python
# Allocate costs by tags
tag_allocation = client.costs.aggregate(
    start_date=start_date,
    end_date=end_date,
    group_by=['tags'],
    metrics=['sum', 'count']
)

# Common tags: environment, feature, customer, product
df_tags = pd.DataFrame(tag_allocation)

# Pivot by tag type
for tag_type in ['environment', 'feature', 'customer']:
    tag_data = df_tags[df_tags['tags'].str.contains(tag_type)]
    print(f"\nAllocation by {tag_type}:")
    print(tag_data[['tags', 'sum', 'count']])
```

---

## Chargeback and Showback

### Chargeback Models

```python
# Implement chargeback system
class ChargebackCalculator:
    def __init__(self, client):
        self.client = client

    def calculate_team_chargeback(self, team_id, period_start, period_end):
        """Calculate chargeback for a team."""
        costs = self.client.costs.aggregate(
            start_date=period_start,
            end_date=period_end,
            filters={'team_id': team_id},
            metrics=['sum', 'count']
        )

        # Apply overhead multiplier (e.g., 20% for platform costs)
        overhead_multiplier = 1.2
        total_chargeback = costs[0]['sum'] * overhead_multiplier

        return {
            'team_id': team_id,
            'period': f"{period_start} to {period_end}",
            'direct_costs': costs[0]['sum'],
            'overhead_rate': overhead_multiplier,
            'total_chargeback': total_chargeback,
            'request_count': costs[0]['count']
        }

    def generate_chargeback_report(self, period_start, period_end):
        """Generate chargeback report for all teams."""
        teams = self.client.teams.list()
        report = []

        for team in teams:
            chargeback = self.calculate_team_chargeback(
                team.id,
                period_start,
                period_end
            )
            report.append(chargeback)

        return pd.DataFrame(report)

# Usage
calculator = ChargebackCalculator(client)
chargeback_report = calculator.generate_chargeback_report(
    period_start='2025-01-01',
    period_end='2025-01-31'
)

print("\nMonthly Chargeback Report:")
print(chargeback_report.to_string(index=False))
```

### Showback Reports

```python
# Generate showback report (informational only)
showback_report = calculator.generate_chargeback_report(
    period_start='2025-01-01',
    period_end='2025-01-31'
)

# Add additional context
showback_report['avg_cost_per_request'] = (
    showback_report['direct_costs'] / showback_report['request_count']
)

# Generate visualization
fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 6))

# Cost by team
ax1.barh(showback_report['team_id'], showback_report['total_chargeback'])
ax1.set_xlabel('Chargeback Amount ($)')
ax1.set_title('Team Chargeback Summary')

# Cost per request
ax2.barh(showback_report['team_id'], showback_report['avg_cost_per_request'])
ax2.set_xlabel('Average Cost per Request ($)')
ax2.set_title('Cost Efficiency by Team')

plt.tight_layout()
plt.savefig('showback_report.png')

# Export to Excel
showback_report.to_excel('showback_report_2025_01.xlsx', index=False)
```

### Cost Center Reporting

```python
# Map teams to cost centers
cost_center_mapping = {
    'engineering': 'CC-100',
    'product': 'CC-200',
    'data-science': 'CC-300',
    'customer-success': 'CC-400'
}

# Aggregate by cost center
cost_center_report = []

for team_id, cost_center in cost_center_mapping.items():
    costs = client.costs.aggregate(
        start_date='2025-01-01',
        end_date='2025-01-31',
        filters={'team_id': team_id},
        metrics=['sum']
    )

    cost_center_report.append({
        'cost_center': cost_center,
        'team': team_id,
        'amount': costs[0]['sum']
    })

df_cost_centers = pd.DataFrame(cost_center_report)

# Export for financial system
df_cost_centers.to_csv('cost_center_report_2025_01.csv', index=False)

print("\nCost Center Report:")
print(df_cost_centers.to_string(index=False))
```

---

## Data Export and Integration

### Export Formats

#### CSV Export

```python
# Export cost data to CSV
costs = client.costs.list(
    start_date='2025-01-01',
    end_date='2025-01-31',
    limit=10000
)

df_export = pd.DataFrame([cost.to_dict() for cost in costs])
df_export.to_csv('costs_2025_01.csv', index=False)

print(f"Exported {len(df_export)} records to CSV")
```

#### Excel Export with Multiple Sheets

```python
# Create Excel workbook with multiple sheets
with pd.ExcelWriter('cost_analysis_2025_01.xlsx', engine='xlsxwriter') as writer:
    # Summary sheet
    summary = df.groupby('model')['sum'].agg(['sum', 'mean', 'count'])
    summary.to_excel(writer, sheet_name='Summary')

    # Daily costs sheet
    df.to_excel(writer, sheet_name='Daily Costs')

    # Team breakdown sheet
    df_teams.to_excel(writer, sheet_name='Team Breakdown')

    # Budget variance sheet
    df_variance.to_excel(writer, sheet_name='Budget Variance')

print("Excel export complete")
```

#### Parquet Export

```python
# Export to Parquet for big data processing
df_export.to_parquet(
    'costs_2025_01.parquet',
    compression='snappy',
    index=False
)

print(f"Exported to Parquet: {os.path.getsize('costs_2025_01.parquet')} bytes")
```

#### JSON Export

```python
# Export to JSON
costs_json = [cost.to_dict() for cost in costs]

with open('costs_2025_01.json', 'w') as f:
    json.dump(costs_json, f, indent=2, default=str)

print("JSON export complete")
```

### API Access

```python
# Programmatic data access
from llm_cost_ops import CostOpsClient

client = CostOpsClient(api_key="your-api-key")

# Fetch data with pagination
def fetch_all_costs(start_date, end_date, page_size=1000):
    """Fetch all costs with pagination."""
    all_costs = []
    offset = 0

    while True:
        costs = client.costs.list(
            start_date=start_date,
            end_date=end_date,
            limit=page_size,
            offset=offset
        )

        if not costs:
            break

        all_costs.extend(costs)
        offset += page_size

        print(f"Fetched {len(all_costs)} records...")

    return all_costs

# Usage
all_costs = fetch_all_costs('2025-01-01', '2025-01-31')
df_all = pd.DataFrame([cost.to_dict() for cost in all_costs])
```

### BI Tool Integration

#### Tableau Integration

```python
# Prepare data for Tableau
# Export as Hyper file for optimal performance

from tableauhyperapi import HyperProcess, Connection, TableDefinition, \
    SqlType, Inserter, CreateMode, TableName

# Define table schema
table_def = TableDefinition(
    table_name=TableName('Extract', 'Costs'),
    columns=[
        TableDefinition.Column('id', SqlType.text()),
        TableDefinition.Column('date', SqlType.date()),
        TableDefinition.Column('model', SqlType.text()),
        TableDefinition.Column('cost', SqlType.double()),
        TableDefinition.Column('tokens_used', SqlType.big_int()),
        TableDefinition.Column('user_id', SqlType.text()),
        TableDefinition.Column('team_id', SqlType.text())
    ]
)

# Create Hyper file
with HyperProcess(Telemetry.SEND_USAGE_DATA_TO_TABLEAU) as hyper:
    with Connection(
        hyper.endpoint,
        'costs.hyper',
        CreateMode.CREATE_AND_REPLACE
    ) as connection:
        connection.catalog.create_table(table_def)

        with Inserter(connection, table_def) as inserter:
            for _, row in df_export.iterrows():
                inserter.add_row([
                    row['id'],
                    row['date'],
                    row['model'],
                    row['cost'],
                    row['tokens_used'],
                    row['user_id'],
                    row['team_id']
                ])
            inserter.execute()

print("Tableau Hyper file created")
```

#### Power BI Integration

```python
# Create Power BI dataset via API
import requests

powerbi_api_url = "https://api.powerbi.com/v1.0/myorg/datasets"
headers = {
    "Authorization": f"Bearer {powerbi_token}",
    "Content-Type": "application/json"
}

# Define dataset schema
dataset_def = {
    "name": "LLM Cost Analysis",
    "tables": [
        {
            "name": "Costs",
            "columns": [
                {"name": "id", "dataType": "String"},
                {"name": "date", "dataType": "DateTime"},
                {"name": "model", "dataType": "String"},
                {"name": "cost", "dataType": "Double"},
                {"name": "tokens_used", "dataType": "Int64"},
                {"name": "user_id", "dataType": "String"},
                {"name": "team_id", "dataType": "String"}
            ]
        }
    ]
}

# Create dataset
response = requests.post(powerbi_api_url, headers=headers, json=dataset_def)
dataset_id = response.json()['id']

# Push data
rows_url = f"{powerbi_api_url}/{dataset_id}/tables/Costs/rows"
rows_data = df_export.to_dict('records')

requests.post(rows_url, headers=headers, json={"rows": rows_data})

print(f"Power BI dataset created: {dataset_id}")
```

#### Looker Integration

```python
# Create Looker view
looker_view = """
view: llm_costs {
  sql_table_name: public.costs ;;

  dimension: id {
    primary_key: yes
    type: string
    sql: ${TABLE}.id ;;
  }

  dimension_group: created {
    type: time
    timeframes: [date, week, month, quarter, year]
    sql: ${TABLE}.created_at ;;
  }

  dimension: model {
    type: string
    sql: ${TABLE}.model ;;
  }

  dimension: team_id {
    type: string
    sql: ${TABLE}.team_id ;;
  }

  measure: total_cost {
    type: sum
    sql: ${TABLE}.cost ;;
    value_format_name: usd
  }

  measure: average_cost {
    type: average
    sql: ${TABLE}.cost ;;
    value_format_name: usd
  }

  measure: total_tokens {
    type: sum
    sql: ${TABLE}.tokens_used ;;
  }

  measure: request_count {
    type: count
    drill_fields: [id, created_date, model, team_id]
  }

  measure: cost_per_request {
    type: number
    sql: ${total_cost} / NULLIF(${request_count}, 0) ;;
    value_format_name: usd
  }
}
"""

with open('llm_costs.view.lkml', 'w') as f:
    f.write(looker_view)

print("Looker view created")
```

---

## Advanced SQL Queries

### Common Query Patterns

```sql
-- Daily cost aggregation
SELECT
    DATE(created_at) as date,
    SUM(cost) as total_cost,
    COUNT(*) as request_count,
    AVG(cost) as avg_cost,
    MAX(cost) as max_cost
FROM costs
WHERE created_at >= '2025-01-01'
GROUP BY DATE(created_at)
ORDER BY date;

-- Cost by model and user
SELECT
    model,
    user_id,
    COUNT(*) as requests,
    SUM(cost) as total_cost,
    SUM(tokens_used) as total_tokens,
    AVG(cost) as avg_cost_per_request
FROM costs
WHERE created_at >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY model, user_id
ORDER BY total_cost DESC
LIMIT 20;

-- Moving average
SELECT
    DATE(created_at) as date,
    SUM(cost) as daily_cost,
    AVG(SUM(cost)) OVER (
        ORDER BY DATE(created_at)
        ROWS BETWEEN 6 PRECEDING AND CURRENT ROW
    ) as ma_7day
FROM costs
WHERE created_at >= '2025-01-01'
GROUP BY DATE(created_at)
ORDER BY date;

-- Cumulative cost
SELECT
    DATE(created_at) as date,
    SUM(cost) as daily_cost,
    SUM(SUM(cost)) OVER (
        ORDER BY DATE(created_at)
    ) as cumulative_cost
FROM costs
WHERE created_at >= '2025-01-01'
GROUP BY DATE(created_at)
ORDER BY date;

-- Cost percentiles
SELECT
    model,
    PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY cost) as median_cost,
    PERCENTILE_CONT(0.75) WITHIN GROUP (ORDER BY cost) as p75_cost,
    PERCENTILE_CONT(0.90) WITHIN GROUP (ORDER BY cost) as p90_cost,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY cost) as p95_cost,
    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY cost) as p99_cost
FROM costs
WHERE created_at >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY model;

-- Team budget utilization
SELECT
    t.name as team_name,
    b.amount as budget,
    COALESCE(SUM(c.cost), 0) as spent,
    b.amount - COALESCE(SUM(c.cost), 0) as remaining,
    (COALESCE(SUM(c.cost), 0) / b.amount) * 100 as utilization_pct
FROM teams t
LEFT JOIN budgets b ON t.id = b.team_id
LEFT JOIN costs c ON t.id = c.team_id
    AND c.created_at >= b.start_date
    AND c.created_at <= b.end_date
WHERE b.period = 'month'
    AND b.start_date >= '2025-01-01'
GROUP BY t.name, b.amount
ORDER BY utilization_pct DESC;
```

### Custom Analytics

```sql
-- Cohort analysis
WITH user_first_request AS (
    SELECT
        user_id,
        DATE_TRUNC('month', MIN(created_at)) as cohort_month
    FROM costs
    GROUP BY user_id
),
monthly_activity AS (
    SELECT
        c.user_id,
        DATE_TRUNC('month', c.created_at) as activity_month,
        SUM(c.cost) as monthly_spend
    FROM costs c
    GROUP BY c.user_id, DATE_TRUNC('month', c.created_at)
)
SELECT
    ufr.cohort_month,
    ma.activity_month,
    EXTRACT(MONTH FROM AGE(ma.activity_month, ufr.cohort_month)) as months_since_cohort,
    COUNT(DISTINCT ma.user_id) as active_users,
    SUM(ma.monthly_spend) as total_spend,
    AVG(ma.monthly_spend) as avg_spend_per_user
FROM user_first_request ufr
JOIN monthly_activity ma ON ufr.user_id = ma.user_id
GROUP BY ufr.cohort_month, ma.activity_month
ORDER BY ufr.cohort_month, ma.activity_month;

-- Anomaly detection with z-score
WITH daily_stats AS (
    SELECT
        DATE(created_at) as date,
        SUM(cost) as daily_cost
    FROM costs
    WHERE created_at >= CURRENT_DATE - INTERVAL '90 days'
    GROUP BY DATE(created_at)
),
cost_stats AS (
    SELECT
        AVG(daily_cost) as mean_cost,
        STDDEV(daily_cost) as stddev_cost
    FROM daily_stats
)
SELECT
    ds.date,
    ds.daily_cost,
    cs.mean_cost,
    cs.stddev_cost,
    (ds.daily_cost - cs.mean_cost) / cs.stddev_cost as z_score,
    CASE
        WHEN ABS((ds.daily_cost - cs.mean_cost) / cs.stddev_cost) > 3 THEN 'Anomaly'
        ELSE 'Normal'
    END as status
FROM daily_stats ds
CROSS JOIN cost_stats cs
ORDER BY ds.date DESC;
```

---

## Key Performance Indicators (KPIs)

### Cost Metrics

```python
# Calculate key cost metrics
def calculate_cost_kpis(start_date, end_date):
    costs = client.costs.aggregate(
        start_date=start_date,
        end_date=end_date,
        metrics=['sum', 'count', 'avg', 'min', 'max']
    )

    kpis = {
        'total_cost': costs[0]['sum'],
        'total_requests': costs[0]['count'],
        'avg_cost_per_request': costs[0]['avg'],
        'min_cost': costs[0]['min'],
        'max_cost': costs[0]['max'],
        'cost_std_dev': costs[0].get('stddev', 0)
    }

    # Calculate additional metrics
    previous_period_start = start_date - (end_date - start_date)
    previous_costs = client.costs.aggregate(
        start_date=previous_period_start,
        end_date=start_date,
        metrics=['sum']
    )

    kpis['cost_change'] = kpis['total_cost'] - previous_costs[0]['sum']
    kpis['cost_change_pct'] = (kpis['cost_change'] / previous_costs[0]['sum']) * 100

    return kpis

kpis = calculate_cost_kpis(
    start_date=datetime(2025, 1, 1),
    end_date=datetime(2025, 1, 31)
)

print("\nCost KPIs:")
for key, value in kpis.items():
    if 'pct' in key:
        print(f"{key}: {value:.1f}%")
    elif isinstance(value, float):
        print(f"{key}: ${value:.2f}")
    else:
        print(f"{key}: {value}")
```

### Efficiency Metrics

```python
# Calculate efficiency metrics
efficiency_metrics = {
    'cost_per_1k_tokens': (kpis['total_cost'] / total_tokens) * 1000,
    'tokens_per_request': total_tokens / kpis['total_requests'],
    'cost_efficiency_ratio': kpis['avg_cost_per_request'] / industry_benchmark,
    'resource_utilization': (actual_usage / allocated_budget) * 100
}

print("\nEfficiency Metrics:")
for key, value in efficiency_metrics.items():
    print(f"{key}: {value:.2f}")
```

### Usage Metrics

```python
# Calculate usage metrics
usage_metrics = client.costs.aggregate(
    start_date=start_date,
    end_date=end_date,
    group_by=['model'],
    metrics=['count', 'sum']
)

df_usage = pd.DataFrame(usage_metrics)
df_usage['market_share'] = (df_usage['count'] / df_usage['count'].sum()) * 100
df_usage['cost_share'] = (df_usage['sum'] / df_usage['sum'].sum()) * 100

print("\nUsage Metrics by Model:")
print(df_usage[['model', 'count', 'market_share', 'cost_share']])
```

---

## Cost Optimization

### Identifying Opportunities

```python
# Identify cost optimization opportunities
def identify_optimization_opportunities():
    opportunities = []

    # High-cost users
    high_cost_users = client.costs.aggregate(
        start_date=datetime.now() - timedelta(days=30),
        end_date=datetime.now(),
        group_by=['user_id'],
        metrics=['sum', 'count'],
        order_by=['-sum'],
        limit=10
    )

    for user in high_cost_users:
        if user['sum'] > 1000:  # Threshold
            opportunities.append({
                'type': 'high_cost_user',
                'user_id': user['user_id'],
                'cost': user['sum'],
                'recommendation': 'Review usage patterns and optimize queries'
            })

    # Model inefficiencies
    model_efficiency = client.costs.aggregate(
        start_date=datetime.now() - timedelta(days=30),
        end_date=datetime.now(),
        group_by=['model'],
        metrics=['sum', 'count', 'avg']
    )

    for model in model_efficiency:
        cost_per_request = model['sum'] / model['count']
        if model['model'] == 'gpt-4' and cost_per_request > 0.05:
            opportunities.append({
                'type': 'model_optimization',
                'model': model['model'],
                'current_cost_per_request': cost_per_request,
                'recommendation': 'Consider using GPT-3.5-turbo for simpler tasks'
            })

    return pd.DataFrame(opportunities)

optimization_opps = identify_optimization_opportunities()
print("\nOptimization Opportunities:")
print(optimization_opps.to_string(index=False))
```

### Optimization Recommendations

```python
# Generate optimization recommendations
recommendations = {
    'model_switching': {
        'current': 'GPT-4 for all tasks',
        'recommended': 'GPT-3.5-turbo for simple tasks, GPT-4 for complex',
        'estimated_savings': 5000,
        'implementation_effort': 'Medium'
    },
    'caching': {
        'current': 'No caching',
        'recommended': 'Implement response caching for common queries',
        'estimated_savings': 2000,
        'implementation_effort': 'Low'
    },
    'batch_processing': {
        'current': 'Individual requests',
        'recommended': 'Batch similar requests',
        'estimated_savings': 1500,
        'implementation_effort': 'High'
    }
}

# Calculate total potential savings
total_savings = sum(r['estimated_savings'] for r in recommendations.values())
print(f"\nTotal Potential Monthly Savings: ${total_savings:.2f}")
```

---

## Best Practices

1. **Regular Review**: Review cost reports weekly
2. **Budget Monitoring**: Set up automated alerts at 50%, 75%, and 90% thresholds
3. **Trend Analysis**: Monitor 7-day and 30-day moving averages
4. **Anomaly Detection**: Investigate daily anomalies promptly
5. **Model Optimization**: Regularly evaluate model performance vs. cost
6. **Documentation**: Document all analysis methodologies
7. **Stakeholder Communication**: Share insights with relevant teams monthly
8. **Forecasting**: Update forecasts quarterly
9. **Continuous Improvement**: Iterate on optimization strategies

---

## Conclusion

This analyst guide provides comprehensive tools and techniques for analyzing, reporting, and optimizing LLM costs. For additional support:

- Documentation: https://docs.llmcostops.com
- Community: https://community.llmcostops.com
- Support: support@llmcostops.com
