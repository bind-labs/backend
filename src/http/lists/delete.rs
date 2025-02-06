#![allow(unused)]
use axum::response::IntoResponse;
use ormx::{Delete, Table};

use crate::http::auth::AuthUser;
use crate::http::common::*;
use crate::sql::UserList;

pub async fn delete_list(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse> {
    let index = UserList::get(&state.pool, id).await?;
    if index.owner != user.id {
        return Err(Error::Forbidden);
    }

    index.delete(&state.pool).await?;
    Ok(())
}
