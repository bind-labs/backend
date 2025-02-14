use crate::http::common::*;
use crate::sql::{Icon, InsertUserList, UserList};

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateListRequest {
    pub title: String,
    pub description: Option<String>,
    pub icon: Icon,
}

pub async fn create_list(
    user: AuthUser,
    State(state): State<ApiContext>,
    Json(body): Json<CreateListRequest>,
) -> Result<Json<UserList>> {
    body.validate()?;

    let query = InsertUserList {
        owner: user.id,
        title: body.title,
        description: body.description,
        icon: Some(body.icon),
    }
    .insert(&state.pool)
    .await?;
    Ok(Json(query))
}
