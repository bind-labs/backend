use crate::http::common::*;
use crate::sql::Feed;

pub async fn list_feeds(_: AuthUser, State(state): State<ApiContext>) -> Result<Json<Vec<Feed>>> {
    Ok(Json(Feed::all(&state.pool).await?))
}
