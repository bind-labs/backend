use ormx::Table;

use crate::http::common::*;
use crate::sql::Feed;

pub async fn get_feed(State(state): State<ApiContext>, Path(id): Path<i32>) -> Result<Json<Feed>> {
    Ok(Json(Feed::get(&state.pool, id).await?))
}
