use axum::response::IntoResponse;

use crate::http::common::*;
use crate::sql::UserIndex;

pub async fn delete_index(
    State(state): State<ApiContext>,
    Path(id): Path<String>,

) -> Result<impl IntoResponse> {
    let index = sqlx::query_as::<_, UserIndex>("SELECT * FROM user_index WHERE id = $1")
        .bind(&id)
        .fetch_one(&state.pool)
        .await?;

    // if index.owner != state.user.id {
    //     (axum::http::StatusCode::FORBIDDEN, "Index does not belong to the user")
    // }
    // only delete if the owner of the index is the current user
    let query: UserIndex = sqlx::query_as("DELETE FROM user_index WHERE id = $1 RETURNING *")
        .bind(id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(query))
}



