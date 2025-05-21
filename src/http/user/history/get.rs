use crate::http::common::*;
use crate::sql::HistoryItem;

/// Get user's reading history
#[utoipa::path(
    get,
    path = "/",
    tag = "user:history",
    params(
        Pagination
    ),
    responses(
        (status = 200, description = "User's reading history", body = Vec<HistoryItem>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("BearerToken" = [])
    )
)]
pub async fn get_user_history(
    user: AuthUser,
    State(state): State<ApiContext>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<HistoryItem>>> {
    let query = sqlx::query_as!(
        HistoryItem,
        r#"
        SELECT * FROM user_history_item
        WHERE owner = $1
        ORDER BY updated_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user.id,
        pagination.limit,
        ((pagination.page - 1) * pagination.limit).into(),
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(query))
}

/// Get a specific history item
#[utoipa::path(
    get,
    path = "/{id}",
    tag = "user:history",
    params(
        ("id" = i32, Path, description = "History item ID")
    ),
    responses(
        (status = 200, description = "History item details", body = HistoryItem),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the owner of the history item"),
        (status = 404, description = "History item not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("BearerToken" = [])
    )
)]
pub async fn get_user_history_item(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
) -> Result<Json<HistoryItem>> {
    let item = HistoryItem::get(&state.pool, id).await?;
    if item.owner != user.id {
        return Err(Error::NotOwner);
    }
    Ok(Json(item))
}
