use ormx::Table;

use crate::http::auth::AuthUser;
use crate::http::common::*;
use crate::sql::{Icon, UserList};

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateListRequest {
    #[validate(length(min = 1, max = 1024))]
    title: Option<String>,
    description: Option<String>,
    icon: Option<Icon>,
}

pub async fn update_list(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
    Json(body): Json<UpdateListRequest>,
) -> Result<Json<UserList>> {
    body.validate()?;
    let mut index = UserList::get(&state.pool, id).await?;
    if index.owner != user.id {
        return Err(Error::Forbidden);
    }

    body.title.map(|title| index.title = title);
    body.description
        .map(|description| index.description = Some(description));
    body.icon.map(|icon| index.icon = Some(icon));

    index.updated_at = chrono::Utc::now();
    index.update(&state.pool).await?;

    Ok(Json(index))
}


