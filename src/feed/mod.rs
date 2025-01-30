mod daemon;
pub mod discover;

use serde::Serialize;

pub use crate::sql::FeedFormat;
pub mod parsed_feed;
pub mod json;
pub use crate::sql::FeedType;

#[derive(Serialize)]
pub struct FeedInformation {
    pub link: String,
    pub format: FeedFormat,
}
