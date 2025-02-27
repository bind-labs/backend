use crate::http::common::*;
use crate::sql::UserListItem;

/// Get a specific item from a list
#[utoipa::path(
    get,
    path = "/{list_id}/item/{item_id}",
    tag = "lists",
    params(
        ("list_id" = i32, Path, description = "List ID"),
        ("item_id" = i32, Path, description = "Item ID")
    ),
    responses(
        (status = 200, description = "List item details", body = UserListItem),
        (status = 400, description = "Item does not belong to the list"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the owner of the list item"),
        (status = 404, description = "List item not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
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

/// Get all items from a list
#[utoipa::path(
    get,
    path = "/{list_id}/item",
    tag = "lists",
    params(
        ("list_id" = i32, Path, description = "List ID"),
        Pagination
    ),
    responses(
        (status = 200, description = "List of items in the list", body = Vec<UserListItem>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "List not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
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
