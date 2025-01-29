use crate::{error::ServerError, Feed, FeedItem};

use super::FeedParser;

struct RssParser {}

impl FeedParser for RssParser {
    fn get_feed(content: &str) -> Result<Feed, rss::Error> {
        let channel = rss::Channel::read_from(content.as_bytes())?;
        todo!("Create a feed object from our channel")
    }

    fn get_items(content: &str) -> Result<Vec<FeedItem>, rss::Error> {
        todo!()
    }
}
