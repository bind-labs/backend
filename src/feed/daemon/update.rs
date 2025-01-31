use chrono::{Duration, Utc};
use sqlx::PgPool;

use crate::sql::{FeedItem, FeedStatus};

use super::{
    fetch::{FeedFetch, FeedFetchError},
    http::parse_cache_control_max_age,
    FeedToUpdate,
};

#[derive(Debug, Default)]
pub struct FeedUpdate {
    pub status: Option<FeedStatus>,
    pub title: Option<String>,
    pub link: Option<String>,

    pub fetched_at: Option<chrono::DateTime<chrono::Utc>>,
    pub successful_fetch_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_fetch_at: Option<chrono::DateTime<chrono::Utc>>,

    pub items: Vec<FeedItem>,
}

pub fn get_feed_update(fetch: Result<FeedFetch, FeedFetchError>, feed: FeedToUpdate) -> FeedUpdate {
    match fetch {
        Ok(FeedFetch::Modified(response)) => FeedUpdate {
            fetched_at: Some(Utc::now()),
            successful_fetch_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            items: Vec::new(),
            ..Default::default()
        },

        // Update the next fetch time
        Ok(FeedFetch::NotModified(response)) => {
            let cache_duration = response
                .headers()
                .get("Cache-Control")
                .and_then(parse_cache_control_max_age);

            FeedUpdate {
                fetched_at: Some(Utc::now()),
                successful_fetch_at: Some(Utc::now()),
                ..get_next_fetch_time(&feed, cache_duration).into()
            }
        }

        // Update the feed URL and leave it for the next run
        Ok(FeedFetch::Moved(location)) => FeedUpdate {
            link: Some(location),
            ..Default::default()
        },

        // Sets the next fetch time to the current time + duration of rate limit
        Err(FeedFetchError::RateLimited(duration)) => FeedUpdate {
            fetched_at: Some(Utc::now()),
            next_fetch_at: Some(Utc::now() + duration + Duration::minutes(1)),
            ..Default::default()
        },

        // Update the next fetch time, but don't update last successful fetch time
        _ => FeedUpdate {
            fetched_at: Some(Utc::now()),
            ..get_next_fetch_time(&feed, None).into()
        },
    }
}

pub async fn update_feed_link(pool: &PgPool, feed_id: i32, link: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE feed
        SET link = $1
        WHERE id = $2
        "#,
        link,
        feed_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub fn get_next_fetch_time(feed: &FeedToUpdate, cache_duration: Option<Duration>) -> NextUpdate {
    let time_since_successful_fetch: Duration =
        feed.successful_fetch_at.signed_duration_since(Utc::now());
    if time_since_successful_fetch > Duration::weeks(4) {
        return NextUpdate::Broken;
    }

    // TODO: handle skip_hours and skip_days

    // <= 3 days, update every 15 minutes
    // 6 days, update every 30 minutes
    // ...
    // NOTE: `updated_at` is the time since we got new content or content was updated
    let time_since_update: Duration = feed.updated_at.signed_duration_since(Utc::now());
    let desired_time_until_update = Duration::minutes(time_since_update.num_days() * 5);

    // Respect the TTL if set and the cache header
    let ttl_duration = feed
        .ttl_in_minutes
        .map(|mins| Duration::minutes(mins as i64));
    let min_time_until_update = cache_duration
        .or(ttl_duration)
        .unwrap_or(Duration::minutes(15))
        .min(Duration::minutes(15));

    // Cap at 1 day
    let max_time_until_update = Duration::days(1);

    let time_until_update = min_time_until_update
        .max(desired_time_until_update)
        .min(max_time_until_update);
    NextUpdate::Time(Utc::now() + time_until_update)
}

pub enum NextUpdate {
    Time(chrono::DateTime<Utc>),
    Broken,
}

impl Into<FeedUpdate> for NextUpdate {
    fn into(self) -> FeedUpdate {
        match self {
            NextUpdate::Time(time) => FeedUpdate {
                next_fetch_at: Some(time),
                ..Default::default()
            },
            NextUpdate::Broken => FeedUpdate {
                status: Some(FeedStatus::Broken),
                ..Default::default()
            },
        }
    }
}
