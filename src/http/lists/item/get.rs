use crate::http::common::*;
use crate::sql::UserListItem;

pub async fn get_list_item(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path((list_id, item_id)): Path<(i32, i32)>,
) -> Result<Json<UserListItem>> {
    let item = UserListItem::get(&state.pool, item_id).await?;

    if item.owner != user.id {
        return Err(Error::NotOwner);
    }

    if item.list != list_id {
        return Err(Error::BadRequest("Item does not belong to list".into()));
    }
    Ok(Json(item))
}

pub async fn get_list_items(
    _: AuthUser,
    State(state): State<ApiContext>,
    Query(pagination): Query<Pagination>,
    Path(list_id): Path<i32>,
) -> Result<Json<Vec<UserListItem>>> {
    let query = sqlx::query_as!(
        UserListItem,
        r#"
      SELECT * FROM user_list_item
      WHERE list = $1
      ORDER BY updated_at DESC
      LIMIT $2 OFFSET $3
      "#,
        list_id,
        pagination.limit,
        ((pagination.page - 1) * pagination.limit).into(),
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(query))
}
