use axum_extra::extract::Query;
use ormx::Table;

use crate::http::auth::AuthUser;
use crate::http::{common::*, Pagination};
use crate::sql::UserList;

pub async fn list_lists(
    _user: AuthUser,
    State(state): State<ApiContext>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<UserList>>> {
    let offset = (pagination.page - 1) * pagination.limit;
    let limit = pagination.limit;
    let values = UserList::all_paginated(&state.pool, offset.into(), limit.into()).await?;
    Ok(Json(values))
}
