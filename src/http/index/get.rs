use crate::http::common::*;
use crate::sql::UserIndex;

pub async fn get_index(
    State(state): State<ApiContext>,
    Path(id): Path<String>,
) -> Result<Json<UserIndex>> {
    let query: UserIndex = sqlx::query_as("SELECT * FROM user_index WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(query))
}
