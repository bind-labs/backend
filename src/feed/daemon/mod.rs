// TODO: if a feed has its link updated and we already have a feed with that link,
// we need to delete the old feed and migrate anything pointing to that feed to the existing one

mod apply;
mod constants;
mod create;
mod fetch;
mod http;
mod update;

use anyhow::{Context, Result};
use apply::apply_feed_update;
use chrono::Utc;
use kube_leader_election::{LeaseLock, LeaseLockParams};
use ormx::Insert;
use rand::{distr::Alphanumeric, Rng};
use sqlx::PgPool;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{oneshot, OwnedSemaphorePermit, Semaphore},
    task::JoinHandle,
};

use create::get_feed_creation;
pub use create::FeedCreationError;
pub use update::FeedUpdate;

use fetch::{build_reqwest_client, fetch_feed};
use update::get_feed_update;

use crate::sql::{Feed, InsertFeedItem};

fn generate_random_name(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .take(len)
        .collect()
}

pub struct DaemonOptions {
    pub concurrent_updates: usize,
    pub lease_name: Option<String>,
}

pub struct Daemon {
    task: JoinHandle<()>,
    cancel_tx: oneshot::Sender<()>,
}

impl Daemon {
    pub async fn new(pool: PgPool, options: DaemonOptions) -> Result<Self> {
        let (cancel_tx, mut cancel_rx) = oneshot::channel();

        // Leader lock
        let leadership = if let Some(lease_name) = options.lease_name {
            let client = kube::Client::try_default()
                .await
                .context("Failed to connect to Kubernetes")?;
            let namespace = client.default_namespace().to_string();
            Some(LeaseLock::new(
                client,
                &namespace,
                LeaseLockParams {
                    holder_id: format!("{}-{}", lease_name, generate_random_name(8)),
                    lease_name,
                    lease_ttl: Duration::from_secs(120),
                },
            ))
        } else {
            None
        };

        // TODO: handle the task failing
        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                tokio::select! {
                    // Cancelled, release lock and exit
                    _ = &mut cancel_rx => {
                        if let Some(leadership) = &leadership {
                            if let Err(err) = leadership.step_down().await {
                                tracing::error!("Error stepping down from leadership: {:?}", err);
                            }
                        }
                        break
                    },

                    // Interval ticked, update if we have lock
                    _ = interval.tick() => {
                        if let Some(leadership) = &leadership {
                            match leadership.try_acquire_or_renew().await {
                                Ok(lease) => {
                                    if !lease.acquired_lease {
                                        tracing::info!("Not the leader, skipping feed update");
                                        continue;
                                    }
                                },
                                Err(err) => {
                                    // TODO: exit after X failures
                                    tracing::error!("Failed while attempting to acquire lease: {:?}", err);
                                    continue;
                                }
                            };
                        }

                        tracing::info!("Running feed update");
                        // TODO: split up the updates into batches and check for cancel each time
                        // TODO: exit after X failures
                        if let Err(err) = Daemon::update_outdated_feeds(&pool, options.concurrent_updates).await {
                            tracing::error!("Error in update: {:?}", err);
                        }
                    }
                }
            }
        });

        Ok(Self { task, cancel_tx })
    }

    /// Runs a single feed update with the provided concurrency limit by first fetching all
    /// out of date feeds (via next_fetch_at < now) and then running the updates concurrently,
    /// with a provided concurrency limit.
    async fn update_outdated_feeds(
        pool: &PgPool,
        concurrent_updates: usize,
    ) -> Result<(), anyhow::Error> {
        let semaphore = Arc::new(Semaphore::new(concurrent_updates));
        let mut handles = Vec::new();

        for feed in Feed::get_out_of_date(pool).await?.into_iter() {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let pool = pool.clone();

            handles.push(tokio::spawn(Daemon::update_feed(pool, feed, Some(permit))));
        }

        for handle in handles {
            handle.await??;
        }

        Ok(())
    }

    /// Updates a single feed via the following steps:
    ///
    /// 1. Fetch the feed's url
    /// 2. Convert the response to a FeedUpdate
    ///     - If the feed is modified, parse the feed and items
    ///     - If the feed is not modified, update the next fetch time
    ///     - If the feed returned 304 (Moved), update the link and leave for next iteration
    ///     - If the feed returned 429, update the next fetch time based on the Retry-After header
    ///     - Otherwise, the feed failed to fetch, update the next fetch time with backoff
    /// 3. Apply the FeedUpdate to the database
    ///
    /// Notably, all feed fetches will result in at least one update to the database, to update at
    /// least next_fetch_at. A fetch indicating modified content will result in as many updates as
    /// there are new or modified items.
    async fn update_feed(
        pool: PgPool,
        feed: Feed,
        permit: Option<OwnedSemaphorePermit>,
    ) -> Result<(), anyhow::Error> {
        // HTTP request to get the feed
        let client = build_reqwest_client(false);
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
        apply_feed_update(&pool, &feed, &feed_update)
            .await
            .context("failed to apply feed update")?;

        drop(permit);

        Ok(())
    }

    pub async fn create_feed(pool: &PgPool, link: &str) -> Result<Feed, FeedCreationError> {
        // HTTP request to get the feed
        let client = build_reqwest_client(true);
        let status = fetch_feed(&client, link, None, None).await;

        // Convert result of HTTP request to a feed creation
        let (feed_insert, parsed_items) = get_feed_creation(status, link).await?;

        // Insert the feed and items into the database
        let mut tx = pool.begin().await?;

        let feed = feed_insert.insert(&mut *tx).await?;
        for item in parsed_items {
            InsertFeedItem::from_parsed(&item, feed.id, 0, Utc::now())
                .insert(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(feed)
    }

    pub async fn cancel(self) {
        // TODO: timeout and force cancel
        self.cancel_tx.send(()).unwrap();
        if let Err(err) = self.task.await {
            tracing::error!("Error while canceling feed daemon: {:?}", err);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::sql::{Feed, FeedItem, InsertFeed};
    use crate::tests::{dates::*, sql::TempDB};

    use super::Daemon;

    use chrono::{Duration, Utc};
    use ormx::{Insert, Table};

    #[tokio::test]
    async fn test_not_modified() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .with_status(304)
            .with_header("ETag", "123")
            .create();

        let pool = TempDB::new().await;

        // Insert a feed into the database
        let now_minus_15_mins = chrono::Utc::now() - chrono::Duration::minutes(15);
        let feed = InsertFeed::from_mockito(&server, now_minus_15_mins)
            .insert(&*pool)
            .await
            .unwrap();

        // Update the feed
        Daemon::update_feed(pool.clone(), feed.clone(), None)
            .await
            .unwrap();
        let updated_feed = Feed::get(&*pool, feed.id).await.unwrap();

        // Should be grabbed from the ETag header
        assert_eq!(feed.etag, None);
        assert_eq!(updated_feed.etag, Some("123".to_string()));

        // Nothing changed so created_at/updated_at shouldn't have updated
        assert!(eq_within_second(updated_feed.created_at, feed.created_at));
        assert!(eq_within_second(updated_feed.updated_at, feed.created_at));

        // Updated since the fetch was successful
        assert!(is_now_within_second(updated_feed.fetched_at));
        assert!(is_now_within_second(updated_feed.successful_fetch_at));

        // Next fetch time should be 15 minutes from now
        assert!(eq_within_second(
            updated_feed.next_fetch_at,
            Utc::now() + Duration::minutes(15)
        ));
    }

    #[tokio::test]
    async fn test_rate_limited() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .with_status(429)
            .with_header("ETag", "123")
            .with_header("Retry-After", "120")
            .create();

        let pool = TempDB::new().await;

        let now_minus_15_mins = chrono::Utc::now() - chrono::Duration::minutes(15);
        let feed = InsertFeed::from_mockito(&server, now_minus_15_mins)
            .insert(&*pool)
            .await
            .unwrap();

        // Update the feed
        Daemon::update_feed((*pool).clone(), feed.clone(), None)
            .await
            .unwrap();
        let updated_feed = Feed::get(&*pool, feed.id).await.unwrap();

        // ETag should be ignored, since we failed to fetch
        assert_eq!(feed.etag, None);
        assert_eq!(updated_feed.etag, None);

        // Nothing changed so created_at/updated_at shouldn't have updated
        assert!(eq_within_second(updated_feed.created_at, feed.created_at));
        assert!(eq_within_second(updated_feed.updated_at, feed.created_at));

        // Ensure only fetched_at has been updated since the fetch failed
        assert!(eq_within_second(updated_feed.fetched_at, Utc::now()));
        assert!(eq_within_second(
            updated_feed.successful_fetch_at,
            feed.successful_fetch_at
        ));

        // Next fetch time should be in 120 + 60 seconds (extra buffer for safety)
        // due to the Retry-After header
        assert!(eq_within_second(
            updated_feed.next_fetch_at,
            Utc::now() + Duration::seconds(120 + 60)
        ));
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server.mock("GET", "/").with_status(500).create();

        let pool = TempDB::new().await;

        let now_minus_6_days = chrono::Utc::now() - chrono::Duration::days(6);
        let feed = InsertFeed::from_mockito(&server, now_minus_6_days)
            .insert(&*pool)
            .await
            .unwrap();

        // Update the feed
        Daemon::update_feed((*pool).clone(), feed.clone(), None)
            .await
            .unwrap();
        let updated_feed = Feed::get(&*pool, feed.id).await.unwrap();

        // ETag should be ignored, since we failed to fetch
        assert_eq!(feed.etag, None);
        assert_eq!(updated_feed.etag, None);

        // Nothing changed so created_at/updated_at shouldn't have updated
        assert_within_second(updated_feed.created_at, feed.created_at);
        assert_within_second(updated_feed.updated_at, feed.created_at);

        // Ensure only fetched_at has been updated since the fetch failed
        assert_now_within_second(updated_feed.fetched_at);
        assert_within_second(updated_feed.successful_fetch_at, feed.successful_fetch_at);

        // Next fetch time should be 30 minutes from now
        // due to the last successful fetch being 6 days ago
        // (exponential backoff)
        assert_within_second(
            updated_feed.next_fetch_at,
            Utc::now() + Duration::minutes(30),
        );
    }

    #[tokio::test]
    async fn test_fetch_feed() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .with_header("Content-Type", "application/rss+xml")
            .with_status(200)
            .with_body_from_file("tests/feeds/hacker-news-rss.xml")
            .create();

        let pool = TempDB::new().await;

        let feed = InsertFeed::from_mockito(&server, Utc::now())
            .insert(&*pool)
            .await
            .unwrap();

        Daemon::update_feed((*pool).clone(), feed.clone(), None)
            .await
            .unwrap();
        let updated_feed = Feed::get(&*pool, feed.id).await.unwrap();

        assert_eq!(updated_feed.title, "Hacker News");
        assert_eq!(
            updated_feed.description,
            "Links for the intellectually curious, ranked by readers."
        );

        let items = FeedItem::get_by_feed(&*pool, &feed.id).await.unwrap();
        assert_eq!(items.len(), 1);

        // Edit the title to ensure that it updates back to the original
        let mut item = items[0].clone();
        assert_eq!(item.title, "A Brief History of Code Signing at Mozilla");
        item.title = "Hello World".to_string();
        item.update(&*pool).await.unwrap();

        // Ensure updates don't duplicate items
        Daemon::update_feed((*pool).clone(), feed.clone(), None)
            .await
            .unwrap();
        let items = FeedItem::get_by_feed(&*pool, &feed.id).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "A Brief History of Code Signing at Mozilla");
    }
}
