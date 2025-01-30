use crate::feed::json::JsonFeed;

use super::{feed_item::ParsedFeedItem, ParsedFeedCreationError};

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

// From RSS to ParsedFeed

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

impl TryFrom<JsonFeed> for ParsedFeed {
    type Error = ParsedFeedCreationError;
    fn try_from(value: JsonFeed) -> Result<Self, Self::Error> {
        let mut items = Vec::new();
        for item in value.items {
            items.push(ParsedFeedItem::try_from(item)?);
        }
        Ok(Self {
            link: value
                .feed_url
                .or(value.home_page_url)
                .ok_or(ParsedFeedCreationError::JsonFeedParsingError)?,
            title: value.title,
            description: value.description.unwrap_or_default(),
            icon: value.icon,
            skip_hours: Vec::new(),
            skip_days_of_week: Vec::new(),
            ttl_in_minutes: 0,
            items,
        })
    }
}
