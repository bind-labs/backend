use crate::http::common::*;
use crate::sql::UserList;

/// Delete a list
#[utoipa::path(
    delete,
    path = "/index/{id}",
    tag = "lists",
    params(
        ("id" = i32, Path, description = "List ID")
    ),
    responses(
        (status = 204, description = "List deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the owner of the list"),
        (status = 404, description = "List not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("BearerToken" = [])
    )
)]
pub async fn delete_list(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse> {
    let index = UserList::get(&state.pool, id).await?;
    if index.owner != user.id {
        return Err(Error::NotOwner);
    }

    index.delete(&state.pool).await?;
    Ok(())
}
