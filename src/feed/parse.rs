use core::fmt;
use thiserror::Error;

use crate::FeedItemEnclosure;

pub struct ParsedFeed {
    pub link: String,
    pub title: String,
    pub description: String,
    pub icon: Option<String>,

    pub skip_hours: Vec<u32>,
    pub skip_days_of_week: Vec<u32>,

    pub ttl_in_minutes: i32,
    pub items: Vec<ParsedFeedItem>,
}

pub struct ParsedFeedItem {
    pub guid: Option<String>,
    pub title: String,
    pub link: Option<String>,
    pub description: Option<String>,
    pub enclosure: Option<FeedItemEnclosure>,
    pub content: Option<String>,
    pub categories: Vec<String>,
    pub comments_link: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Error)]
pub struct ParsedFeedCreationError {}
impl fmt::Display for ParsedFeedCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to create parsed feed")
    }
}

// From RSS to ParsedFeed
impl TryFrom<rss::Item> for ParsedFeedItem {
    type Error = ParsedFeedCreationError;
    fn try_from(value: rss::Item) -> Result<Self, Self::Error> {
        let guid = value.guid().map(|guid| guid.value().to_string());
        // if title is not present then use description as title
        // as per rss spec one of the title or description should be present
        let title = value
            .title()
            .map(|title| title.to_string())
            .unwrap_or_else(|| {
                value
                    .description()
                    .map(|description| description.to_string())
                    .unwrap_or_default()
            });

        let enclosure = value.enclosure().map(|enclosure| FeedItemEnclosure {
            url: enclosure.url.clone(),
            length: enclosure.length.clone().parse::<i32>().unwrap_or(0),
            mime_type: enclosure.mime_type.clone(),
        });

        Ok(Self {
            guid,
            title,
            link: value.link,
            description: value.description,
            enclosure,
            content: value.content.map(|content| content.to_string()),
            categories: Vec::new(),
            comments_link: value.comments.map(|comments| comments.to_string()),
            published_at: value
                .pub_date
                .map(|date| date.parse::<chrono::DateTime<chrono::Utc>>())
                .transpose()
                .map_err(|_| ParsedFeedCreationError {})?,
        })
    }
}

impl TryFrom<rss::Channel> for ParsedFeed {
    type Error = ParsedFeedCreationError;
    fn try_from(value: rss::Channel) -> Result<Self, Self::Error> {
        let mut items = Vec::new();
        for item in value.items() {
            items.push(ParsedFeedItem::try_from(item.clone())?);
        }

        let skip_hours: Vec<u32> = value
            .skip_hours()
            .into_iter()
            .filter_map(|x| {
                x.parse::<u32>()
                    .ok()
                    .and_then(|x| if x < 24 { Some(x) } else { None })
            })
            .collect::<Vec<_>>();

        let skip_days_of_week: Vec<u32> = value
            .skip_days()
            .into_iter()
            .filter_map(|x| match x.to_lowercase().as_str() {
                "sunday" => Some(0),
                "monday" => Some(1),
                "tuesday" => Some(2),
                "wednesday" => Some(3),
                "thursday" => Some(4),
                "friday" => Some(5),
                "saturday" => Some(6),
                _ => None,
            })
            .collect::<Vec<_>>();

        Ok(Self {
            link: value.link,
            title: value.title,
            description: value.description,
            icon: value.image.as_ref().map(|image| image.url.clone()),
            skip_hours,
            skip_days_of_week,
            ttl_in_minutes: 0,
            items,
        })
    }
}

// From atom item to ParsedFeedItem
impl TryFrom<atom_syndication::Entry> for ParsedFeedItem {
    type Error = ParsedFeedCreationError;
    fn try_from(value: atom_syndication::Entry) -> Result<Self, Self::Error> {
        let enclosure: Option<FeedItemEnclosure> = value
            .links
            .iter()
            .find(|link| link.rel == "enclosure")
            .map(|link| FeedItemEnclosure {
                url: link.href.clone(),
                length: link
                    .length
                    .clone()
                    .unwrap_or_default()
                    .parse::<i32>()
                    .unwrap_or(0),
                mime_type: link.mime_type.clone().unwrap_or_default(),
            });

        let comments_link = value
            .links
            .iter()
            .find(|link| link.rel == "comments")
            .map(|link| link.href.clone());

        // atom content can either have a value or an src attribute which is a link to the content
        let content = value
            .content
            .map(|content| content.value.or_else(|| content.src))
            .flatten();

        Ok(Self {
            guid: Some(value.id.clone()),
            title: value.title.value,
            link: Some(value.id),
            description: value.summary.map(|summary| summary.value),
            enclosure,
            comments_link,
            published_at: value.published.map(|published| published.into()),
            content,
            categories: Vec::new(),
        })
    }
}

// From atom to ParsedFeed
impl TryFrom<atom_syndication::Feed> for ParsedFeed {
    type Error = ParsedFeedCreationError;
    fn try_from(value: atom_syndication::Feed) -> Result<Self, Self::Error> {
        let mut items = Vec::new();
        for entry in value.entries {
            items.push(ParsedFeedItem::try_from(entry)?);
        }
        Ok(Self {
            link: value.id,
            title: value.title.value,
            description: value
                .subtitle
                .map(|subtitle| subtitle.value)
                .unwrap_or_default(),
            icon: value.icon,
            skip_hours: Vec::new(),
            skip_days_of_week: Vec::new(),
            ttl_in_minutes: 0,
            items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
