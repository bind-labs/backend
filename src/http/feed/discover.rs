use crate::feed::{discover::discover_feed_links, FeedInformation};
use crate::http::common::*;

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverFeedsRequest {
    #[validate(url)]
    link: String,
}

pub async fn discover_feeds(
    State(state): State<ApiContext>,
    Json(body): Json<DiscoverFeedsRequest>,
) -> Result<Json<Vec<FeedInformation>>> {
    body.validate()?;

    let html_page = state
        .reqwest_client
        .get(body.link)
        .send()
        .await?
        .text()
        .await?;

    Ok(Json(discover_feed_links(&html_page)))
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn finds_hacker_news_rss() {}

    #[tokio::test]
    async fn follows_redirects() {}
}
