use axum::{
    extract::{Path, State},
    http, Json,
};

use serde::Deserialize;
use validator::Validate;

use crate::{
    error::ServerError,
    schema::{Icon, SortOrder, UserIndex},
    ApiContext,
};

pub async fn get_index(
    State(state): State<ApiContext>,
    Path(id): Path<String>,
) -> Result<Json<UserIndex>, ServerError> {
    let query: UserIndex = sqlx::query_as("SELECT * FROM user_index WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(query))
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateIndexRequest {
    #[validate(custom(function = "crate::query::validate_query"))]
    query: String,
    sort: SortOrder,
    title: String,
    description: Option<String>,
    icon: Icon,
}

pub async fn create_index(
    State(state): State<ApiContext>,
    Json(body): Json<CreateIndexRequest>,
) -> Result<Json<UserIndex>, ServerError> {
    todo!()
}
