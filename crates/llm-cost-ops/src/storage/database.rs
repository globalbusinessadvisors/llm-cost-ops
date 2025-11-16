// Database connection and pool management

use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, Pool, Sqlite};
use std::str::FromStr;
use std::time::Duration;
use tracing::info;

#[cfg(feature = "postgres")]
use tracing::warn;

#[cfg(feature = "postgres")]
use sqlx::Postgres;

use crate::domain::{CostOpsError, Result};

/// Database type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    Sqlite,
    #[cfg(feature = "postgres")]
    Postgres,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database type (sqlite or postgres)
    pub database_type: DatabaseType,

    /// Connection URL
    /// - SQLite: "sqlite://path/to/db.db" or "sqlite::memory:"
    /// - PostgreSQL: "postgresql://user:password@host:port/database"
    pub url: String,

    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Minimum number of idle connections in the pool
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_secs: u64,

    /// Idle timeout in seconds
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,

    /// Maximum lifetime of a connection in seconds
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime_secs: u64,

    /// Run migrations on startup
    #[serde(default = "default_run_migrations")]
    pub run_migrations: bool,
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    2
}

fn default_connection_timeout() -> u64 {
    30
}

fn default_idle_timeout() -> u64 {
    600 // 10 minutes
}

fn default_max_lifetime() -> u64 {
    1800 // 30 minutes
}

fn default_run_migrations() -> bool {
    true
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_type: DatabaseType::Sqlite,
            url: "sqlite::memory:".to_string(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connection_timeout_secs: default_connection_timeout(),
            idle_timeout_secs: default_idle_timeout(),
            max_lifetime_secs: default_max_lifetime(),
            run_migrations: default_run_migrations(),
        }
    }
}

impl DatabaseConfig {
    /// Create SQLite configuration
    pub fn sqlite(path: impl Into<String>) -> Self {
        Self {
            database_type: DatabaseType::Sqlite,
            url: format!("sqlite://{}", path.into()),
            ..Default::default()
        }
    }

    /// Create in-memory SQLite configuration
    pub fn sqlite_memory() -> Self {
        Self {
            database_type: DatabaseType::Sqlite,
            url: "sqlite::memory:".to_string(),
            ..Default::default()
        }
    }

    /// Create PostgreSQL configuration
    #[cfg(feature = "postgres")]
    pub fn postgres(url: impl Into<String>) -> Self {
        Self {
            database_type: DatabaseType::Postgres,
            url: url.into(),
            max_connections: 20, // Higher default for PostgreSQL
            ..Default::default()
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.url.is_empty() {
            return Err(CostOpsError::config(
                "Database URL cannot be empty",
            ));
        }

        if self.max_connections == 0 {
            return Err(CostOpsError::config(
                "Max connections must be greater than 0",
            ));
        }

        if self.min_connections > self.max_connections {
            return Err(CostOpsError::config(
                "Min connections cannot exceed max connections",
            ));
        }

        Ok(())
    }
}

/// SQLite connection pool manager
#[derive(Clone)]
pub struct SqlitePool {
    pool: Pool<Sqlite>,
}

impl SqlitePool {
    /// Create a new SQLite pool
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        config.validate()?;

        info!("Creating SQLite connection pool: {}", config.url);

        // Create database if it doesn't exist (for file-based SQLite)
        if !config.url.contains(":memory:") && !Sqlite::database_exists(&config.url).await? {
            info!("Creating SQLite database: {}", config.url);
            Sqlite::create_database(&config.url).await?;
        }

        // Configure pool options
        let pool = sqlx::SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::from_str(&config.url)?
                .create_if_missing(true)
                .busy_timeout(Duration::from_secs(config.connection_timeout_secs))
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
                .foreign_keys(true),
        )
        .await?;

        // Set pool configuration
        pool.set_connect_options(
            sqlx::sqlite::SqliteConnectOptions::from_str(&config.url)?
                .create_if_missing(true)
                .busy_timeout(Duration::from_secs(config.connection_timeout_secs))
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .foreign_keys(true),
        );

        let pool_instance = Self { pool };

        // Run migrations if enabled
        if config.run_migrations {
            pool_instance.run_migrations().await?;
        }

        info!("SQLite pool created successfully");
        Ok(pool_instance)
    }

    /// Get the underlying pool
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Run migrations
    pub async fn run_migrations(&self) -> Result<()> {
        info!("Running SQLite migrations");
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| CostOpsError::internal(format!("Migration failed: {}", e)))?;
        info!("Migrations completed successfully");
        Ok(())
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            connections: self.pool.size(),
            idle_connections: self.pool.num_idle() as u32,
        }
    }

    /// Close the pool
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

/// PostgreSQL connection pool manager
#[cfg(feature = "postgres")]
#[derive(Clone)]
pub struct PostgresPool {
    pool: Pool<Postgres>,
}

#[cfg(feature = "postgres")]
impl PostgresPool {
    /// Create a new PostgreSQL pool
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        config.validate()?;

        info!("Creating PostgreSQL connection pool: {}", config.url);

        // Check if database exists
        let db_exists = Postgres::database_exists(&config.url).await?;
        if !db_exists {
            warn!("PostgreSQL database does not exist, attempting to create it");
            Postgres::create_database(&config.url).await?;
            info!("PostgreSQL database created successfully");
        }

        // Configure pool options
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.connection_timeout_secs))
            .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
            .max_lifetime(Duration::from_secs(config.max_lifetime_secs))
            .connect(&config.url)
            .await?;

        let pool_instance = Self { pool };

        // Run migrations if enabled
        if config.run_migrations {
            pool_instance.run_migrations().await?;
        }

        info!("PostgreSQL pool created successfully");
        Ok(pool_instance)
    }

    /// Get the underlying pool
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    /// Run migrations
    pub async fn run_migrations(&self) -> Result<()> {
        info!("Running PostgreSQL migrations");
        sqlx::migrate!("./migrations_postgres")
            .run(&self.pool)
            .await
            .map_err(|e| CostOpsError::internal(format!("Migration failed: {}", e)))?;
        info!("Migrations completed successfully");
        Ok(())
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            connections: self.pool.size() as u32,
            idle_connections: self.pool.num_idle() as u32,
        }
    }

    /// Close the pool
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Execute a query to refresh materialized views (PostgreSQL specific)
    pub async fn refresh_materialized_views(&self) -> Result<()> {
        info!("Refreshing PostgreSQL materialized views");
        sqlx::query("SELECT refresh_cost_summary()")
            .execute(&self.pool)
            .await?;
        info!("Materialized views refreshed successfully");
        Ok(())
    }
}

/// Pool statistics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PoolStats {
    pub connections: u32,
    pub idle_connections: u32,
}

/// Unified database pool enum
#[derive(Clone)]
pub enum DatabasePool {
    Sqlite(SqlitePool),
    #[cfg(feature = "postgres")]
    Postgres(PostgresPool),
}

impl DatabasePool {
    /// Create a new database pool from configuration
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        match config.database_type {
            DatabaseType::Sqlite => {
                let pool = SqlitePool::new(config).await?;
                Ok(DatabasePool::Sqlite(pool))
            }
            #[cfg(feature = "postgres")]
            DatabaseType::Postgres => {
                let pool = PostgresPool::new(config).await?;
                Ok(DatabasePool::Postgres(pool))
            }
        }
    }

    /// Run migrations
    pub async fn run_migrations(&self) -> Result<()> {
        match self {
            DatabasePool::Sqlite(pool) => pool.run_migrations().await,
            #[cfg(feature = "postgres")]
            DatabasePool::Postgres(pool) => pool.run_migrations().await,
        }
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        match self {
            DatabasePool::Sqlite(pool) => pool.health_check().await,
            #[cfg(feature = "postgres")]
            DatabasePool::Postgres(pool) => pool.health_check().await,
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        match self {
            DatabasePool::Sqlite(pool) => pool.stats(),
            #[cfg(feature = "postgres")]
            DatabasePool::Postgres(pool) => pool.stats(),
        }
    }

    /// Close the pool
    pub async fn close(&self) {
        match self {
            DatabasePool::Sqlite(pool) => pool.close().await,
            #[cfg(feature = "postgres")]
            DatabasePool::Postgres(pool) => pool.close().await,
        }
    }

    /// Get database type
    pub fn database_type(&self) -> DatabaseType {
        match self {
            DatabasePool::Sqlite(_) => DatabaseType::Sqlite,
            #[cfg(feature = "postgres")]
            DatabasePool::Postgres(_) => DatabaseType::Postgres,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_validation() {
        let mut config = DatabaseConfig::default();
        assert!(config.validate().is_ok());

        config.url = String::new();
        assert!(config.validate().is_err());

        config = DatabaseConfig::default();
        config.max_connections = 0;
        assert!(config.validate().is_err());

        config = DatabaseConfig::default();
        config.min_connections = 20;
        config.max_connections = 10;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_sqlite_config_creation() {
        let config = DatabaseConfig::sqlite("test.db");
        assert_eq!(config.database_type, DatabaseType::Sqlite);
        assert_eq!(config.url, "sqlite://test.db");

        let config = DatabaseConfig::sqlite_memory();
        assert_eq!(config.url, "sqlite::memory:");
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn test_postgres_config_creation() {
        let config = DatabaseConfig::postgres("postgresql://localhost/test");
        assert_eq!(config.database_type, DatabaseType::Postgres);
        assert_eq!(config.url, "postgresql://localhost/test");
    }

    #[tokio::test]
    async fn test_sqlite_pool_creation() {
        let mut config = DatabaseConfig::sqlite_memory();
        config.run_migrations = false; // Disable migrations for test

        let pool = SqlitePool::new(&config).await;
        assert!(pool.is_ok());

        let pool = pool.unwrap();
        assert!(pool.health_check().await.is_ok());

        let stats = pool.stats();
        assert!(stats.connections > 0);
    }
}
