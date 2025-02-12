use axum::response::IntoResponse;
use ormx::{Delete, Table};

use crate::http::auth::AuthUser;
use crate::http::common::*;
use crate::sql::{UserList, UserListItem};

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
        return Err(Error::Forbidden);
    }

    if item.list != list_id {
        return Err(Error::BadRequest("Item does not belong to list".into()));
    }

    item.delete(&state.pool).await?;
    Ok(())
}
