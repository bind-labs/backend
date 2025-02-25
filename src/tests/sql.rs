use std::ops::Deref;

use pgtemp::PgTempDB;
use sqlx::postgres::PgPoolOptions;

#[allow(dead_code)]
pub struct TempDB(pub sqlx::PgPool, pub PgTempDB);

impl TempDB {
    pub async fn new() -> Self {
        let db = PgTempDB::async_new().await;
        let pool = PgPoolOptions::new()
            .connect(&db.connection_uri())
            .await
            .unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        Self(pool, db)
    }
}

impl Deref for TempDB {
    type Target = sqlx::PgPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
