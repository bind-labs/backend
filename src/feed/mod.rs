// mod daemon;
pub mod discover;

use serde::{Deserialize, Serialize};

pub use crate::sql::FeedFormat;
pub mod daemon;
pub mod json;
pub mod parser;

#[derive(Serialize, Debug, Deserialize)]
pub struct FeedInformation {
    pub link: String,
    pub format: FeedFormat,
}
