use chrono::Duration;
use reqwest::{Response, StatusCode};

use super::http::parse_retry_after;

const USER_AGENT: &str = concat!("Bind/", env!("CARGO_PKG_VERSION"));

pub fn build_reqwest_client(follow_redirects: bool) -> reqwest::Client {
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
        .redirect(if !follow_redirects {
            reqwest::redirect::Policy::custom(|attempt| {
                if attempt.status() == StatusCode::MOVED_PERMANENTLY {
                    attempt.stop()
                } else if attempt.previous().len() > 5 {
                    attempt.error("too many redirects")
                } else {
                    attempt.follow()
                }
            })
        } else {
            reqwest::redirect::Policy::limited(20)
        })
        .build()
        .unwrap()
}

pub async fn fetch_feed(
    client: &reqwest::Client,
    link: &str,
    updated_at: Option<&chrono::DateTime<chrono::Utc>>,
    etag: Option<&str>,
) -> Result<FeedFetch, FeedFetchError> {
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
        StatusCode::OK => Ok(FeedFetch::Modified(response)),
        StatusCode::NOT_MODIFIED => Ok(FeedFetch::NotModified(response)),

        StatusCode::MOVED_PERMANENTLY => {
            let location = response
                .headers()
                .get("Location")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
                .ok_or(FeedFetchError::MovedWithoutLocation)?;
            Ok(FeedFetch::Moved(location))
        }

        // Errors
        StatusCode::NOT_FOUND => Err(FeedFetchError::NotFound),
        StatusCode::BAD_REQUEST => Err(FeedFetchError::BadRequest),
        StatusCode::FORBIDDEN => Err(FeedFetchError::Forbidden),

        StatusCode::TOO_MANY_REQUESTS => Err(FeedFetchError::RateLimited(
            response
                .headers()
                .get("Retry-After")
                .and_then(parse_retry_after)
                // Default to 1 hour
                .unwrap_or(Duration::hours(1)),
        )),

        status if status.as_u16() >= 500 && status.as_u16() <= 599 => {
            Err(FeedFetchError::ServerError(status))
        }

        status => Err(FeedFetchError::UnexpectedError(status)),
    }
}

#[derive(Debug)]
pub enum FeedFetch {
    Modified(Response),
    NotModified(Response),
    Moved(String),
}

#[derive(Debug, thiserror::Error)]
pub enum FeedFetchError {
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

type Result<T, E = FeedFetch> = std::result::Result<T, E>;
