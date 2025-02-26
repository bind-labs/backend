use crate::feed::daemon::Daemon;
use crate::http::common::*;
use crate::sql::Feed;
use utoipa::ToSchema;

/// Request to create a new feed
#[derive(Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeedRequest {
    /// URL of the feed to create
    #[validate(url)]
    pub link: String,
}

/// Create a new feed
#[utoipa::path(
    put,
    path = "/",
    tag = "feed",
    request_body = CreateFeedRequest,
    responses(
        (status = 201, description = "Feed created successfully", body = Feed),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn create_feed(
    _: AuthUser,
    State(state): State<ApiContext>,
    Json(body): Json<CreateFeedRequest>,
) -> Result<impl IntoResponse> {
    body.validate()?;

    let feed = Daemon::create_feed(&state.pool, &body.link).await?;

    Ok((http::StatusCode::CREATED, Json(feed)))
}
