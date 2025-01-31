mod fetch;
mod http;
mod update;

use std::sync::Arc;

use fetch::{build_reqwest_client, fetch_feed};
use sqlx::PgPool;
use tokio::sync::Semaphore;
use update::get_feed_update;

use crate::sql::{FeedFormat, FeedStatus};

#[derive(Debug, Clone)]
struct FeedToUpdate {
    pub id: i32,
    pub status: FeedStatus,
    pub format: FeedFormat,
    pub link: String,

    pub skip_hours: Vec<i32>,
    pub skip_days_of_week: Vec<i32>,
    pub ttl_in_minutes: Option<i32>,
    pub etag: Option<String>,

    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub successful_fetch_at: chrono::DateTime<chrono::Utc>,
    pub next_fetch_at: chrono::DateTime<chrono::Utc>,
}

pub async fn get_out_of_date_feeds(pool: &PgPool) -> Result<Vec<FeedToUpdate>, sqlx::Error> {
    let out_of_date_feeds = sqlx::query_as!(
        FeedToUpdate,
        r#"
        SELECT
            id,
            status AS "status: FeedStatus",
            format AS "format: FeedFormat",
            link,

            skip_hours,
            skip_days_of_week,
            ttl_in_minutes,
            etag,

            updated_at,
            fetched_at,
            successful_fetch_at,
            next_fetch_at
        FROM feed
        WHERE next_fetch_at < NOW() AND status = 'active'"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(out_of_date_feeds)
}

pub async fn run_update(pool: &PgPool, concurrent_updates: usize) -> Result<(), sqlx::Error> {
    let semaphore = Arc::new(Semaphore::new(concurrent_updates));
    let mut handles = Vec::new();

    let client = build_reqwest_client();
    let out_of_date_feeds = get_out_of_date_feeds(pool).await?;

    for feed in out_of_date_feeds.into_iter() {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let client = client.clone();
        handles.push(tokio::spawn(async move {
            let status = fetch_feed(
                &client,
                &feed.link,
                Some(&feed.updated_at),
                feed.etag.as_deref(),
            )
            .await;

            let update = get_feed_update(status, feed);

            drop(permit);
        }));
    }

    Ok(())
}
