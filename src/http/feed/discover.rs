use crate::feed::{discover::discover_feed_links, FeedInformation};
use crate::http::auth::AuthUser;
use crate::http::common::*;

#[derive(Deserialize, Serialize, Validate)]
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
    use super::*;
    use axum::{http::Request, routing::post, Router};
    use http_body_util::BodyExt;
    use pgtemp::PgTempDB;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    #[tokio::test]
    #[ignore]
    async fn finds_hacker_news_rss() {
        let hacker_news_url = "https://news.ycombinator.com/";
        let db = PgTempDB::async_new().await;
        let pool = PgPoolOptions::new()
            .connect(&db.connection_uri())
            .await
            .unwrap();

        let state = ApiContext::new(pool);

        let router = Router::new()
            .route("/feed/discover", post(discover_feeds))
            .with_state(state);

        let response = router
            .oneshot(
                Request::post("/feed/discover")
                    .header("content-type", "application/json")
                    .body(
                        serde_json::to_string(&DiscoverFeedsRequest {
                            link: hacker_news_url.to_string(),
                        })
                        .unwrap(),
                    )
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let response_body = response.into_body().collect().await.unwrap().to_bytes();
        let feeds = serde_json::from_slice::<Vec<FeedInformation>>(&response_body).unwrap();

        println!("{:?}", feeds);
    }

    #[tokio::test]
    async fn follows_redirects() {}
}
