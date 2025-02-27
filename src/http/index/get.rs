#![allow(unused)]
use crate::http::common::*;
use crate::sql::UserIndex;

/// Get an index by ID
#[utoipa::path(
    get,
    path = "/{id}",
    tag = "index",
    params(
        ("id" = i32, Path, description = "Index ID")
    ),
    responses(
        (status = 200, description = "Index details", body = UserIndex),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the owner of the index"),
        (status = 404, description = "Index not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
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
