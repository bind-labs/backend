// mod daemon;
pub mod discover;

use serde::{Deserialize, Serialize};

pub use crate::sql::FeedFormat;
pub mod parsed_feed;
pub mod json;

#[derive(Serialize, Debug, Deserialize)]
pub struct FeedInformation {
    pub link: String,
    pub format: FeedFormat,
}
