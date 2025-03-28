use crate::http::common::*;
use crate::sql::UserList;

/// Get a list by ID
#[utoipa::path(
    get,
    path = "/{id}",
    tag = "lists",
    params(
        ("id" = i32, Path, description = "List ID")
    ),
    responses(
        (status = 200, description = "List details", body = UserList),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the owner of the list"),
        (status = 404, description = "List not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
pub async fn get_list(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
) -> Result<Json<UserList>> {
    let list = UserList::get(&state.pool, id).await?;
    if list.owner != user.id {
        return Err(Error::NotOwner);
    }

    Ok(Json(list))
}
