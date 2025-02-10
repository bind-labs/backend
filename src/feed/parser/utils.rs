use reqwest::Url;

pub fn domain_from_link(link: &str) -> Option<String> {
    Url::parse(link)
        .ok()
        .and_then(|url| url.domain().map(|domain| domain.to_string()))
}
