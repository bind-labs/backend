use ormx::{Insert, Table};

use crate::http::auth::AuthUser;
use crate::http::common::*;
use crate::sql::{FeedItem, InsertUserListItem, UserList, UserListItem};

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateListItemRequest {
    pub index: i32,
    pub owner: i32,
    /// The id of the feed item this item is referencing
    pub item: i64,
}

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
        return Err(Error::Forbidden);
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
