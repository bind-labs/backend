mod daemon;
pub mod discover;
pub mod parse;

use serde::Serialize;

pub use crate::sql::FeedFormat;

#[derive(Serialize)]
pub struct FeedInformation {
    pub link: String,
    pub format: FeedFormat,
}
