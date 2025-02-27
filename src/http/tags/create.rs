use crate::http::common::*;
use crate::sql::tags::UserTag;
use crate::sql::InsertUserTag;
use utoipa::ToSchema;

/// Request to create a new tag
#[derive(Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateTagRequest {
    /// Title of the tag
    #[validate(length(min = 1, max = 100))]
    pub title: String,
}

/// Create a new tag for the authenticated user
///
/// Creates a new tag with the provided title. Tags can be used to organize feeds and indexes.
/// Initially, a new tag has no items. Items can be added using the add item endpoint.
#[utoipa::path(
    put,
    path = "/",
    tag = "tags",
    request_body = CreateTagRequest,
    responses(
        (status = 201, description = "Tag created successfully", body = UserTag, 
         example = json!({
            "id": 123,
            "title": "Tech Blogs",
            "children": [],
            "created_at": "2023-01-01T12:00:00Z",
            "updated_at": "2023-01-01T12:00:00Z"
         })),
        (status = 400, description = "Invalid request - Title must be between 1 and 100 characters"),
        (status = 401, description = "Unauthorized - Valid JWT token required"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
pub async fn create_tag(
    user: AuthUser,
    State(state): State<ApiContext>,
    Json(body): Json<CreateTagRequest>,
) -> Result<impl IntoResponse> {
    body.validate()?;

    let tag = InsertUserTag { 
        owner: user.id, 
        title: body.title, 
        children: vec![],
        updated_at: chrono::Utc::now(),
    };

    let tag = tag.insert(&state.pool).await?;

    Ok((http::StatusCode::CREATED, Json(tag)))
}