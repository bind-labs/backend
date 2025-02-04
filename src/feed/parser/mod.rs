use feed::ParsedFeed;
use reqwest::Response;
use rss::Channel;
use thiserror::Error;

use super::{json::JsonFeed, FeedFormat};

pub mod feed;
pub mod feed_item;

#[derive(Debug, Error)]
pub enum ParsedFeedCreationError {
    #[error("Error parsing JSON feed item")]
    JsonFeedParsingError,
    #[error("Error parsing RSS feed item")]
    RssFeedParsingError,
    #[error("Error parsing Atom feed item")]
    AtomFeedParsingError,
}

pub async fn parse_feed_from_response(
    response: Response,
) -> Result<ParsedFeed, ParsedFromResponseError> {
    let headers = response.headers().clone();
    let content_type = headers
        .get("content-type")
        .ok_or_else(|| ParsedFromResponseError::UnknownContentType)
        .and_then(|x| {
            x.to_str()
                .map_err(|_| ParsedFromResponseError::CorruptContentType)
        })
        .and_then(|x| {
            x.split(";")
                .next()
                .ok_or(ParsedFromResponseError::CorruptContentType)
        })?;

    let body = response
        .bytes()
        .await
        .map_err(|err| ParsedFromResponseError::CorruptResponseBody(err))?;

    match FeedFormat::from_content_type(content_type) {
        Some(FeedFormat::Rss) => Channel::read_from(body.as_ref())
            .map_err(ParsedFromResponseError::RssParseError)?
            .try_into()
            .map_err(ParsedFromResponseError::GenericParseError),
        Some(FeedFormat::Atom) => atom_syndication::Feed::read_from(body.as_ref())
            .map_err(ParsedFromResponseError::AtomParseError)?
            .try_into()
            .map_err(ParsedFromResponseError::GenericParseError),
        Some(FeedFormat::Json) => JsonFeed::read_from(body.as_ref())
            .map_err(ParsedFromResponseError::JsonParseError)?
            .try_into()
            .map_err(ParsedFromResponseError::GenericParseError),
        _ => Err(ParsedFromResponseError::UnknownContentType),
    }
}

#[derive(Debug, Error)]
pub enum ParsedFromResponseError {
    #[error("Unknown content type")]
    UnknownContentType,
    #[error("Corrupt content type")]
    CorruptContentType,
    #[error("Corrupt response body")]
    CorruptResponseBody(reqwest::Error),

    #[error("Error parsing RSS feed")]
    RssParseError(rss::Error),
    #[error("Error parsing Atom feed")]
    AtomParseError(atom_syndication::Error),
    #[error("Error parsing JSON feed")]
    JsonParseError(serde_json::Error),

    #[error("Error parsing feed")]
    GenericParseError(ParsedFeedCreationError),
}

#[cfg(test)]
mod tests {
    use crate::feed::json::JsonFeed;

    use super::feed::*;
    use atom_syndication::Feed;
    use rss::Channel;

    #[test]
    fn from_rss_feed() {
        let simple_feed = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <rss version="2.0">
                <channel>
                    <title>Simple Feed</title>
                    <link>https://example.com/feed</link>
                    <description>Simple feed description</description>
                </channel>
            </rss>
        "#;

        let channel = Channel::read_from(simple_feed.as_bytes()).unwrap();
        let parsed_feed = ParsedFeed::try_from(channel).unwrap();

        assert_eq!(parsed_feed.link, "https://example.com/feed");
        assert_eq!(parsed_feed.title, "Simple Feed");
        assert_eq!(parsed_feed.description, "Simple feed description");
        assert_eq!(parsed_feed.items.len(), 0);
        assert_eq!(parsed_feed.icon, None);
    }

    #[test]
    fn from_rss_feed_with_items() {
        let feed_with_items = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <rss version="2.0">
                <channel>
                    <title>Feed with items</title>
                    <link>https://example.com/feed</link>
                    <description>Feed with items description</description>
                    <item>
                        <title>Item 1</title>
                        <link>https://example.com/item1</link>
                        <description>Item 1 description</description>
                    </item>
                    <item>
                        <title>Item 2</title>
                        <link>https://example.com/item2</link>
                        <description>Item 2 description</description>
                        <enclosure url="https://example.com/item2.mp3" length="1024" type="audio/mpeg" />
                    </item>
                </channel>
            </rss>
        "#;

        let channel = Channel::read_from(feed_with_items.as_bytes()).unwrap();
        let parsed_feed = ParsedFeed::try_from(channel).unwrap();

        assert!(parsed_feed.items.len() == 2);
        assert_eq!(parsed_feed.items[1].title, "Item 2");
        assert_eq!(
            parsed_feed.items[1].link,
            Some("https://example.com/item2".to_string())
        );
        assert_eq!(
            parsed_feed.items[1].description,
            Some("Item 2 description".to_string())
        );
    }

    #[test]
    fn from_atom_feed() {
        let simple_feed = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <feed xmlns="http://www.w3.org/2005/Atom">
                <title>Simple Feed</title>
                <id>https://example.com/feed</id>
                <subtitle>Simple feed description</subtitle>
            </feed>
        "#;

        let feed = Feed::read_from(simple_feed.as_bytes()).unwrap();
        let parsed_feed = ParsedFeed::try_from(feed).unwrap();

        assert_eq!(parsed_feed.link, "https://example.com/feed");
        assert_eq!(parsed_feed.title, "Simple Feed");
        assert_eq!(parsed_feed.description, "Simple feed description");
        assert_eq!(parsed_feed.items.len(), 0);
        assert_eq!(parsed_feed.icon, None);
    }

    #[test]
    fn from_atom_feed_with_items() {
        let feed_with_items = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <feed xmlns="http://www.w3.org/2005/Atom">
                <title>Feed with items</title>
                <id>https://example.com/feed</id>
                <subtitle>Feed with items description</subtitle>
                <entry>
                    <title>Item 1</title>
                    <id>https://example.com/item1</id>
                    <summary>Item 1 description</summary>
                </entry>
                <entry>
                    <title>Item 2</title>
                    <id>https://example.com/item2</id>
                    <summary>Item 2 description</summary>
                    <link rel="enclosure" href="https://example.com/item2.mp3" length="1024" type="audio/mpeg" />
                </entry>
            </feed>"#;

        let feed = Feed::read_from(feed_with_items.as_bytes()).unwrap();
        let parsed_feed = ParsedFeed::try_from(feed).unwrap();

        assert!(parsed_feed.items.len() == 2);
        assert_eq!(parsed_feed.items[1].title, "Item 2");
        assert_eq!(
            parsed_feed.items[1].link,
            Some("https://example.com/item2".to_string())
        )
    }

    #[test]
    fn from_json_feed() {
        let simple_feed = r#"
            {
                "version": "https://jsonfeed.org/version/1",
                "title": "Simple Feed",
                "home_page_url": "https://example.com/feed",
                "description": "Simple feed description",
                "items": []
            }
        "#;

        let feed = JsonFeed::read_from(simple_feed.as_bytes()).unwrap();
        let parsed_feed = ParsedFeed::try_from(feed).unwrap();

        assert_eq!(parsed_feed.link, "https://example.com/feed");
        assert_eq!(parsed_feed.title, "Simple Feed");
        assert_eq!(parsed_feed.description, "Simple feed description");
        assert_eq!(parsed_feed.items.len(), 0);
        assert_eq!(parsed_feed.icon, None);
    }

    #[test]
    fn from_json_feed_with_items() {
        let feed_with_items = r#"
        {
            "version": "https://jsonfeed.org/version/1.1",
            "title": "My Example Feed",
            "home_page_url": "https://example.org/",
            "feed_url": "https://example.org/feed.json",
            "items": [
                {
                    "id": "2",
                    "content_text": "This is a second item.",
                    "url": "https://example.org/second-item"
                },
                {
                    "id": "1",
                    "title": "My Example Feed Item",
                    "content_html": "<p>Hello, world!</p>",
                    "url": "https://example.org/initial-post"
                }
            ]
        }"#;

        let feed = JsonFeed::read_from(feed_with_items.as_bytes()).unwrap();

        let parsed_feed = ParsedFeed::try_from(feed).unwrap();

        assert!(parsed_feed.items.len() == 2);
        assert_eq!(parsed_feed.items[1].title, "My Example Feed Item");
        assert_eq!(
            parsed_feed.items[1].link,
            Some("https://example.org/initial-post".to_string())
        );
        assert_eq!(
            parsed_feed.items[1].content,
            Some("<p>Hello, world!</p>".to_string())
        );
    }
}
