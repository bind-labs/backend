use axum::response::IntoResponse;

use crate::http::auth::AuthUser;
use crate::http::common::*;
use crate::sql::UserIndex;

pub async fn delete_index(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    todo!();

    Ok(())
}
