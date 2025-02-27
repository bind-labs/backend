use crate::http::common::*;
use crate::sql::UserIndex;

/// Delete an index
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "index",
    params(
        ("id" = i32, Path, description = "Index ID")
    ),
    responses(
        (status = 204, description = "Index deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the owner of the index"),
        (status = 404, description = "Index not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
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
