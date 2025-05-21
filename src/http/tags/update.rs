use crate::http::common::*;
use crate::sql::tags::UserTag;
use crate::sql::TagChild;
use utoipa::ToSchema;

/// Request to modify a tag
#[derive(Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModifyTagRequest {
    /// New title for the tag
    #[validate(length(min = 1, max = 100))]
    pub title: Option<String>,
    pub children_to_add: Option<Vec<TagChild>>,
    pub children_to_remove: Option<Vec<TagChild>>,
}

/// Modify a tag's properties
#[utoipa::path(
    patch,
    path = "/{id}",
    tag = "tags",
    params(
        ("id" = i32, Path, description = "Tag ID to modify")
    ),
    request_body = ModifyTagRequest,
    responses(
        (status = 200, description = "Tag modified successfully", body = UserTag),
        (status = 400, description = "Invalid request - Title must be between 1 and 100 characters"),
        (status = 401, description = "Unauthorized - Valid JWT token required"),
        (status = 403, description = "Forbidden - User does not own this tag"),
        (status = 404, description = "Not found - Tag does not exist"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("BearerToken" = [])
    )
)]
pub async fn modify_tag(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
    Json(body): Json<ModifyTagRequest>,
) -> Result<Json<UserTag>> {
    body.validate()?;

    // First check if tag exists and belongs to the user
    let mut tag = UserTag::get(&state.pool, id).await?;
    if tag.owner != user.id {
        return Err(Error::Forbidden("User does not own this tag".to_string()));
    }

    // Update the tag
    if let Some(title) = body.title {
        tag.title = title.clone();
    }

    if let Some(children_to_add) = body.children_to_add {
        tag.children.extend(children_to_add);
        tag.children.dedup();
    }

    if let Some(children_to_remove) = body.children_to_remove {
        // remove children
        tag.children
            .retain(|child| !children_to_remove.contains(child));
    }

    tag.update(&state.pool).await?;

    Ok(Json(tag))
}
