use crate::http::common::*;
use crate::sql::{Icon, InsertUserList, UserList};

const MAX_LIST_COUNT: i64 = 500;

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
        (status = 403, description = "Reached max list count of 500"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("BearerToken" = [])
    )
)]
pub async fn create_list(
    user: AuthUser,
    State(state): State<ApiContext>,
    Json(body): Json<CreateListRequest>,
) -> Result<Json<UserList>> {
    body.validate()?;

    let list_count = UserList::count_by_owner(&state.pool, user.id).await?;
    if list_count >= MAX_LIST_COUNT {
        return Err(Error::Forbidden(
            "Reached max list count of 500".to_string(),
        ));
    }

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
