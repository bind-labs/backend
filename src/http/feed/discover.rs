use bind_macros::IntoRequest;
use reqwest::Url;

use crate::feed::{discover::discover_feed_links, FeedInformation};
use crate::http::common::*;

#[derive(Deserialize, Serialize, Validate, IntoRequest)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverFeedsRequest {
    #[validate(url)]
    link: String,
}

pub async fn discover_feeds(
    _: AuthUser,
    State(state): State<ApiContext>,
    Json(body): Json<DiscoverFeedsRequest>,
) -> Result<Json<Vec<FeedInformation>>> {
    body.validate()?;

    let url = Url::parse(&body.link).map_err(|_| Error::BadRequest("Invalid URL".to_string()))?;

    let response = state.reqwest_client.get(url.clone()).send().await?;
    let final_url = response.url().clone(); // We may have redirected, so get the final URL
    let html_page = response.text().await?;

    Ok(Json(discover_feed_links(&final_url, &html_page)))
}

#[cfg(test)]
mod test {
    use crate::tests::TestContext;

    use super::*;
    use axum::http::method::Method;

    #[tokio::test]
    async fn finds_hacker_news_rss() {
        let hacker_news_url = "https://news.ycombinator.com/";

        let ctx = TestContext::new().await;
        let request = DiscoverFeedsRequest {
            link: hacker_news_url.to_string(),
        };
        let response = ctx
            .req(request.into_request(Method::POST, "/feed/discover"))
            .await;
        assert_eq!(response.status(), 200);

        let feeds: Vec<FeedInformation> = ctx.decode(response).await;
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0].url.to_string(), "https://news.ycombinator.com/rss");
    }

    #[tokio::test]
    async fn follows_redirects() {
        let hacker_news_url = "https://hacker.news/";

        let ctx = TestContext::new().await;
        let request = DiscoverFeedsRequest {
            link: hacker_news_url.to_string(),
        };
        let response = ctx
            .req(request.into_request(Method::POST, "/feed/discover"))
            .await;
        assert_eq!(response.status(), 200);

        let feeds: Vec<FeedInformation> = ctx.decode(response).await;
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0].url.to_string(), "https://news.ycombinator.com/rss");
    }
}
