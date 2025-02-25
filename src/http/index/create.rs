use crate::http::common::*;
use crate::sql::{Icon, InsertUserIndex, SortOrder, UserIndex};
use bind_macros::IntoRequest;

#[derive(Deserialize, Serialize, Validate, IntoRequest)]
#[serde(rename_all = "camelCase")]
pub struct CreateIndexRequest {
    #[validate(custom(function = "crate::query::validate_query"))]
    query: String,
    sort: SortOrder,
    #[validate(length(min = 1, max = 1024))]
    title: String,
    description: Option<String>,
    icon: Icon,
}

pub async fn create_index(
    user: AuthUser,
    State(state): State<ApiContext>,
    Json(body): Json<CreateIndexRequest>,
) -> Result<Json<UserIndex>> {
    body.validate()?;

    let sort = body.sort.to_string();
    let query = InsertUserIndex {
        owner: user.id,
        query: body.query,
        sort,
        title: body.title,
        description: body.description,
        icon: body.icon,
    }
    .insert(&state.pool)
    .await?;

    Ok(Json(query))
}

#[cfg(test)]
mod test {
    use crate::{sql::Icon, tests::TestContext};

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn should_create_index_only_on_valid_query() {
        let ctx = TestContext::new().await;

        let create_index = CreateIndexRequest {
            query: "test hello world".to_string(),
            sort: SortOrder::AsIs,
            title: "Hello World".to_string(),
            description: None,
            icon: Icon::get_random_icon(),
        };

        let response = ctx
            .req(create_index.into_request(http::Method::PUT, "/index"))
            .await;
        assert_eq!(response.status(), http::StatusCode::OK);
    }
}
