use chrono::{DateTime, Utc};

use crate::{feed::json::JsonFeed, sql::FeedFormat};

use super::{feed_item::ParsedFeedItem, utils::domain_from_link, ParsedFeedCreationError};

#[derive(Debug, Clone)]
pub struct ParsedFeed {
    pub format: FeedFormat,
    pub link: String,
    pub domain: Option<String>,
    pub title: String,
    pub description: String,
    pub icon: Option<String>,

    pub skip_hours: Vec<i32>,
    pub skip_days_of_week: Vec<i32>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,

    pub ttl_in_minutes: i32,
    pub items: Vec<ParsedFeedItem>,
}

impl TryFrom<rss::Channel> for ParsedFeed {
    type Error = ParsedFeedCreationError;
    fn try_from(value: rss::Channel) -> Result<Self, Self::Error> {
        let mut items = Vec::new();
        for item in value.items() {
            items.push(ParsedFeedItem::try_from(item.clone())?);
        }

        let skip_hours: Vec<i32> = value
            .skip_hours()
            .iter()
            .filter_map(|x| {
                x.parse::<i32>()
                    .ok()
                    .and_then(|x| if x < 24 { Some(x) } else { None })
            })
            .collect::<Vec<_>>();

        let skip_days_of_week: Vec<i32> = value
            .skip_days()
            .iter()
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
            format: FeedFormat::Rss,
            link: value.link.clone(),
            domain: domain_from_link(&value.link),
            title: value.title,
            description: value.description,
            icon: value.image.as_ref().map(|image| image.url.clone()),
            skip_hours,
            skip_days_of_week,
            updated_at: value
                .last_build_date
                .and_then(|date| DateTime::parse_from_rfc2822(&date).ok())
                .map(|date| date.with_timezone(&Utc)),
            ttl_in_minutes: 0,
            items,
        })
    }
}

impl TryFrom<atom_syndication::Feed> for ParsedFeed {
    type Error = ParsedFeedCreationError;
    fn try_from(value: atom_syndication::Feed) -> Result<Self, Self::Error> {
        // TODO: add support for paged and completed feeds
        let mut items = Vec::new();
        for entry in value.entries {
            items.push(ParsedFeedItem::try_from(entry)?);
        }
        Ok(Self {
            format: FeedFormat::Atom,
            link: value.id.clone(),
            domain: domain_from_link(&value.id),
            title: value.title.value,
            description: value
                .subtitle
                .map(|subtitle| subtitle.value)
                .unwrap_or_default(),
            icon: value.icon,
            skip_hours: Vec::new(),
            skip_days_of_week: Vec::new(),
            updated_at: Some(value.updated.with_timezone(&Utc)),
            ttl_in_minutes: 0,
            items,
        })
    }
}

impl TryFrom<JsonFeed> for ParsedFeed {
    type Error = ParsedFeedCreationError;
    fn try_from(value: JsonFeed) -> Result<Self, Self::Error> {
        let mut items = Vec::new();
        for item in value.items {
            items.push(ParsedFeedItem::try_from(item)?);
        }

        let link = value
            .feed_url
            .or(value.home_page_url)
            .ok_or(ParsedFeedCreationError::JsonFeedParsingError)?;

        Ok(Self {
            format: FeedFormat::Json,
            link: link.clone(),
            domain: domain_from_link(&link),
            title: value.title,
            description: value.description.unwrap_or_default(),
            icon: value.icon,
            skip_hours: Vec::new(),
            skip_days_of_week: Vec::new(),
            updated_at: None,
            ttl_in_minutes: 0,
            items,
        })
    }
}
