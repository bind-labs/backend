use scraper::{Html, Selector};

use super::{FeedFormat, FeedInformation};

/// Parses an HTML document and searches for feed links of all kind such as RSS, Atom, JSON etc.
pub fn discover_feed_links(html: &str) -> Vec<FeedInformation> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("link[type]").unwrap();

    document
        .select(&selector)
        .filter_map(|element| {
            let link = element.value().attr("href")?;
            let format = match element.value().attr("type") {
                Some("application/atom+xml") => FeedFormat::Atom,
                Some("application/rss+xml") => FeedFormat::Rss,
                Some("application/json") => FeedFormat::Json,
                _ => return None,
            };

            Some(FeedInformation {
                link: link.to_string(),
                format,
            })
        })
        .collect()
}
