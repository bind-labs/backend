use axum_extra::extract::Query;
use ormx::Table;

use crate::http::auth::AuthUser;
use crate::http::{common::*, Pagination};
use crate::sql::UserIndex;

pub async fn list_indexes(
    user: AuthUser,
    State(state): State<ApiContext>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<UserIndex>>> {
    let offset = (pagination.page - 1) * pagination.limit;
    let limit = pagination.limit;
    let values = UserIndex::all_paginated(&state.pool, offset.into(), limit.into()).await?;
    Ok(Json(values))
}
