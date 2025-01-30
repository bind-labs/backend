use crate::http::common::*;
use crate::sql::Feed;

pub async fn get_feed(
    State(state): State<ApiContext>,
    Path(id): Path<String>,
) -> Result<Json<Feed>> {
    todo!()
}
