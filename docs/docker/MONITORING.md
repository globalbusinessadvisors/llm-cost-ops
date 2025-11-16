# Monitoring and Observability Guide for LLM Cost Ops

**Comprehensive Monitoring, Metrics, Logging, and Tracing**

Version: 1.0.0
Last Updated: 2025-11-16

---

## Table of Contents

1. [Overview](#overview)
2. [Metrics Collection](#metrics-collection)
3. [Logging Setup](#logging-setup)
4. [Distributed Tracing](#distributed-tracing)
5. [Dashboards](#dashboards)
6. [Alerting](#alerting)
7. [Performance Monitoring](#performance-monitoring)
8. [Debugging](#debugging)
9. [Best Practices](#best-practices)
10. [Troubleshooting](#troubleshooting)

---

## Overview

### Observability Stack

The platform uses a comprehensive observability stack:

```
┌─────────────────────────────────────────────────────────┐
│                   Application                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Metrics    │  │     Logs     │  │    Traces    │  │
│  │   (OpenTel)  │  │  (Structured)│  │   (Jaeger)   │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
└─────────┼──────────────────┼──────────────────┼─────────┘
          │                  │                  │
          ▼                  ▼                  ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│   Prometheus    │  │      Loki       │  │     Jaeger      │
│   (Metrics)     │  │    (Logs)       │  │    (Traces)     │
└────────┬────────┘  └────────┬────────┘  └────────┬────────┘
         │                    │                    │
         └────────────────────┼────────────────────┘
                              ▼
                     ┌─────────────────┐
                     │     Grafana     │
                     │  (Dashboards)   │
                     └─────────────────┘
```

### Three Pillars of Observability

**1. Metrics** - What is happening?
- Request rates, error rates, duration
- Resource utilization (CPU, memory, disk)
- Business metrics (costs, usage, revenue)

**2. Logs** - Why did it happen?
- Application events
- Error messages
- Audit trails

**3. Traces** - Where did it happen?
- Request flow across services
- Performance bottlenecks
- Error propagation

---

## Metrics Collection

### Prometheus Configuration

**prometheus.yml:**

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'llm-cost-ops'
    environment: 'production'

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

rule_files:
  - /etc/prometheus/alerts.yml
  - /etc/prometheus/recording-rules.yml

scrape_configs:
  # Prometheus itself
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  # LLM Cost Ops application
  - job_name: 'llm-cost-ops'
    static_configs:
      - targets: ['app:9090']
    metrics_path: '/metrics'
    scrape_interval: 10s
    scrape_timeout: 5s

  # PostgreSQL
  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  # Redis
  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']

  # NATS
  - job_name: 'nats'
    static_configs:
      - targets: ['nats:8222']
    metrics_path: '/metrics'

  # Node metrics
  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']

  # Docker containers
  - job_name: 'cadvisor'
    static_configs:
      - targets: ['cadvisor:8080']
```

### Application Metrics

**Rust implementation:**

```rust
use prometheus::{
    Counter, Histogram, Gauge, Registry,
    opts, histogram_opts,
};
use lazy_static::lazy_static;

lazy_static! {
    // HTTP request counter
    pub static ref HTTP_REQUESTS_TOTAL: Counter = Counter::new(
        "http_requests_total",
        "Total number of HTTP requests"
    ).unwrap();

    // HTTP request duration histogram
    pub static ref HTTP_REQUEST_DURATION: Histogram = Histogram::with_opts(
        histogram_opts!(
            "http_request_duration_seconds",
            "HTTP request duration in seconds",
            vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]
        )
    ).unwrap();

    // Active connections gauge
    pub static ref ACTIVE_CONNECTIONS: Gauge = Gauge::new(
        "active_connections",
        "Number of active connections"
    ).unwrap();

    // Database connection pool
    pub static ref DB_CONNECTIONS_ACTIVE: Gauge = Gauge::new(
        "database_connections_active",
        "Active database connections"
    ).unwrap();

    // Business metrics
    pub static ref TOTAL_COST_TRACKED: Counter = Counter::new(
        "llm_total_cost_tracked_dollars",
        "Total cost tracked in dollars"
    ).unwrap();

    pub static ref API_CALLS_TOTAL: Counter = Counter::new(
        "llm_api_calls_total",
        "Total number of LLM API calls"
    ).unwrap();
}

// Register metrics
pub fn register_metrics(registry: &Registry) {
    registry.register(Box::new(HTTP_REQUESTS_TOTAL.clone())).unwrap();
    registry.register(Box::new(HTTP_REQUEST_DURATION.clone())).unwrap();
    registry.register(Box::new(ACTIVE_CONNECTIONS.clone())).unwrap();
    registry.register(Box::new(DB_CONNECTIONS_ACTIVE.clone())).unwrap();
    registry.register(Box::new(TOTAL_COST_TRACKED.clone())).unwrap();
    registry.register(Box::new(API_CALLS_TOTAL.clone())).unwrap();
}

// Middleware to track metrics
async fn track_metrics<B>(req: Request<B>, next: Next<B>) -> Response {
    let start = Instant::now();
    HTTP_REQUESTS_TOTAL.inc();
    ACTIVE_CONNECTIONS.inc();

    let response = next.run(req).await;

    let duration = start.elapsed();
    HTTP_REQUEST_DURATION.observe(duration.as_secs_f64());
    ACTIVE_CONNECTIONS.dec();

    response
}
```

### Custom Metrics

**Business metrics:**

```rust
// Track LLM API call
pub fn track_llm_call(provider: &str, model: &str, tokens: u32, cost: f64) {
    API_CALLS_TOTAL
        .with_label_values(&[provider, model])
        .inc();

    TOTAL_COST_TRACKED.inc_by(cost);

    TOKEN_USAGE
        .with_label_values(&[provider, model])
        .observe(tokens as f64);
}
```

### Exporters

**PostgreSQL Exporter:**

```yaml
services:
  postgres-exporter:
    image: prometheuscommunity/postgres-exporter:latest
    environment:
      DATA_SOURCE_NAME: "postgresql://postgres:password@postgres:5432/llm_cost_ops_prod?sslmode=disable"
    ports:
      - "9187:9187"
```

**Redis Exporter:**

```yaml
services:
  redis-exporter:
    image: oliver006/redis_exporter:latest
    environment:
      REDIS_ADDR: redis:6379
      REDIS_PASSWORD: ${REDIS_PASSWORD}
    ports:
      - "9121:9121"
```

**Node Exporter:**

```yaml
services:
  node-exporter:
    image: prom/node-exporter:latest
    command:
      - '--path.procfs=/host/proc'
      - '--path.sysfs=/host/sys'
      - '--collector.filesystem.mount-points-exclude=^/(sys|proc|dev|host|etc)($$|/)'
    volumes:
      - /proc:/host/proc:ro
      - /sys:/host/sys:ro
      - /:/rootfs:ro
    ports:
      - "9100:9100"
```

---

## Logging Setup

### Structured Logging

**Application logging:**

```rust
use tracing::{info, warn, error, debug, instrument};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    fmt,
    EnvFilter,
};

// Initialize logging
pub fn init_logging() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().json())
        .init();
}

// Structured logging example
#[instrument(skip(db))]
async fn create_user(db: &Database, email: String) -> Result<User> {
    info!(
        email = %email,
        action = "create_user",
        "Creating new user"
    );

    let user = db.insert_user(&email).await?;

    info!(
        user_id = %user.id,
        email = %email,
        action = "user_created",
        "User created successfully"
    );

    Ok(user)
}

// Error logging
async fn handle_request(req: Request) -> Result<Response> {
    match process_request(req).await {
        Ok(response) => Ok(response),
        Err(e) => {
            error!(
                error = %e,
                error_type = ?e,
                action = "request_failed",
                "Request processing failed"
            );
            Err(e)
        }
    }
}
```

### Log Aggregation with Loki

**docker-compose.yml:**

```yaml
services:
  loki:
    image: grafana/loki:2.9.0
    ports:
      - "3100:3100"
    volumes:
      - ./docker/loki/loki-config.yml:/etc/loki/local-config.yaml
      - loki-data:/loki
    command: -config.file=/etc/loki/local-config.yaml

  promtail:
    image: grafana/promtail:2.9.0
    volumes:
      - ./docker/promtail/promtail-config.yml:/etc/promtail/config.yml
      - /var/log:/var/log
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
    command: -config.file=/etc/promtail/config.yml

volumes:
  loki-data:
```

**loki-config.yml:**

```yaml
auth_enabled: false

server:
  http_listen_port: 3100

ingester:
  lifecycler:
    address: 127.0.0.1
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1
  chunk_idle_period: 5m
  chunk_retain_period: 30s

schema_config:
  configs:
    - from: 2024-01-01
      store: boltdb-shipper
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h

storage_config:
  boltdb_shipper:
    active_index_directory: /loki/index
    cache_location: /loki/cache
    shared_store: filesystem
  filesystem:
    directory: /loki/chunks

limits_config:
  enforce_metric_name: false
  reject_old_samples: true
  reject_old_samples_max_age: 168h
  retention_period: 720h  # 30 days
```

**promtail-config.yml:**

```yaml
server:
  http_listen_port: 9080
  grpc_listen_port: 0

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  # Docker containers
  - job_name: docker
    docker_sd_configs:
      - host: unix:///var/run/docker.sock
        refresh_interval: 5s
    relabel_configs:
      - source_labels: ['__meta_docker_container_name']
        regex: '/(.*)'
        target_label: 'container'
      - source_labels: ['__meta_docker_container_log_stream']
        target_label: 'stream'

  # Application logs
  - job_name: app
    static_configs:
      - targets:
          - localhost
        labels:
          job: llm-cost-ops
          __path__: /var/log/llm-cost-ops/*.log
```

### ELK Stack Alternative

**docker-compose.yml:**

```yaml
services:
  elasticsearch:
    image: elasticsearch:8.11.0
    environment:
      - discovery.type=single-node
      - "ES_JAVA_OPTS=-Xms512m -Xmx512m"
    volumes:
      - elasticsearch-data:/usr/share/elasticsearch/data

  logstash:
    image: logstash:8.11.0
    volumes:
      - ./docker/logstash/logstash.conf:/usr/share/logstash/pipeline/logstash.conf
    depends_on:
      - elasticsearch

  kibana:
    image: kibana:8.11.0
    ports:
      - "5601:5601"
    depends_on:
      - elasticsearch
```

---

## Distributed Tracing

### Jaeger Configuration

**docker-compose.yml:**

```yaml
services:
  jaeger:
    image: jaegertracing/all-in-one:1.52
    ports:
      - "5775:5775/udp"   # Zipkin compact thrift
      - "6831:6831/udp"   # Jaeger compact thrift
      - "6832:6832/udp"   # Jaeger binary thrift
      - "5778:5778"       # Serve configs
      - "16686:16686"     # Jaeger UI
      - "14268:14268"     # HTTP collector
      - "14269:14269"     # Admin port
      - "9411:9411"       # Zipkin compatible
    environment:
      COLLECTOR_ZIPKIN_HOST_PORT: :9411
      COLLECTOR_OTLP_ENABLED: true
      SPAN_STORAGE_TYPE: badger
      BADGER_EPHEMERAL: false
      BADGER_DIRECTORY_VALUE: /badger/data
      BADGER_DIRECTORY_KEY: /badger/key
    volumes:
      - jaeger-data:/badger
```

### OpenTelemetry Integration

**Rust implementation:**

```rust
use opentelemetry::{
    global,
    sdk::{
        trace::{self, Sampler},
        Resource,
    },
    KeyValue,
};
use opentelemetry_jaeger::new_agent_pipeline;
use tracing_subscriber::layer::SubscriberExt;

pub fn init_tracing() -> Result<()> {
    // Create Jaeger exporter
    let tracer = new_agent_pipeline()
        .with_service_name("llm-cost-ops")
        .with_endpoint("jaeger:6831")
        .with_auto_split_batch(true)
        .install_batch(opentelemetry::runtime::Tokio)?;

    // Create tracing subscriber
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

// Instrumented function
#[instrument]
async fn process_cost_calculation(user_id: &str, usage_data: UsageData) -> Result<Cost> {
    let span = tracing::Span::current();
    span.record("user_id", user_id);

    // Database query
    let user = fetch_user(user_id).await?;
    span.record("user_tier", &user.tier);

    // Calculate cost
    let cost = calculate_cost(&usage_data, &user.tier).await?;
    span.record("calculated_cost", cost.total);

    // Store result
    store_cost(&cost).await?;

    Ok(cost)
}
```

### Trace Visualization

Access Jaeger UI at http://localhost:16686

**Sample trace:**

```
POST /api/v1/costs [245ms]
├── authenticate_user [12ms]
│   └── fetch_user_from_db [10ms]
├── validate_request [5ms]
├── calculate_cost [180ms]
│   ├── fetch_pricing_data [45ms]
│   ├── calculate_tokens [15ms]
│   ├── calculate_total [5ms]
│   └── apply_discount [8ms]
├── store_cost [35ms]
│   └── insert_database [30ms]
└── send_notification [8ms]
```

---

## Dashboards

### Grafana Setup

**docker-compose.yml:**

```yaml
services:
  grafana:
    image: grafana/grafana:10.2.2
    ports:
      - "3000:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD: admin
      GF_INSTALL_PLUGINS: grafana-piechart-panel
    volumes:
      - grafana-data:/var/lib/grafana
      - ./docker/grafana/provisioning:/etc/grafana/provisioning
      - ./docker/grafana/dashboards:/var/lib/grafana/dashboards
```

### Data Sources

**datasource.yml:**

```yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    jsonData:
      timeInterval: 15s

  - name: Loki
    type: loki
    access: proxy
    url: http://loki:3100

  - name: Jaeger
    type: jaeger
    access: proxy
    url: http://jaeger:16686
```

### Application Dashboard

**Key panels:**

1. **Request Rate**
```promql
rate(http_requests_total[5m])
```

2. **Error Rate**
```promql
rate(http_requests_total{status=~"5.."}[5m])
/ rate(http_requests_total[5m])
```

3. **Request Duration (p95)**
```promql
histogram_quantile(0.95,
  rate(http_request_duration_seconds_bucket[5m])
)
```

4. **Active Connections**
```promql
active_connections
```

5. **Database Connections**
```promql
database_connections_active
```

6. **Total Cost Tracked**
```promql
llm_total_cost_tracked_dollars
```

### Infrastructure Dashboard

**System metrics:**

1. **CPU Usage**
```promql
100 - (avg by (instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)
```

2. **Memory Usage**
```promql
(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes)
/ node_memory_MemTotal_bytes * 100
```

3. **Disk Usage**
```promql
100 - ((node_filesystem_avail_bytes{mountpoint="/"}
/ node_filesystem_size_bytes{mountpoint="/"}) * 100)
```

4. **Network I/O**
```promql
rate(node_network_receive_bytes_total[5m])
rate(node_network_transmit_bytes_total[5m])
```

---

## Alerting

### Alert Rules

**alerts.yml:**

```yaml
groups:
  - name: llm-cost-ops-alerts
    interval: 30s
    rules:
      # High error rate
      - alert: HighErrorRate
        expr: |
          (
            rate(http_requests_total{status=~"5.."}[5m])
            / rate(http_requests_total[5m])
          ) > 0.05
        for: 5m
        labels:
          severity: critical
          component: application
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }} (threshold: 5%)"

      # Slow response time
      - alert: SlowResponseTime
        expr: |
          histogram_quantile(0.95,
            rate(http_request_duration_seconds_bucket[5m])
          ) > 1.0
        for: 5m
        labels:
          severity: warning
          component: application
        annotations:
          summary: "Slow response time"
          description: "p95 latency is {{ $value }}s (threshold: 1s)"

      # Database down
      - alert: DatabaseDown
        expr: up{job="postgres"} == 0
        for: 1m
        labels:
          severity: critical
          component: database
        annotations:
          summary: "Database is down"
          description: "PostgreSQL database is unreachable"

      # High CPU usage
      - alert: HighCPUUsage
        expr: |
          (
            100 - (avg by (instance)
              (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100
            )
          ) > 80
        for: 10m
        labels:
          severity: warning
          component: infrastructure
        annotations:
          summary: "High CPU usage"
          description: "CPU usage is {{ $value }}% (threshold: 80%)"

      # High memory usage
      - alert: HighMemoryUsage
        expr: |
          (
            (node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes)
            / node_memory_MemTotal_bytes * 100
          ) > 90
        for: 5m
        labels:
          severity: critical
          component: infrastructure
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value }}% (threshold: 90%)"

      # Disk space low
      - alert: DiskSpaceLow
        expr: |
          (
            100 - (
              (node_filesystem_avail_bytes{mountpoint="/"}
              / node_filesystem_size_bytes{mountpoint="/"}) * 100
            )
          ) > 85
        for: 5m
        labels:
          severity: warning
          component: infrastructure
        annotations:
          summary: "Disk space low"
          description: "Disk usage is {{ $value }}% (threshold: 85%)"
```

### Alertmanager Configuration

**alertmanager.yml:**

```yaml
global:
  resolve_timeout: 5m
  slack_api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'

route:
  group_by: ['alertname', 'severity']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'default'
  routes:
    - match:
        severity: critical
      receiver: 'critical'
    - match:
        severity: warning
      receiver: 'warning'

receivers:
  - name: 'default'
    slack_configs:
      - channel: '#alerts'
        title: 'LLM Cost Ops Alert'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

  - name: 'critical'
    slack_configs:
      - channel: '#alerts-critical'
        title: 'CRITICAL: {{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'
    pagerduty_configs:
      - service_key: 'YOUR_PAGERDUTY_KEY'

  - name: 'warning'
    slack_configs:
      - channel: '#alerts-warning'
        title: 'WARNING: {{ .GroupLabels.alertname }}'
        text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'instance']
```

---

## Performance Monitoring

### Key Metrics

**Golden Signals:**

1. **Latency** - Request duration
2. **Traffic** - Requests per second
3. **Errors** - Error rate
4. **Saturation** - Resource utilization

**RED Method:**

1. **Rate** - Requests per second
2. **Errors** - Error rate
3. **Duration** - Response time

**USE Method:**

1. **Utilization** - Resource usage %
2. **Saturation** - Queue depth
3. **Errors** - Error count

### SLI/SLO Monitoring

```yaml
# Recording rules
groups:
  - name: slo
    interval: 30s
    rules:
      # Availability SLI
      - record: slo:availability:ratio
        expr: |
          sum(rate(http_requests_total{status!~"5.."}[5m]))
          / sum(rate(http_requests_total[5m]))

      # Latency SLI (p95 < 500ms)
      - record: slo:latency:ratio
        expr: |
          histogram_quantile(0.95,
            sum(rate(http_request_duration_seconds_bucket[5m]))
          ) < 0.5

# SLO alerts
- alert: SLOViolation-Availability
  expr: slo:availability:ratio < 0.999
  for: 1h
  annotations:
    summary: "SLO violation: Availability below 99.9%"
```

---

## Debugging

### Live Debugging

```bash
# View real-time metrics
watch -n 1 curl -s http://localhost:9090/metrics

# Follow logs
docker compose logs -f app

# Tail specific log level
docker compose logs -f app | grep ERROR

# View traces
open http://localhost:16686
```

### Performance Profiling

```bash
# CPU profiling
cargo flamegraph --bin llm-cost-ops

# Memory profiling
valgrind --tool=massif ./target/release/llm-cost-ops

# Load testing
hey -n 10000 -c 100 http://localhost:8080/api/v1/health
```

---

## Best Practices

1. **Use structured logging** - JSON format
2. **Set up alerts** - Don't wait for users to report issues
3. **Monitor SLOs** - Track service level objectives
4. **Trace critical paths** - Identify bottlenecks
5. **Retain metrics** - At least 30 days
6. **Dashboard for each service** - Quick troubleshooting
7. **Document runbooks** - Response procedures
8. **Regular reviews** - Analyze trends
9. **Test alerts** - Verify they work
10. **Optimize queries** - Fast dashboard loading

---

## Troubleshooting

### Prometheus Issues

```bash
# Check Prometheus targets
curl http://localhost:9091/api/v1/targets | jq

# Reload configuration
curl -X POST http://localhost:9091/-/reload

# Query test
curl 'http://localhost:9091/api/v1/query?query=up'
```

### Grafana Issues

```bash
# Check data source connectivity
# Grafana UI -> Configuration -> Data Sources -> Test

# View Grafana logs
docker compose logs grafana

# Reset admin password
docker compose exec grafana grafana-cli admin reset-admin-password newpassword
```

### Jaeger Issues

```bash
# Check Jaeger health
curl http://localhost:14269/ | jq

# View spans
curl 'http://localhost:16686/api/traces?service=llm-cost-ops' | jq
```

---

**Last Updated:** 2025-11-16
**Version:** 1.0.0
