use axum_extra::extract::Query;

use crate::http::auth::AuthUser;
use crate::http::{common::*, Pagination};
use crate::sql::{Icon, UserIndex};

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
