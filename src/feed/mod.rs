pub mod daemon;
pub mod discover;
pub mod json;
pub mod parser;

use reqwest::Url;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use crate::sql::FeedFormat;

/// Information about a discovered feed
#[derive(Serialize, Debug, Deserialize, ToSchema)]
pub struct FeedInformation {
    /// URL of the feed
    pub url: String,
    /// Format of the feed (RSS, Atom, or JSON)
    pub format: FeedFormat,
}
