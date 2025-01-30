use std::sync::Arc;

use chrono::Duration;
use reqwest::{Response, StatusCode};
use sqlx::PgPool;
use tokio::sync::Semaphore;

use crate::sql::{Feed, FeedFormat, FeedStatus};

struct FeedToUpdate {
    pub id: i32,
    pub status: FeedStatus,
    pub format: FeedFormat,
    pub link: String,

    pub skip_hours: Vec<i32>,
    pub skip_days_of_week: Vec<i32>,
    pub ttl_in_minutes: i32,
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
        WHERE next_fetch_at < NOW()"#,
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
            let feed_status = fetch_feed(
                &client,
                &feed.link,
                Some(&feed.updated_at),
                feed.etag.as_deref(),
            )
            .await;

            match feed_status {
                Ok(FeedUpdate::Modified(response)) => {
                    todo!("Parse and update feed")
                }
                Ok(FeedUpdate::NotModified) => {
                    todo!("Set new next_fetch_at time")
                }
                Ok(FeedUpdate::Moved(location)) => {
                    todo!("Update feed URL and leave it for the next run")
                }

                Err(FeedUpdateError::RateLimited(duration)) => {
                    todo!("Set the next_fetch_at time to the current time + duration")
                }
                _ => todo!(
                    "Set the status to 'broken' if we haven't successfully fetched for a week+. Set next_fetch_at with backoff"
                ),
            };

            drop(permit);
        }));
    }

    Ok(())
}

const USER_AGENT: &str = concat!("Bind/", env!("CARGO_PKG_VERSION"));

fn build_reqwest_client() -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static(
            "application/rss+xml, application/xml, application/atom+xml, application/json, text/xml;q=0.9",
        ),
    );
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static(USER_AGENT),
    );
    reqwest::Client::builder()
        .default_headers(headers)
        // Don't follow permanent redirects so that we can update the feed URL
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.status() == StatusCode::MOVED_PERMANENTLY {
                attempt.stop()
            } else if attempt.previous().len() > 5 {
                attempt.error("too many redirects")
            } else {
                attempt.follow()
            }
        }))
        .build()
        .unwrap()
}

pub async fn fetch_feed(
    client: &reqwest::Client,
    link: &str,
    updated_at: Option<&chrono::DateTime<chrono::Utc>>,
    etag: Option<&str>,
) -> Result<FeedUpdate, FeedUpdateError> {
    let mut request = client.get(link)
        .timeout(std::time::Duration::from_secs(30))
        .header(
        "Accept",
        "application/rss+xml, application/xml, application/atom+xml, application/json, text/xml;q=0.9",
    );
    if let Some(updated_at) = updated_at {
        request = request.header("If-Modified-Since", updated_at.to_rfc2822());
    }
    if let Some(etag) = etag {
        request = request.header("If-None-Match", etag);
    }
    let response = request.send().await?;

    match response.status() {
        // Success
        StatusCode::OK => Ok(FeedUpdate::Modified(response)),
        StatusCode::NOT_MODIFIED => Ok(FeedUpdate::NotModified),

        StatusCode::MOVED_PERMANENTLY => {
            let location = response
                .headers()
                .get("Location")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
                .ok_or(FeedUpdateError::MovedWithoutLocation)?;
            Ok(FeedUpdate::Moved(location))
        }

        // Errors
        StatusCode::NOT_FOUND => Err(FeedUpdateError::NotFound),
        StatusCode::BAD_REQUEST => Err(FeedUpdateError::BadRequest),
        StatusCode::FORBIDDEN => Err(FeedUpdateError::Forbidden),

        StatusCode::TOO_MANY_REQUESTS => Err(FeedUpdateError::RateLimited(
            response
                .headers()
                .get("Retry-After")
                .and_then(parse_retry_after)
                // Default to 30 minutes
                .unwrap_or(Duration::minutes(30)),
        )),

        status if status.as_u16() >= 500 && status.as_u16() <= 599 => {
            Err(FeedUpdateError::ServerError(status))
        }

        status => Err(FeedUpdateError::UnexpectedError(status)),
    }
}

pub fn parse_retry_after(header: &reqwest::header::HeaderValue) -> Option<Duration> {
    let retry_after = header.to_str().ok()?;

    let from_seconds = retry_after.parse::<i64>().ok().map(Duration::seconds);
    let from_date = chrono::DateTime::parse_from_rfc2822(retry_after)
        .ok()
        .map(|d| d.signed_duration_since(chrono::Utc::now()));

    from_seconds.or(from_date)
}

#[derive(Debug)]
enum FeedUpdate {
    Modified(Response),
    NotModified,
    Moved(String),
}

#[derive(Debug, thiserror::Error)]
enum FeedUpdateError {
    #[error("Feed no longer exists")]
    NotFound,
    #[error("Sent a bad request to the server")]
    BadRequest,
    #[error("Not allowed to access the feed")]
    Forbidden,

    #[error("Feed is rate limited")]
    RateLimited(Duration),

    #[error("Feed moved without providing a new location")]
    MovedWithoutLocation,

    #[error("Feed server failed with status code: {0}")]
    ServerError(StatusCode),

    #[error("Feed server responded with an unexpected status code: {0}")]
    UnexpectedError(StatusCode),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

type Result<T, E = FeedUpdate> = std::result::Result<T, E>;
