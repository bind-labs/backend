use crate::http::common::*;
use crate::sql::UserTag;

/// Delete a list
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "tags",
    params(
        ("id" = i32, Path, description = "Delete Tag ID")
    ),
    responses(
        (status = 204, description = "Tag deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the owner of the tag"),
        (status = 404, description = "Tag not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
pub async fn delete_tag(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse> {
    let tag = UserTag::get(&state.pool, id).await?;
    if tag.owner != user.id {
        return Err(Error::NotOwner);
    }

    tag.delete(&state.pool).await?;
    Ok(())
}
