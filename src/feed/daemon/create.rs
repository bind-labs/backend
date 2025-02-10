use chrono::Utc;

use crate::feed::parser::utils::domain_from_link;
use crate::feed::parser::{
    feed_item::ParsedFeedItem, parse_feed_from_response, ParsedFromResponseError,
};
use crate::sql::InsertFeed;

use super::{
    constants::{MAX_TIME_BETWEEN_UPDATES, MIN_TIME_BETWEEN_UPDATES},
    fetch::{FeedFetch, FeedFetchError},
    http::{parse_cache_control_max_age, parse_etag},
};

#[derive(Debug, thiserror::Error)]
pub enum FeedCreationError {
    #[error("feed returned not modified during creation")]
    NotModified,
    #[error("feed redirected too many times")]
    RedirectLoop,
    #[error("feed does not exist")]
    NotFound,
    #[error(transparent)]
    ParsingError(#[from] ParsedFromResponseError),
    #[error(transparent)]
    OtherFetchError(FeedFetchError),
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
}

pub async fn get_feed_creation(
    fetch: Result<FeedFetch, FeedFetchError>,
    link: &str,
) -> Result<(InsertFeed, Vec<ParsedFeedItem>), FeedCreationError> {
    match fetch {
        Ok(FeedFetch::Modified(response)) => {
            let ttl_in_minutes = response
                .headers()
                .get("Cache-Control")
                .and_then(parse_cache_control_max_age);
            let etag = response.headers().get("ETag").and_then(parse_etag);

            let parsed_feed = parse_feed_from_response(response).await?;
            let feed_insert = InsertFeed {
                link: parsed_feed.link.to_string(),
                domain: domain_from_link(link),
                etag,
                ttl_in_minutes: ttl_in_minutes.map(|d| d.num_minutes() as i32),
                next_fetch_at: Utc::now()
                    + ttl_in_minutes
                        .unwrap_or(MIN_TIME_BETWEEN_UPDATES)
                        .min(MAX_TIME_BETWEEN_UPDATES)
                        .max(MIN_TIME_BETWEEN_UPDATES),
                ..parsed_feed.clone().into()
            };

            Ok((feed_insert, parsed_feed.items))
        }
        Ok(FeedFetch::NotModified(_)) => Err(FeedCreationError::NotModified),
        Ok(FeedFetch::Moved(_)) => Err(FeedCreationError::RedirectLoop),
        Err(FeedFetchError::NotFound) => Err(FeedCreationError::NotFound),
        Err(err) => Err(FeedCreationError::OtherFetchError(err)),
    }
}
