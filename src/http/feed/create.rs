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

/// Create a new feed subscription
///
/// Add a new RSS/Atom feed to the system by providing its URL. The system will:
/// 1. Validate the URL format
/// 2. Fetch the feed content
/// 3. Parse the feed metadata (title, description, etc.)
/// 4. Store the feed in the database
/// 5. Begin regular updates of the feed content
///
/// The feed will be available for all users to add to their indexes.
/// If the feed already exists in the system, the existing feed will be returned.
///
/// The feed daemon handles automatic updates based on the feed's update frequency
/// or the presence of standard headers like Last-Modified and ETag.
#[utoipa::path(
    put,
    path = "/",
    tag = "feed",
    request_body = CreateFeedRequest,
    responses(
        (status = 201, description = "Feed created successfully", body = Feed, 
         example = json!({
            "id": 123,
            "title": "Example Tech Blog",
            "description": "A blog about technology and programming",
            "link": "https://example.com/feed.xml",
            "url": "https://example.com",
            "updated_at": "2023-01-01T12:00:00Z"
         })),
        (status = 400, description = "Invalid request - URL format is invalid or feed could not be fetched"),
        (status = 401, description = "Unauthorized - Valid JWT token required"),
        (status = 500, description = "Internal server error - Feed parsing failed")
    ),
    security(
        ("Authorization Token" = [])
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
