mod apply;
mod fetch;
mod http;
mod update;

use apply::apply_feed_update;
use sqlx::PgPool;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{oneshot, Semaphore},
    task::JoinHandle,
};

use fetch::{build_reqwest_client, fetch_feed};
use update::get_feed_update;

use crate::sql::{Feed, FeedFormat, FeedStatus};

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

pub struct Daemon {
    task: JoinHandle<()>,
    cancel_tx: oneshot::Sender<()>,
}

impl Daemon {
    pub fn new(pool: PgPool, concurrent_updates: usize) -> Self {
        let (cancel_tx, mut cancel_rx) = oneshot::channel();

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(15));

            loop {
                tokio::select! {
                    _ = &mut cancel_rx => break,
                    _ = interval.tick() => {
                        // TODO: split up the updates into batches and check for cancel each time
                        if let Err(err) = Daemon::run_update(&pool, concurrent_updates).await {
                            tracing::error!("Error in update: {:?}", err);
                        }
                    }
                }
            }
        });

        Self { task, cancel_tx }
    }

    async fn run_update(pool: &PgPool, concurrent_updates: usize) -> Result<(), sqlx::Error> {
        let semaphore = Arc::new(Semaphore::new(concurrent_updates));
        let mut handles = Vec::new();

        let client = build_reqwest_client();
        let out_of_date_feeds = Feed::get_out_of_date(pool).await?;

        for feed in out_of_date_feeds.into_iter() {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let client = client.clone();
            let pool = pool.clone();
            handles.push(tokio::spawn(async move {
                // HTTP request to get the feed
                let status = fetch_feed(
                    &client,
                    &feed.link,
                    Some(&feed.updated_at),
                    feed.etag.as_deref(),
                )
                .await;

                // Convert result of HTTP request to an update to the database
                let feed_update = get_feed_update(status, &feed).await;

                // Apply the update to the database
                apply_feed_update(&pool, &feed, &feed_update).await.unwrap();

                drop(permit);
            }));
        }

        Ok(())
    }

    pub async fn cancel(self) {
        // TODO: timeout and force cancel
        self.cancel_tx.send(()).unwrap();
        if let Err(err) = self.task.await {
            tracing::error!("Error while canceling feed daemon: {:?}", err);
        }
    }
}
