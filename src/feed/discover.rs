use scraper::{Html, Selector};

use super::{FeedInformation, FeedType};

/// Parses an HTML document and searches for feed links of all kind such as RSS, Atom, JSON etc.
pub fn discover_feed_links(html: &str) -> Vec<FeedInformation> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("link[type]").unwrap();

    document
        .select(&selector)
        .filter_map(|element| {
            let link = element.value().attr("href")?;
            let type_ = match element.value().attr("type") {
                Some("application/atom+xml") => FeedType::Atom,
                Some("application/rss+xml") => FeedType::Rss,
                Some("application/json") => FeedType::Json,
                _ => return None,
            };

            Some(FeedInformation {
                link: link.to_string(),
                type_,
            })
        })
        .collect()
}
