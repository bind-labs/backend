use axum::http;
use axum::response::IntoResponse;

use crate::feed::daemon::Daemon;
use crate::http::common::*;
use crate::sql::FeedFormat;

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeedRequest {
    #[validate(url)]
    link: String,
    format: FeedFormat,
}

pub async fn create_feed(
    State(state): State<ApiContext>,
    Json(body): Json<CreateFeedRequest>,
) -> Result<impl IntoResponse> {
    body.validate()?;

    let feed = Daemon::create_feed(&state.pool, &body.link).await?;

    Ok((http::StatusCode::CREATED, Json(feed)))
}
