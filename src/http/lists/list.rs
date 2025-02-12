use axum_extra::extract::Query;

use crate::http::auth::AuthUser;
use crate::http::{common::*, Pagination};
use crate::sql::Icon;
use crate::sql::UserList;


pub async fn list_lists(
    _user: AuthUser,
    State(state): State<ApiContext>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<UserList>>> {
    let offset = (pagination.page - 1) * pagination.limit;
    let limit = pagination.limit;
    let values = sqlx::query_as!(
        UserList,
        r#"
        SELECT id,owner, title, description, icon as "icon:Icon", created_at, updated_at FROM user_list
        ORDER BY updated_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset,
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(values))
}
