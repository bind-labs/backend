use crate::http::common::*;
use crate::sql::{UserList, UserListItem};

/// Remove an item from a list
#[utoipa::path(
    delete,
    path = "/{list_id}/item/{item_id}",
    tag = "lists",
    params(
        ("list_id" = i32, Path, description = "List ID"),
        ("item_id" = i32, Path, description = "Item ID")
    ),
    responses(
        (status = 204, description = "Item removed from list successfully"),
        (status = 400, description = "Item does not belong to the list"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the owner of the list"),
        (status = 404, description = "List or item not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
pub async fn delete_list_item(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path((list_id, item_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse> {
    let (list, item) = tokio::try_join!(
        UserList::get(&state.pool, list_id),
        UserListItem::get(&state.pool, item_id)
    )?;
    if list.owner != user.id {
        return Err(Error::NotOwner);
    }

    if item.list != list_id {
        return Err(Error::BadRequest("Item does not belong to list".into()));
    }

    item.delete(&state.pool).await?;
    Ok(())
}
