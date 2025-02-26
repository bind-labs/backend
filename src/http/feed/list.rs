use crate::http::common::*;
use crate::sql::Feed;

/// List all feeds
#[utoipa::path(
    get,
    path = "/",
    tag = "feed",
    responses(
        (status = 200, description = "List of all feeds", body = Vec<Feed>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn list_feeds(_: AuthUser, State(state): State<ApiContext>) -> Result<Json<Vec<Feed>>> {
    Ok(Json(Feed::all(&state.pool).await?))
}
