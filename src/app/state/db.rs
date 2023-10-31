//! Application state stored in Sqlite database

use sqlx::{sqlite::SqliteConnectOptions, Pool, Sqlite, SqlitePool};

#[derive(Debug)]
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
            last_fetched TEXT
        );",
        )
        .execute(&pool)
        .await?;

        Ok(Db { pool })
    }
}
