#![allow(unused)]
use crate::http::auth::AuthUser;
use crate::http::common::*;
use crate::sql::UserIndex;

pub async fn list_indexes(
    user: AuthUser,
    State(state): State<ApiContext>,
) -> Result<Json<Vec<UserIndex>>> {
    todo!()
}
