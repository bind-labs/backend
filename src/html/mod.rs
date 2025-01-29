use scraper::{Html, Selector};

use crate::{FeedInformation, FeedType};

/// Parses an HTML document and searches for feed links of all kind such as RSS, Atom, JSON etc.
pub fn get_feed_links(html: &str) -> Vec<FeedInformation> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("link[type]").unwrap();

    document
        .select(&selector)
        .filter_map(|element| {
            let link = element.value().attr("href")?;
            let feed_type = match element.value().attr("type") {
                Some("application/atom+xml") => FeedType::Atom,
                Some("application/rss+xml") => FeedType::Rss,
                Some("application/json") => FeedType::Json,
                _ => return None,
            };

            Some(FeedInformation {
                link: link.to_string(),
                feed_type: feed_type,
            })
        })
        .collect()
}
