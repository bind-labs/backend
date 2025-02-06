use ormx::Insert;

use crate::http::auth::AuthUser;
use crate::http::common::*;
use crate::sql::{Icon, InsertUserIndex, SortOrder, UserIndex};

#[derive(Deserialize, Validate)]
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

    
    let sort: &str = body.sort.into();
    let query = InsertUserIndex {
        owner: user.id,
        query: body.query,
        sort: sort.to_string(),
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
    use crate::sql::Icon;

    use super::*;
    use pgtemp::PgTempDB;
    use sqlx::postgres::PgPoolOptions;
    #[tokio::test]
    #[ignore]
    async fn should_create_index_only_on_valid_query() {
        let db = PgTempDB::async_new().await;
        let pool = PgPoolOptions::new()
            .connect(&db.connection_uri())
            .await
            .unwrap();

        let _state = ApiContext::new(pool);

        let _create_index = CreateIndexRequest {
            query: "test hello world".to_string(),
            sort: SortOrder::AsIs,
            title: "Hello World".to_string(),
            description: None,
            icon: Icon::get_random_icon(),
        };
    }
}
