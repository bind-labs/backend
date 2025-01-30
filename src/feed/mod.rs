mod daemon;
pub mod discover;
pub mod parse;

use serde::Serialize;

pub use crate::sql::FeedType;

#[derive(Serialize)]
pub struct FeedInformation {
    pub link: String,
    #[serde(rename = "type")]
    pub type_: FeedType,
}
