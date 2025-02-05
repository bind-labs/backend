use crate::http::auth::AuthUser;
use crate::http::common::*;
use crate::sql::{Icon, SortOrder, UserIndex};

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
    let query = sqlx::query_as!(
        UserIndex,
        r#"
        INSERT INTO user_index (owner, query, sort, title, description, icon)
        VALUES ($1, $2,$3, $4, $5, $6)
        RETURNING 
            id,
            owner,
            query,
            sort,
            title,
            description,
            icon as "icon:Icon",
            created_at,
            updated_at
        "#,
        user.id,
        body.query,
        sort,
        body.title,
        body.description,
        body.icon as _
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(query))
}

#[cfg(test)]
mod test {
    use crate::sql::Icon;

    use super::*;
    use axum::{http::Request, routing::post, Router};
    use http_body_util::BodyExt;
    use pgtemp::PgTempDB;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;
    #[tokio::test]
    #[ignore]
    async fn should_create_index_only_on_valid_query() {
        let db = PgTempDB::async_new().await;
        let pool = PgPoolOptions::new()
            .connect(&db.connection_uri())
            .await
            .unwrap();

        let state = ApiContext::new(pool);

        let create_index = CreateIndexRequest {
            query: "test hello world".to_string(),
            sort: SortOrder::AsIs,
            title: "Hello World".to_string(),
            description: None,
            icon: Icon::get_random_icon()
        };

        
    }
}
