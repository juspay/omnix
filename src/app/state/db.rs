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
        // FIXME: Handle error and display in UI!
        let pool = SqlitePool::connect_with(db_opts).await?;

        // Initial schema
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS flake (
                url TEXT NOT NULL,
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
}
