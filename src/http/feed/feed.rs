use axum::{extract::State, http, response::IntoResponse, Json};
use serde::Deserialize;
use validator::Validate;

use crate::{error::ServerError, feed::ParsedFeed, html, AppState, FeedType};

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

    let parsed_feed = match body.feed_type {
        FeedType::Atom => {
            ParsedFeed::try_from(atom_syndication::Feed::read_from(feed_string.as_bytes())?)?
        }
        FeedType::Rss => ParsedFeed::try_from(rss::Channel::read_from(feed_string.as_bytes())?)?,
        _ => unimplemented!(),
    };

    // store queries in the database

    todo!("Parse the feed string and save it to the database");
    Ok(http::StatusCode::OK)
}
