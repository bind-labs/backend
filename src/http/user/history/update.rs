use ormx::Table;

use crate::http::auth::AuthUser;
use crate::http::common::*;
use crate::sql::HistoryItem;

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateHistoryItem {
    pub progress: f64,
}

pub async fn update_history_item(
    user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i32>,
    Json(body): Json<UpdateHistoryItem>,
) -> Result<Json<HistoryItem>> {
    body.validate()?;
    let mut item = HistoryItem::get(&state.pool, id).await?;
    if item.owner != user.id {
        return Err(Error::Forbidden);
    }

    item.progress = body.progress;
    item.updated_at = chrono::Utc::now();
    item.update(&state.pool).await?;

    Ok(Json(item))
}
