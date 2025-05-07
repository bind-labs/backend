use crate::http::common::*;
use crate::sql::Feed;

/// List all available feeds
///
/// Retrieves a list of all RSS/Atom feeds available in the system. This endpoint requires
/// authentication and returns feeds that the system is aggregating. Users can subscribe to
/// these feeds by adding them to their personal indexes.
///
/// Each feed contains metadata such as the title, description, URL, and update frequency.
/// The actual feed content is accessed through the index endpoints.
#[utoipa::path(
    get,
    path = "/",
    tag = "feed",
    responses(
        (status = 200, description = "List of all feeds", body = Vec<Feed>),
        (status = 401, description = "Unauthorized - Valid JWT token required"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
pub async fn list_feeds(_: AuthUser, State(state): State<ApiContext>) -> Result<Json<Vec<Feed>>> {
    Ok(Json(Feed::all(&state.pool).await?))
}
