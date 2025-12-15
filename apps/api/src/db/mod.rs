//! Database module

use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./src/db/migrations")
            .run(&self.pool)
            .await?;
        Ok(())
    }
}
