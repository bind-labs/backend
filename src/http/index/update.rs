use crate::http::common::*;
use crate::sql::{Icon, SortOrder, UserIndex};

#[derive(Deserialize, Validate)]
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

    body.query.map(|query| index.query = query);
    body.sort
        .map(|sort| index.sort = Into::<&'static str>::into(sort).to_string());
    body.title.map(|title| index.title = title);
    body.description
        .map(|description| index.description = Some(description));
    body.icon.map(|icon| index.icon = icon);

    index.updated_at = chrono::Utc::now();
    index.update(&state.pool).await?;

    Ok(Json(index))
}
