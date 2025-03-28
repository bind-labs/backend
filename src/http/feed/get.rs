use crate::http::common::*;
use crate::sql::Feed;

/// Get a feed by ID
#[utoipa::path(
    get,
    path = "/{id}",
    tag = "feed",
    params(
        ("id" = i32, Path, description = "Feed ID")
    ),
    responses(
        (status = 200, description = "Feed found", body = Feed),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Feed not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    )
)]
pub async fn get_feed(
    _: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
) -> Result<Json<Feed>> {
    Ok(Json(Feed::get(&state.pool, id).await?))
}
