use crate::{feed::json::JsonFeedItem, sql::FeedItemEnclosure};

use super::ParsedFeedCreationError;

#[derive(Debug)]
pub struct ParsedFeedItem {
    pub guid: String,
    pub link: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub enclosure: Option<FeedItemEnclosure>,
    pub content: Option<String>,
    pub categories: Vec<String>,
    pub comments_link: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<rss::Item> for ParsedFeedItem {
    type Error = ParsedFeedCreationError;
    fn try_from(value: rss::Item) -> Result<Self, Self::Error> {
        let guid = value
            .guid()
            .map(|guid| guid.value())
            .or(value.link())
            // because if you don't include guid or link, wtf is wrong with you
            .or(value.title())
            .or(value.description())
            .map(|val| val.to_string())
            .ok_or_else(|| ParsedFeedCreationError::RssFeedParsingError)?;

        // if title is not present then use description as title
        // as per rss spec one of the title or description should be present
        let title = value
            .title()
            .or(value.description())
            .map(|val| val.to_string())
            .ok_or_else(|| ParsedFeedCreationError::RssFeedParsingError)?;

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
                .map_err(|_| ParsedFeedCreationError::RssFeedParsingError)?,
        })
    }
}

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
            guid: value.id.clone(),
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

impl TryFrom<JsonFeedItem> for ParsedFeedItem {
    type Error = ParsedFeedCreationError;
    fn try_from(value: JsonFeedItem) -> Result<Self, Self::Error> {
        let content = value.content_text.or(value.content_html);
        let title = value
            .title
            .or(content.clone())
            .ok_or(ParsedFeedCreationError::JsonFeedParsingError)?;
        let enclosure = value
            .attachments
            .map(|attachments| {
                attachments.get(0).map(|attachment| FeedItemEnclosure {
                    url: attachment.url.clone(),
                    length: attachment.size_in_bytes.unwrap_or_default() as i32,
                    mime_type: attachment.mime_type.clone(),
                })
            })
            .flatten();

        Ok(Self {
            guid: value.id,
            title,
            link: value.url,
            description: value.summary,
            enclosure,
            content,
            categories: value.tags.unwrap_or_default(),
            comments_link: value.external_url,
            published_at: value.date_published,
        })
    }
}
