use crate::http::common::*;
use crate::sql::{FeedItem, InsertUserListItem, UserList, UserListItem};

#[derive(Deserialize, Validate, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateListItemRequest {
    pub index: i32,
    pub owner: i32,
    /// The id of the feed item this item is referencing
    pub item: i64,
}

/// Add an item to a list
#[utoipa::path(
    post,
    path = "/{list_id}/item",
    tag = "lists",
    params(
        ("list_id" = i32, Path, description = "List ID")
    ),
    request_body = CreateListItemRequest,
    responses(
        (status = 200, description = "Item added to list successfully", body = UserListItem),
        (status = 400, description = "Invalid list item parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the owner of the list"),
        (status = 404, description = "List or item not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("BearerToken" = [])
    )
)]
pub async fn create_list_item(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(list_id): Path<i32>,
    Json(body): Json<CreateListItemRequest>,
) -> Result<Json<UserListItem>> {
    body.validate()?;

    let (list, _item) = tokio::try_join!(
        UserList::get(&state.pool, list_id),
        FeedItem::get(&state.pool, body.item)
    )?;

    if list.owner != user.id {
        return Err(Error::NotOwner);
    }

    let query = InsertUserListItem {
        owner: body.owner,
        list: list_id,
        item: body.item,
        index: body.index,
    }
    .insert(&state.pool)
    .await?;
    Ok(Json(query))
}
