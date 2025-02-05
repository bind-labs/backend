use std::io::BufRead;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct Attachment {
    pub url: String,
    pub mime_type: String,
    pub title: Option<String>,
    pub size_in_bytes: Option<u64>,
    pub duration_in_seconds: Option<u64>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Hub {
    #[serde(rename = "type")]
    pub type_: String,
    url: String,
}
#[derive(Deserialize, Debug, Serialize)]
pub struct Author {
    pub name: Option<String>,
    pub url: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct JsonFeedItem {
    pub id: String,
    pub url: Option<String>,
    pub external_url: Option<String>,
    pub title: Option<String>,
    pub content_text: Option<String>,
    pub content_html: Option<String>,
    pub summary: Option<String>,
    pub image: Option<String>,
    pub banner_image: Option<String>,
    pub date_published: Option<DateTime<Utc>>,
    pub date_modified: Option<DateTime<Utc>>,
    pub author: Option<Author>,
    pub tags: Option<Vec<String>>,
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct JsonFeed {
    pub version: String,
    pub title: String,
    pub items: Vec<JsonFeedItem>,
    pub home_page_url: Option<String>,
    pub feed_url: Option<String>,
    pub description: Option<String>,
    pub user_comment: Option<String>,
    pub next_url: Option<String>,
    pub icon: Option<String>,
    pub favicon: Option<String>,
    pub author: Option<Author>,
    pub expired: Option<bool>,
    pub hubs: Option<Vec<Hub>>,
}

impl JsonFeed {
    pub fn read_from<B: BufRead>(reader: B) -> Result<JsonFeed, serde_json::Error> {
        serde_json::from_reader(reader)
    }
}
