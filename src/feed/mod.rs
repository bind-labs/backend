// mod daemon;
pub mod discover;

use reqwest::Url;
use serde::{Deserialize, Serialize};

pub use crate::sql::FeedFormat;
pub mod daemon;
pub mod json;
pub mod parser;

#[derive(Serialize, Debug, Deserialize)]
pub struct FeedInformation {
    pub url: Url,
    pub format: FeedFormat,
}
