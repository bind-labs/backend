use crate::http::auth::AuthUser;
use crate::http::common::*;
use crate::sql::UserIndex;

pub async fn get_index(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<String>,
) -> Result<Json<UserIndex>> {
    todo!();
}
