use crate::http::common::*;
use crate::sql::HistoryItem;

pub async fn delete_history_item(
    _: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse> {
    let item = HistoryItem::get(&state.pool, id).await?;

    item.delete(&state.pool).await?;
    Ok(())
}
