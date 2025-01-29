use atom_syndication::Feed;
use chrono::format::Parsed;
use rss::Channel;

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

// From RSS to ParsedFeed
impl TryFrom<rss::Item> for ParsedFeedItem {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: rss::Item) -> Result<Self, Box<dyn std::error::Error>> {
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
            length: enclosure.length.clone().parse::<u32>().unwrap_or(0),
            mime_type: enclosure.mime_type.clone(),
        });

        Ok(ParsedFeedItem {
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
                .transpose()?,
        })
    }
}

impl TryFrom<rss::Channel> for ParsedFeed {
    type Error = Box<dyn std::error::Error>;
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

        Ok(ParsedFeed {
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

// From Atom to ParsedFeed
// impl TryFrom<atom_syndication::Feed> for ParsedFeed {
//     type Error = Box<dyn std::error::Error>;
//     fn try_from(value: atom_syndication::Feed) -> Result<Self, Self::Error> {
//         let mut items = Vec::new();
//         for entry in value.entries() {
//             items.push(ParsedFeedItem {
//                 guid: entry.id().map(|id| id.to_string()),
//                 title: entry.title().to_string(),
//                 link: entry.links().first().map(|link| link.href().to_string()),
//                 description: entry.summary().map(|summary| summary.to_string()),
//                 enclosure: None,
//                 content: entry.content().map(|content| content.value().to_string()),
//                 categories: entry
//                     .categories()
//                     .iter()
//                     .map(|category| category.term().to_string())
//                     .collect(),
//                 comments_link: None,
//                 published_at: entry
//                     .published()
//                     .map(|date| date.to_rfc3339())
//                     .map(|date| date.parse::<chrono::DateTime<chrono::Utc>>())
//                     .transpose()?,
//             });
//         }

//         Ok(ParsedFeed {
//             link: value
//                 .links()
//                 .first()
//                 .map(|link| link.href().to_string())
//                 .unwrap_or_default(),
//             title: value.title().to_string(),
//             description: value
//                 .subtitle()
//                 .map(|subtitle| subtitle.to_string())
//                 .unwrap_or_default(),
//             icon: value.logo().map(|logo| logo.to_string()),
//             skip_hours: Vec::new(),
//             skip_days_of_week: Vec::new(),
//             ttl_in_minutes: 0,
//             items,
//         })
//     }
// }
