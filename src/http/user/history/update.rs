use crate::http::common::*;
use crate::sql::HistoryItem;

#[derive(Deserialize, Validate, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateHistoryItem {
    pub progress: f64,
}

/// Update a history item
#[utoipa::path(
    patch,
    path = "/{id}",
    tag = "user:history",
    params(
        ("id" = i32, Path, description = "History item ID")
    ),
    request_body = UpdateHistoryItem,
    responses(
        (status = 200, description = "History item updated successfully", body = HistoryItem),
        (status = 400, description = "Invalid history item parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the owner of the history item"),
        (status = 404, description = "History item not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("BearerToken" = [])
    )
)]
pub async fn update_history_item(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
    Json(body): Json<UpdateHistoryItem>,
) -> Result<Json<HistoryItem>> {
    body.validate()?;
    let mut item = HistoryItem::get(&state.pool, id).await?;
    if item.owner != user.id {
        return Err(Error::NotOwner);
    }

    item.progress = body.progress;
    item.updated_at = chrono::Utc::now();
    item.update(&state.pool).await?;

    Ok(Json(item))
}
