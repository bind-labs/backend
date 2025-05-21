use crate::http::common::*;
use crate::sql::{Icon, SortOrder, UserIndex};

#[derive(Deserialize, Validate, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIndexRequest {
    #[validate(custom(function = "crate::query::validate_query"))]
    query: Option<String>,
    sort: Option<SortOrder>,
    #[validate(length(min = 1, max = 1024))]
    title: Option<String>,
    description: Option<String>,
    icon: Option<Icon>,
}

/// Update an index
#[utoipa::path(
    patch,
    path = "/{id}",
    tag = "index",
    params(
        ("id" = i32, Path, description = "Index ID")
    ),
    request_body = UpdateIndexRequest,
    responses(
        (status = 200, description = "Index updated successfully", body = UserIndex),
        (status = 400, description = "Invalid index parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the owner of the index"),
        (status = 404, description = "Index not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("BearerToken" = [])
    )
)]
pub async fn update_index(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
    Json(body): Json<UpdateIndexRequest>,
) -> Result<Json<UserIndex>> {
    body.validate()?;
    let mut index = UserIndex::get(&state.pool, id).await?;
    if index.owner != user.id {
        return Err(Error::NotOwner);
    }

    if let Some(query) = body.query {
        index.query = query;
    }
    if let Some(sort) = body.sort {
        index.sort = sort.to_string();
    }
    if let Some(title) = body.title {
        index.title = title;
    }
    if let Some(description) = body.description {
        index.description = Some(description);
    }
    if let Some(icon) = body.icon {
        index.icon = icon;
    }

    index.updated_at = chrono::Utc::now();
    index.update(&state.pool).await?;

    Ok(Json(index))
}
