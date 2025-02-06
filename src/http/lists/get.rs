#![allow(unused)]

use crate::http::auth::AuthUser;
use crate::http::common::*;
use crate::sql::UserList;
use ormx::Table;

pub async fn get_index(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
) -> Result<Json<UserList>> {
    let index = UserList::get(&state.pool, id).await?;
    if index.owner != user.id {
        return Err(Error::Forbidden);
    }

    Ok(Json(index))
}
