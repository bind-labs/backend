mod daemon;
pub mod discover;
pub mod parse;

pub use crate::sql::FeedType;

pub struct FeedInformation {
    pub link: String,
    pub type_: FeedType,
}
