use crate::http::common::*;
use crate::sql::UserIndex;

pub async fn list_indexes(
    State(state): State<ApiContext>,
) -> Result<Json<Vec<UserIndex>>> {
    let query: Vec<UserIndex> = sqlx::query_as("SELECT * FROM user_index WHERE owner = $1")
        .bind()
        .fetch_all(&state.pool)
        .await?;

    Ok(Json(query))
}
