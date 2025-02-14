#![allow(unused)]
use axum::response::IntoResponse;

use crate::http::auth::AuthUser;
use crate::http::common::*;
use crate::sql::UserIndex;

use ormx::{Delete, Table};

pub async fn delete_index(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse> {
    let index = UserIndex::get(&state.pool, id).await?;
    if index.owner != user.id {
        return Err(Error::NotOwner);
    }

    index.delete(&state.pool).await?;
    Ok(())
}
