#![allow(unused)]
use crate::http::auth::AuthUser;
use crate::http::common::*;
use crate::sql::UserIndex;
use ormx::Table;

pub async fn get_index(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
) -> Result<Json<UserIndex>> {
    let index = UserIndex::get(&state.pool, id).await?;
    if index.owner != user.id {
        return Err(Error::NotOwner);
    }

    Ok(Json(index))
}
