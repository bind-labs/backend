mod daemon;
pub mod discover;

use serde::Serialize;

pub use crate::sql::FeedFormat;
pub mod parsed_feed;
pub mod json;

#[derive(Serialize)]
pub struct FeedInformation {
    pub link: String,
    pub format: FeedFormat,
}
