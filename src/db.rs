use std::time::Duration;


use sqlx::{postgres::PgPoolOptions, PgPool};


pub async fn get_db_pool() -> Result<PgPool, sqlx::Error> {
    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".to_string());

    // set up connection poolou
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
}
