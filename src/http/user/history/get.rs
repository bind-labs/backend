use axum::{extract::{Path, Query, State}, Json};
use ormx::Table;

use crate::{
    http::{auth::AuthUser, common::ApiContext, Pagination},
    sql::HistoryItem,
    http::common::*,
};


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


pub async fn get_user_history_item(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
) -> Result<Json<HistoryItem>> {
    let item = HistoryItem::get(&state.pool, id).await?;
    if item.owner != user.id {
        return Err(Error::Forbidden);
    }
    Ok(Json(item))
}
