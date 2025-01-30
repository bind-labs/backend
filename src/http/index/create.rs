use crate::http::common::*;
use crate::sql::{Icon, SortOrder, UserIndex};

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateIndexRequest {
    #[validate(custom(function = "crate::query::validate_query"))]
    query: String,
    sort: SortOrder,
    title: String,
    description: Option<String>,
    icon: Icon,
}

pub async fn create_index(
    State(state): State<ApiContext>,
    Json(body): Json<CreateIndexRequest>,
) -> Result<Json<UserIndex>> {
    todo!()
}
