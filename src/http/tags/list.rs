use crate::http::common::*;
use crate::sql::tags::{TagChild, UserTag};

/// List all tags for the authenticated user
///
/// Retrieves a list of all tags created by the authenticated user.
/// Tags are used to organize feeds and indexes. Each tag contains its title and a list of items.
/// Results are paginated.
#[utoipa::path(
    get,
    path = "/",
    tag = "tags",
    params(
        Pagination
    ),
    responses(
        (status = 200, description = "List of user tags", body = Vec<UserTag>),
        (status = 401, description = "Unauthorized - Valid JWT token required"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
pub async fn list_tags(
    _: AuthUser,
    State(state): State<ApiContext>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<UserTag>>> {
    let offset = (pagination.page - 1) * pagination.limit;
    let limit = pagination.limit;
    let tags = sqlx::query_as!(
        UserTag,
        r#"
      SELECT id, title, owner, children as "children:Vec<TagChild>",  created_at, updated_at FROM user_tag
      ORDER BY updated_at DESC
      LIMIT $1 OFFSET $2
      "#,
        limit,
        offset,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(tags))
}
