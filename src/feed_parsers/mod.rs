use crate::{Feed, FeedItem};

pub mod rss_parser;

pub trait FeedParser {
    fn get_feed(content: &str) -> Result<Feed, rss::Error>;
    fn get_items(content: &str) -> Result<Vec<FeedItem>, rss::Error>;
}
