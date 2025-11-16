use chrono::{DateTime, Utc};
use sqlx::{Pool, Sqlite};
#[cfg(feature = "postgres")]
use sqlx::Postgres;
use tracing::info;
use uuid::Uuid;

use crate::domain::{CostRecord, PricingTable, Provider, Result, UsageRecord};
use super::models::{CostRecordRow, PricingTableRow, UsageRecordRow};

#[async_trait::async_trait]
pub trait UsageRepository: Send + Sync {
    async fn create(&self, record: &UsageRecord) -> Result<()>;
    async fn get_by_id(&self, id: Uuid) -> Result<Option<UsageRecord>>;
    async fn list_by_organization(
        &self,
        organization_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<UsageRecord>>;
}

#[async_trait::async_trait]
pub trait CostRepository: Send + Sync {
    async fn create(&self, record: &CostRecord) -> Result<()>;
    async fn get_by_usage_id(&self, usage_id: Uuid) -> Result<Option<CostRecord>>;
    async fn list_by_organization(
        &self,
        organization_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CostRecord>>;
}

#[async_trait::async_trait]
pub trait PricingRepository: Send + Sync {
    async fn create(&self, table: &PricingTable) -> Result<()>;
    async fn get_active(
        &self,
        provider: &Provider,
        model: &str,
        date: &DateTime<Utc>,
    ) -> Result<Option<PricingTable>>;
    async fn list_all(&self) -> Result<Vec<PricingTable>>;
}

// SQLite implementations
#[derive(Clone)]
pub struct SqliteUsageRepository {
    pool: Pool<Sqlite>,
}

impl SqliteUsageRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UsageRepository for SqliteUsageRepository {
    async fn create(&self, record: &UsageRecord) -> Result<()> {
        info!("Creating usage record: id={}", record.id);

        // Bind temporary values to variables to extend their lifetime
        let tags_json = serde_json::to_value(&record.tags)?;
        let source_type = match &record.source {
            crate::domain::IngestionSource::Api { .. } => "api",
            crate::domain::IngestionSource::File { .. } => "file",
            crate::domain::IngestionSource::Webhook { .. } => "webhook",
            crate::domain::IngestionSource::Stream { .. } => "stream",
        };
        let source_json = serde_json::to_value(&record.source)?;
        let context_window = record.model.context_window.map(|c| c as i64).unwrap_or(0);
        let prompt_tokens = record.prompt_tokens as i64;
        let completion_tokens = record.completion_tokens as i64;
        let total_tokens = record.total_tokens as i64;
        let cached_tokens = record.cached_tokens.map(|t| t as i64);
        let reasoning_tokens = record.reasoning_tokens.map(|t| t as i64);
        let latency_ms = record.latency_ms.map(|t| t as i64);
        let time_to_first_token_ms = record.time_to_first_token_ms.map(|t| t as i64);
        let provider_str = record.provider.as_str();

        sqlx::query!(
            r#"
            INSERT INTO usage_records (
                id, timestamp, provider, model_name, model_version, context_window,
                organization_id, project_id, user_id, prompt_tokens, completion_tokens,
                total_tokens, cached_tokens, reasoning_tokens, latency_ms,
                time_to_first_token_ms, tags, metadata, ingested_at, source_type, source_metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            record.id,
            record.timestamp,
            provider_str,
            record.model.name,
            record.model.version,
            context_window,
            record.organization_id,
            record.project_id,
            record.user_id,
            prompt_tokens,
            completion_tokens,
            total_tokens,
            cached_tokens,
            reasoning_tokens,
            latency_ms,
            time_to_first_token_ms,
            tags_json,
            record.metadata,
            record.ingested_at,
            source_type,
            source_json,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<UsageRecord>> {
        let row = sqlx::query_as!(
            UsageRecordRow,
            r#"
            SELECT * FROM usage_records WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| self.row_to_record(r)))
    }

    async fn list_by_organization(
        &self,
        organization_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<UsageRecord>> {
        let rows = sqlx::query_as!(
            UsageRecordRow,
            r#"
            SELECT * FROM usage_records
            WHERE organization_id = ? AND timestamp >= ? AND timestamp <= ?
            ORDER BY timestamp DESC
            "#,
            organization_id,
            start,
            end
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| self.row_to_record(r)).collect())
    }
}

impl SqliteUsageRepository {
    fn row_to_record(&self, row: UsageRecordRow) -> UsageRecord {
        use std::str::FromStr;

        UsageRecord {
            id: Uuid::from_str(&row.id).unwrap_or_else(|_| Uuid::new_v4()),
            timestamp: DateTime::parse_from_rfc3339(&row.timestamp)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            provider: Provider::parse(&row.provider),
            model: crate::domain::ModelIdentifier {
                name: row.model_name,
                version: row.model_version,
                context_window: if row.context_window > 0 {
                    Some(row.context_window as u64)
                } else {
                    None
                },
            },
            organization_id: row.organization_id,
            project_id: row.project_id,
            user_id: row.user_id,
            prompt_tokens: row.prompt_tokens as u64,
            completion_tokens: row.completion_tokens as u64,
            total_tokens: row.total_tokens as u64,
            cached_tokens: row.cached_tokens.map(|t| t as u64),
            reasoning_tokens: row.reasoning_tokens.map(|t| t as u64),
            latency_ms: row.latency_ms.map(|t| t as u64),
            time_to_first_token_ms: row.time_to_first_token_ms.map(|t| t as u64),
            tags: serde_json::from_value(row.tags).unwrap_or_default(),
            metadata: row.metadata,
            ingested_at: DateTime::parse_from_rfc3339(&row.ingested_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            source: serde_json::from_value(row.source_metadata).unwrap_or(
                crate::domain::IngestionSource::Api {
                    endpoint: "unknown".to_string(),
                },
            ),
        }
    }
}

#[derive(Clone)]
pub struct SqliteCostRepository {
    pool: Pool<Sqlite>,
}

impl SqliteCostRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CostRepository for SqliteCostRepository {
    async fn create(&self, record: &CostRecord) -> Result<()> {
        info!("Creating cost record: id={}", record.id);

        // Bind temporary values to variables to extend their lifetime
        let provider_str = record.provider.as_str();
        let currency_str = record.currency.as_str();
        let pricing_structure_json = serde_json::to_value(&record.pricing_structure)?;
        let tags_json = serde_json::to_value(&record.tags)?;
        let input_cost_str = record.input_cost.to_string();
        let output_cost_str = record.output_cost.to_string();
        let total_cost_str = record.total_cost.to_string();

        sqlx::query!(
            r#"
            INSERT INTO cost_records (
                id, usage_id, timestamp, provider, model_name, input_cost, output_cost,
                total_cost, currency, cost_model_id, pricing_structure, organization_id,
                project_id, tags, calculated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            record.id,
            record.usage_id,
            record.timestamp,
            provider_str,
            record.model,
            input_cost_str,
            output_cost_str,
            total_cost_str,
            currency_str,
            record.cost_model_id,
            pricing_structure_json,
            record.organization_id,
            record.project_id,
            tags_json,
            record.calculated_at,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_by_usage_id(&self, usage_id: Uuid) -> Result<Option<CostRecord>> {
        let row = sqlx::query_as!(
            CostRecordRow,
            r#"
            SELECT * FROM cost_records WHERE usage_id = ?
            "#,
            usage_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| self.row_to_record(r)))
    }

    async fn list_by_organization(
        &self,
        organization_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CostRecord>> {
        let rows = sqlx::query_as!(
            CostRecordRow,
            r#"
            SELECT * FROM cost_records
            WHERE organization_id = ? AND timestamp >= ? AND timestamp <= ?
            ORDER BY timestamp DESC
            "#,
            organization_id,
            start,
            end
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| self.row_to_record(r)).collect())
    }
}

impl SqliteCostRepository {
    fn row_to_record(&self, row: CostRecordRow) -> CostRecord {
        use rust_decimal::Decimal;
        use std::str::FromStr;

        CostRecord {
            id: Uuid::from_str(&row.id).unwrap_or_else(|_| Uuid::new_v4()),
            usage_id: Uuid::from_str(&row.usage_id).unwrap_or_else(|_| Uuid::new_v4()),
            timestamp: DateTime::parse_from_rfc3339(&row.timestamp)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            provider: Provider::parse(&row.provider),
            model: row.model_name,
            input_cost: Decimal::from_str(&row.input_cost).unwrap_or(Decimal::ZERO),
            output_cost: Decimal::from_str(&row.output_cost).unwrap_or(Decimal::ZERO),
            total_cost: Decimal::from_str(&row.total_cost).unwrap_or(Decimal::ZERO),
            currency: match row.currency.as_str() {
                "USD" => crate::domain::Currency::USD,
                "EUR" => crate::domain::Currency::EUR,
                "GBP" => crate::domain::Currency::GBP,
                "JPY" => crate::domain::Currency::JPY,
                other => crate::domain::Currency::Custom(other.to_string()),
            },
            cost_model_id: Uuid::from_str(&row.cost_model_id).unwrap_or_else(|_| Uuid::new_v4()),
            pricing_structure: serde_json::from_value(row.pricing_structure).unwrap_or(
                crate::domain::PricingStructure::simple_per_token(
                    Decimal::ZERO,
                    Decimal::ZERO,
                ),
            ),
            organization_id: row.organization_id,
            project_id: row.project_id,
            tags: serde_json::from_value(row.tags).unwrap_or_default(),
            calculated_at: DateTime::parse_from_rfc3339(&row.calculated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        }
    }
}

#[derive(Clone)]
pub struct SqlitePricingRepository {
    pool: Pool<Sqlite>,
}

impl SqlitePricingRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl PricingRepository for SqlitePricingRepository {
    async fn create(&self, table: &PricingTable) -> Result<()> {
        info!("Creating pricing table: id={} provider={} model={}", table.id, table.provider, table.model);

        // Bind temporary values to variables to extend their lifetime
        let pricing_json = serde_json::to_value(&table.pricing)?;
        let provider_str = table.provider.as_str();
        let currency_str = table.currency.as_str();

        sqlx::query!(
            r#"
            INSERT INTO pricing_tables (
                id, provider, model_name, effective_date, end_date, pricing_structure,
                currency, region, metadata, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            table.id,
            provider_str,
            table.model,
            table.effective_date,
            table.end_date,
            pricing_json,
            currency_str,
            table.region,
            table.metadata,
            table.created_at,
            table.updated_at,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_active(
        &self,
        provider: &Provider,
        model: &str,
        date: &DateTime<Utc>,
    ) -> Result<Option<PricingTable>> {
        let provider_str = provider.as_str();

        let row = sqlx::query_as!(
            PricingTableRow,
            r#"
            SELECT * FROM pricing_tables
            WHERE provider = ? AND model_name = ? AND effective_date <= ?
            AND (end_date IS NULL OR end_date >= ?)
            ORDER BY effective_date DESC
            LIMIT 1
            "#,
            provider_str,
            model,
            date,
            date
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| self.row_to_table(r)))
    }

    async fn list_all(&self) -> Result<Vec<PricingTable>> {
        let rows = sqlx::query_as!(
            PricingTableRow,
            r#"
            SELECT * FROM pricing_tables
            ORDER BY provider, model_name, effective_date DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| self.row_to_table(r)).collect())
    }
}

impl SqlitePricingRepository {
    fn row_to_table(&self, row: PricingTableRow) -> PricingTable {
        use std::str::FromStr;

        PricingTable {
            id: Uuid::from_str(&row.id).unwrap_or_else(|_| Uuid::new_v4()),
            provider: Provider::parse(&row.provider),
            model: row.model_name,
            effective_date: DateTime::parse_from_rfc3339(&row.effective_date)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            end_date: row.end_date.and_then(|d| {
                DateTime::parse_from_rfc3339(&d)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
            pricing: serde_json::from_value(row.pricing_structure).unwrap_or(
                crate::domain::PricingStructure::simple_per_token(
                    rust_decimal::Decimal::ZERO,
                    rust_decimal::Decimal::ZERO,
                ),
            ),
            currency: match row.currency.as_str() {
                "USD" => crate::domain::Currency::USD,
                "EUR" => crate::domain::Currency::EUR,
                "GBP" => crate::domain::Currency::GBP,
                "JPY" => crate::domain::Currency::JPY,
                other => crate::domain::Currency::Custom(other.to_string()),
            },
            region: row.region,
            metadata: row.metadata,
            created_at: DateTime::parse_from_rfc3339(&row.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&row.updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        }
    }
}

// ============================================================================
// PostgreSQL implementations
// ============================================================================

#[cfg(feature = "postgres")]
#[derive(Clone)]
pub struct PostgresUsageRepository {
    pool: Pool<Postgres>,
}

#[cfg(feature = "postgres")]
impl PostgresUsageRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait::async_trait]
impl UsageRepository for PostgresUsageRepository {
    async fn create(&self, record: &UsageRecord) -> Result<()> {
        info!("Creating usage record: id={}", record.id);

        let tags_json = serde_json::to_value(&record.tags)?;
        let source_type = match &record.source {
            crate::domain::IngestionSource::Api { .. } => "api",
            crate::domain::IngestionSource::File { .. } => "file",
            crate::domain::IngestionSource::Webhook { .. } => "webhook",
            crate::domain::IngestionSource::Stream { .. } => "stream",
        };
        let source_json = serde_json::to_value(&record.source)?;
        let context_window = record.model.context_window.map(|c| c as i64).unwrap_or(0);
        let prompt_tokens = record.prompt_tokens as i64;
        let completion_tokens = record.completion_tokens as i64;
        let total_tokens = record.total_tokens as i64;
        let cached_tokens = record.cached_tokens.map(|t| t as i64);
        let reasoning_tokens = record.reasoning_tokens.map(|t| t as i64);
        let latency_ms = record.latency_ms.map(|t| t as i64);
        let time_to_first_token_ms = record.time_to_first_token_ms.map(|t| t as i64);
        let provider_str = record.provider.as_str();

        sqlx::query!(
            r#"
            INSERT INTO usage_records (
                id, timestamp, provider, model_name, model_version, context_window,
                organization_id, project_id, user_id, prompt_tokens, completion_tokens,
                total_tokens, cached_tokens, reasoning_tokens, latency_ms,
                time_to_first_token_ms, tags, metadata, ingested_at, source_type, source_metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            "#,
            record.id,
            record.timestamp,
            provider_str,
            record.model.name,
            record.model.version,
            context_window,
            record.organization_id,
            record.project_id,
            record.user_id,
            prompt_tokens,
            completion_tokens,
            total_tokens,
            cached_tokens,
            reasoning_tokens,
            latency_ms,
            time_to_first_token_ms,
            tags_json,
            record.metadata,
            record.ingested_at,
            source_type,
            source_json,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<UsageRecord>> {
        let row = sqlx::query!(
            r#"
            SELECT
                id, timestamp, provider, model_name, model_version, context_window,
                organization_id, project_id, user_id, prompt_tokens, completion_tokens,
                total_tokens, cached_tokens, reasoning_tokens, latency_ms,
                time_to_first_token_ms, tags, metadata, ingested_at, source_type,
                source_metadata, created_at
            FROM usage_records WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            UsageRecord {
                id: r.id,
                timestamp: r.timestamp,
                provider: Provider::parse(&r.provider),
                model: crate::domain::ModelIdentifier {
                    name: r.model_name,
                    version: r.model_version,
                    context_window: if r.context_window > 0 {
                        Some(r.context_window as u64)
                    } else {
                        None
                    },
                },
                organization_id: r.organization_id,
                project_id: r.project_id,
                user_id: r.user_id,
                prompt_tokens: r.prompt_tokens as u64,
                completion_tokens: r.completion_tokens as u64,
                total_tokens: r.total_tokens as u64,
                cached_tokens: r.cached_tokens.map(|t| t as u64),
                reasoning_tokens: r.reasoning_tokens.map(|t| t as u64),
                latency_ms: r.latency_ms.map(|t| t as u64),
                time_to_first_token_ms: r.time_to_first_token_ms.map(|t| t as u64),
                tags: serde_json::from_value(r.tags).unwrap_or_default(),
                metadata: r.metadata,
                ingested_at: r.ingested_at,
                source: serde_json::from_value(r.source_metadata).unwrap_or(
                    crate::domain::IngestionSource::Api {
                        endpoint: "unknown".to_string(),
                    },
                ),
            }
        }))
    }

    async fn list_by_organization(
        &self,
        organization_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<UsageRecord>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, timestamp, provider, model_name, model_version, context_window,
                organization_id, project_id, user_id, prompt_tokens, completion_tokens,
                total_tokens, cached_tokens, reasoning_tokens, latency_ms,
                time_to_first_token_ms, tags, metadata, ingested_at, source_type,
                source_metadata, created_at
            FROM usage_records
            WHERE organization_id = $1 AND timestamp >= $2 AND timestamp <= $3
            ORDER BY timestamp DESC
            "#,
            organization_id,
            start,
            end
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                UsageRecord {
                    id: r.id,
                    timestamp: r.timestamp,
                    provider: Provider::parse(&r.provider),
                    model: crate::domain::ModelIdentifier {
                        name: r.model_name,
                        version: r.model_version,
                        context_window: if r.context_window > 0 {
                            Some(r.context_window as u64)
                        } else {
                            None
                        },
                    },
                    organization_id: r.organization_id,
                    project_id: r.project_id,
                    user_id: r.user_id,
                    prompt_tokens: r.prompt_tokens as u64,
                    completion_tokens: r.completion_tokens as u64,
                    total_tokens: r.total_tokens as u64,
                    cached_tokens: r.cached_tokens.map(|t| t as u64),
                    reasoning_tokens: r.reasoning_tokens.map(|t| t as u64),
                    latency_ms: r.latency_ms.map(|t| t as u64),
                    time_to_first_token_ms: r.time_to_first_token_ms.map(|t| t as u64),
                    tags: serde_json::from_value(r.tags).unwrap_or_default(),
                    metadata: r.metadata,
                    ingested_at: r.ingested_at,
                    source: serde_json::from_value(r.source_metadata).unwrap_or(
                        crate::domain::IngestionSource::Api {
                            endpoint: "unknown".to_string(),
                        },
                    ),
                }
            })
            .collect())
    }
}

#[cfg(feature = "postgres")]
#[derive(Clone)]
pub struct PostgresCostRepository {
    pool: Pool<Postgres>,
}

#[cfg(feature = "postgres")]
impl PostgresCostRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait::async_trait]
impl CostRepository for PostgresCostRepository {
    async fn create(&self, record: &CostRecord) -> Result<()> {
        info!("Creating cost record: id={}", record.id);

        let provider_str = record.provider.as_str();
        let currency_str = record.currency.as_str();
        let pricing_structure_json = serde_json::to_value(&record.pricing_structure)?;
        let tags_json = serde_json::to_value(&record.tags)?;

        sqlx::query!(
            r#"
            INSERT INTO cost_records (
                id, usage_id, timestamp, provider, model_name, input_cost, output_cost,
                total_cost, currency, cost_model_id, pricing_structure, organization_id,
                project_id, tags, calculated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            record.id,
            record.usage_id,
            record.timestamp,
            provider_str,
            record.model,
            record.input_cost,
            record.output_cost,
            record.total_cost,
            currency_str,
            record.cost_model_id,
            pricing_structure_json,
            record.organization_id,
            record.project_id,
            tags_json,
            record.calculated_at,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_by_usage_id(&self, usage_id: Uuid) -> Result<Option<CostRecord>> {
        let row = sqlx::query!(
            r#"
            SELECT
                id, usage_id, timestamp, provider, model_name, input_cost, output_cost,
                total_cost, currency, cost_model_id, pricing_structure, organization_id,
                project_id, tags, calculated_at, created_at
            FROM cost_records WHERE usage_id = $1
            "#,
            usage_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            CostRecord {
                id: r.id,
                usage_id: r.usage_id,
                timestamp: r.timestamp,
                provider: Provider::parse(&r.provider),
                model: r.model_name,
                input_cost: r.input_cost,
                output_cost: r.output_cost,
                total_cost: r.total_cost,
                currency: match r.currency.as_str() {
                    "USD" => crate::domain::Currency::USD,
                    "EUR" => crate::domain::Currency::EUR,
                    "GBP" => crate::domain::Currency::GBP,
                    "JPY" => crate::domain::Currency::JPY,
                    other => crate::domain::Currency::Custom(other.to_string()),
                },
                cost_model_id: r.cost_model_id,
                pricing_structure: serde_json::from_value(r.pricing_structure).unwrap_or(
                    crate::domain::PricingStructure::simple_per_token(
                        rust_decimal::Decimal::ZERO,
                        rust_decimal::Decimal::ZERO,
                    ),
                ),
                organization_id: r.organization_id,
                project_id: r.project_id,
                tags: serde_json::from_value(r.tags).unwrap_or_default(),
                calculated_at: r.calculated_at,
            }
        }))
    }

    async fn list_by_organization(
        &self,
        organization_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CostRecord>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, usage_id, timestamp, provider, model_name, input_cost, output_cost,
                total_cost, currency, cost_model_id, pricing_structure, organization_id,
                project_id, tags, calculated_at, created_at
            FROM cost_records
            WHERE organization_id = $1 AND timestamp >= $2 AND timestamp <= $3
            ORDER BY timestamp DESC
            "#,
            organization_id,
            start,
            end
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                CostRecord {
                    id: r.id,
                    usage_id: r.usage_id,
                    timestamp: r.timestamp,
                    provider: Provider::parse(&r.provider),
                    model: r.model_name,
                    input_cost: r.input_cost,
                    output_cost: r.output_cost,
                    total_cost: r.total_cost,
                    currency: match r.currency.as_str() {
                        "USD" => crate::domain::Currency::USD,
                        "EUR" => crate::domain::Currency::EUR,
                        "GBP" => crate::domain::Currency::GBP,
                        "JPY" => crate::domain::Currency::JPY,
                        other => crate::domain::Currency::Custom(other.to_string()),
                    },
                    cost_model_id: r.cost_model_id,
                    pricing_structure: serde_json::from_value(r.pricing_structure).unwrap_or(
                        crate::domain::PricingStructure::simple_per_token(
                            rust_decimal::Decimal::ZERO,
                            rust_decimal::Decimal::ZERO,
                        ),
                    ),
                    organization_id: r.organization_id,
                    project_id: r.project_id,
                    tags: serde_json::from_value(r.tags).unwrap_or_default(),
                    calculated_at: r.calculated_at,
                }
            })
            .collect())
    }
}

#[cfg(feature = "postgres")]
#[derive(Clone)]
pub struct PostgresPricingRepository {
    pool: Pool<Postgres>,
}

#[cfg(feature = "postgres")]
impl PostgresPricingRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait::async_trait]
impl PricingRepository for PostgresPricingRepository {
    async fn create(&self, table: &PricingTable) -> Result<()> {
        info!("Creating pricing table: id={} provider={} model={}", table.id, table.provider, table.model);

        let pricing_json = serde_json::to_value(&table.pricing)?;
        let provider_str = table.provider.as_str();
        let currency_str = table.currency.as_str();

        sqlx::query!(
            r#"
            INSERT INTO pricing_tables (
                id, provider, model_name, effective_date, end_date, pricing_structure,
                currency, region, metadata, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            table.id,
            provider_str,
            table.model,
            table.effective_date,
            table.end_date,
            pricing_json,
            currency_str,
            table.region,
            table.metadata,
            table.created_at,
            table.updated_at,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_active(
        &self,
        provider: &Provider,
        model: &str,
        date: &DateTime<Utc>,
    ) -> Result<Option<PricingTable>> {
        let provider_str = provider.as_str();

        let row = sqlx::query!(
            r#"
            SELECT
                id, provider, model_name, effective_date, end_date, pricing_structure,
                currency, region, metadata, created_at, updated_at
            FROM pricing_tables
            WHERE provider = $1 AND model_name = $2 AND effective_date <= $3
            AND (end_date IS NULL OR end_date >= $3)
            ORDER BY effective_date DESC
            LIMIT 1
            "#,
            provider_str,
            model,
            date
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            PricingTable {
                id: r.id,
                provider: Provider::parse(&r.provider),
                model: r.model_name,
                effective_date: r.effective_date,
                end_date: r.end_date,
                pricing: serde_json::from_value(r.pricing_structure).unwrap_or(
                    crate::domain::PricingStructure::simple_per_token(
                        rust_decimal::Decimal::ZERO,
                        rust_decimal::Decimal::ZERO,
                    ),
                ),
                currency: match r.currency.as_str() {
                    "USD" => crate::domain::Currency::USD,
                    "EUR" => crate::domain::Currency::EUR,
                    "GBP" => crate::domain::Currency::GBP,
                    "JPY" => crate::domain::Currency::JPY,
                    other => crate::domain::Currency::Custom(other.to_string()),
                },
                region: r.region,
                metadata: r.metadata,
                created_at: r.created_at,
                updated_at: r.updated_at,
            }
        }))
    }

    async fn list_all(&self) -> Result<Vec<PricingTable>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, provider, model_name, effective_date, end_date, pricing_structure,
                currency, region, metadata, created_at, updated_at
            FROM pricing_tables
            ORDER BY provider, model_name, effective_date DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                PricingTable {
                    id: r.id,
                    provider: Provider::parse(&r.provider),
                    model: r.model_name,
                    effective_date: r.effective_date,
                    end_date: r.end_date,
                    pricing: serde_json::from_value(r.pricing_structure).unwrap_or(
                        crate::domain::PricingStructure::simple_per_token(
                            rust_decimal::Decimal::ZERO,
                            rust_decimal::Decimal::ZERO,
                        ),
                    ),
                    currency: match r.currency.as_str() {
                        "USD" => crate::domain::Currency::USD,
                        "EUR" => crate::domain::Currency::EUR,
                        "GBP" => crate::domain::Currency::GBP,
                        "JPY" => crate::domain::Currency::JPY,
                        other => crate::domain::Currency::Custom(other.to_string()),
                    },
                    region: r.region,
                    metadata: r.metadata,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                }
            })
            .collect())
    }
}
