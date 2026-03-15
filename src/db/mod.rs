//! Database connection management.

pub mod guard;
pub mod migrate;
pub mod seed;
pub mod tenant;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

#[derive(Clone)]
pub struct Database {
    writer: PgPool,
    reader: PgPool,
}

impl Database {
    pub async fn connect(writer_url: &str, reader_url: &str) -> Result<Self, sqlx::Error> {
        let writer = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(Duration::from_secs(5))
            .connect(writer_url)
            .await?;

        let reader = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(Duration::from_secs(5))
            .connect(reader_url)
            .await?;

        Ok(Self { writer, reader })
    }

    /// Create a Database from an existing pool (uses the same pool for both reads and writes).
    pub fn from_pool(pool: PgPool) -> Self {
        Self {
            writer: pool.clone(),
            reader: pool,
        }
    }

    pub fn writer(&self) -> &PgPool {
        &self.writer
    }

    pub fn reader(&self) -> &PgPool {
        &self.reader
    }
}
