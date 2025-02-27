use crate::http::common::*;
use crate::sql::{Icon, UserIndex};

/// List all indexes
#[utoipa::path(
    get,
    path = "/",
    tag = "index",
    params(
        Pagination
    ),
    responses(
        (status = 200, description = "List of all indexes", body = Vec<UserIndex>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
pub async fn list_indexes(
    _: AuthUser,
    State(state): State<ApiContext>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<UserIndex>>> {
    let values = sqlx::query_as!(
        UserIndex,
        r#"
        SELECT id, owner, query, sort, title, description, icon as "icon:Icon", created_at, updated_at FROM user_index
        ORDER BY updated_at DESC
        LIMIT $1 OFFSET $2
        "#,
        pagination.limit,
        (pagination.page - 1) * pagination.limit,
    ).fetch_all(&state.pool).await?;
    Ok(Json(values))
}
