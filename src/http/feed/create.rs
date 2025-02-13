use axum::http;
use axum::response::IntoResponse;

use crate::feed::daemon::Daemon;
use crate::http::auth::AuthUser;
use crate::http::common::*;

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeedRequest {
    #[validate(url)]
    link: String,
}

pub async fn create_feed(
    _: AuthUser,
    State(state): State<ApiContext>,
    Json(body): Json<CreateFeedRequest>,
) -> Result<impl IntoResponse> {
    body.validate()?;

    let feed = Daemon::create_feed(&state.pool, &body.link).await?;

    Ok((http::StatusCode::CREATED, Json(feed)))
}
