use crate::http::common::*;
use crate::sql::Icon;
use crate::sql::UserList;

/// List all user lists
#[utoipa::path(
    get,
    path = "/",
    tag = "lists",
    responses(
        (status = 200, description = "List of all user lists", body = Vec<UserList>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
pub async fn list_lists(
    _user: AuthUser,
    State(state): State<ApiContext>,
) -> Result<Json<Vec<UserList>>> {
    let values = sqlx::query_as!(
        UserList,
        r#"
        SELECT id, owner, title, description, icon as "icon:Icon", created_at, updated_at FROM user_list
        ORDER BY updated_at DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(values))
}
