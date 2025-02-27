use crate::http::common::*;
use crate::sql::{Icon, InsertUserList, UserList};

#[derive(Deserialize, Validate, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateListRequest {
    pub title: String,
    pub description: Option<String>,
    pub icon: Icon,
}

/// Create a new list
#[utoipa::path(
    put,
    path = "/",
    tag = "lists",
    request_body = CreateListRequest,
    responses(
        (status = 200, description = "List created successfully", body = UserList),
        (status = 400, description = "Invalid list parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
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
