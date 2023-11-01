//! Application state stored in Sqlite database

use nix_rs::flake::url::FlakeUrl;
use sqlx::{sqlite::SqliteConnectOptions, Pool, Sqlite, SqlitePool};

#[derive(Debug, Clone)]
pub struct Db {
    pool: Pool<Sqlite>,
}

impl Db {
    /// Load and initialize the the application database
    pub async fn new() -> Result<Self, sqlx::Error> {
        let db_opts = SqliteConnectOptions::default()
            .create_if_missing(true)
            .foreign_keys(true)
            // TODO: Use ProjectDirs
            .filename("nix-browser.db");
        let pool = SqlitePool::connect_with(db_opts).await?;

        // Initial schema
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS flake (
                url TEXT NOT NULL PRIMARY KEY,
                metadata JSON,
                last_accessed TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                last_fetched TIMESTAMP
            );",
        )
        .execute(&pool)
        .await?;

        Ok(Db { pool })
    }

    pub async fn register_flake(&self, url: &FlakeUrl) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR IGNORE INTO flake (url) VALUES (?)")
            .bind(url.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn recent_flakes(&self) -> Result<Vec<FlakeUrl>, sqlx::Error> {
        let mut rows: Vec<(String,)> =
            sqlx::query_as("SELECT url FROM flake ORDER BY last_accessed ASC LIMIT 10")
                .fetch_all(&self.pool)
                .await?;
        let mut urls = Vec::new();
        while let Some(row) = rows.pop() {
            urls.push(row.0.into());
        }
        Ok(urls)
    }
}
