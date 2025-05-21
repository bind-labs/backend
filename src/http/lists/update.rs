use crate::http::common::*;
use crate::sql::{Icon, UserList};

#[derive(Deserialize, Validate, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateListRequest {
    #[validate(length(min = 1, max = 1024))]
    title: Option<String>,
    description: Option<String>,
    icon: Option<Icon>,
}

/// Update a list
#[utoipa::path(
    patch,
    path = "/index/{id}",
    tag = "lists",
    params(
        ("id" = i32, Path, description = "List ID")
    ),
    request_body = UpdateListRequest,
    responses(
        (status = 200, description = "List updated successfully", body = UserList),
        (status = 400, description = "Invalid list parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the owner of the list"),
        (status = 404, description = "List not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("BearerToken" = [])
    )
)]
pub async fn update_list(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
    Json(body): Json<UpdateListRequest>,
) -> Result<Json<UserList>> {
    body.validate()?;
    let mut index = UserList::get(&state.pool, id).await?;
    if index.owner != user.id {
        return Err(Error::NotOwner);
    }

    if let Some(title) = body.title {
        index.title = title;
    }
    if let Some(description) = body.description {
        index.description = Some(description);
    }
    if let Some(icon) = body.icon {
        index.icon = Some(icon);
    }

    index.updated_at = chrono::Utc::now();
    index.update(&state.pool).await?;

    Ok(Json(index))
}
