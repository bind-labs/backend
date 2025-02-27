use crate::http::common::*;
use crate::sql::HistoryItem;

/// Delete a history item
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "user:history",
    params(
        ("id" = i32, Path, description = "History item ID")
    ),
    responses(
        (status = 204, description = "History item deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "History item not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
pub async fn delete_history_item(
    _: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse> {
    let item = HistoryItem::get(&state.pool, id).await?;

    item.delete(&state.pool).await?;
    Ok(())
}
