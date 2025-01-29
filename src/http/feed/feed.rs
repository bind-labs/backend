use axum::{extract::State, http, response::IntoResponse, Json};
use serde::Deserialize;
use validator::Validate;

use crate::{error::ServerError, html, AppState, Feed, FeedType};

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GetFeedInfoRequest {
    #[validate(url)]
    link: String,
}

pub async fn get_feed_information(
    State(state): State<AppState>,
    Json(body): Json<GetFeedInfoRequest>,
) -> Result<impl IntoResponse, ServerError> {
    body.validate()?;

    let html_page = state
        .reqwest_client
        .get(body.link)
        .send()
        .await?
        .text()
        .await?;

    let feed_links = html::get_feed_links(&html_page);

    Ok((http::StatusCode::OK, Json(feed_links)))
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeedRequest {
    #[validate(url)]
    link: String,
    feed_type: FeedType,
    title: String,
}

pub async fn create_feed(
    State(state): State<AppState>,
    Json(body): Json<CreateFeedRequest>,
) -> Result<impl IntoResponse, ServerError> {
    body.validate()?;

    let feed_string = state
        .reqwest_client
        .get(&body.link)
        .send()
        .await?
        .text()
        .await?;

    todo!("Parse the feed string and save it to the database");
    Ok(http::StatusCode::OK)

}
