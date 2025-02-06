use super::auth::AuthUser;
use super::Pagination;
use crate::http::common::*;
use crate::query::Query as SearchQuery;
use crate::sql::{FeedItem, SortOrder};
use axum::routing::post;
use axum::Router;
use axum_extra::extract::Query;

#[derive(Deserialize, Validate)]
pub struct SearchRequest {
    #[validate(custom(function = "crate::query::validate_query"))]
    query: String,
    sort: SortOrder,
}

pub async fn search(
    _user: AuthUser,
    State(state): State<ApiContext>,
    Query(pagination): Query<Pagination>,
    Json(body): Json<SearchRequest>,
) -> Result<Json<Vec<FeedItem>>> {
    body.validate()?;
    let query = SearchQuery::try_from(body.query)
        .expect("This is not be possible because the query is validated");
    // TODO: Think about this behavior of querying
    let (sql, mut params) = query.to_sql();
    let order_by_column = match body.sort {
        SortOrder::AsIs => "index_in_feed",
        SortOrder::RecentlyUpdated => "updated_at",
    };
    let paginated_sql = format!("{} ORDER BY {} LIMIT ? OFFSET ?", sql, order_by_column,);

    params.push(format!("{}", pagination.limit));
    params.push(format!("{}", (pagination.page - 1) * pagination.limit));

    let mut query = sqlx::query_as::<_, FeedItem>(&paginated_sql);

    for param in params {
        query = query.bind(param);
    }

    let values = query.fetch_all(&state.pool).await?;
    Ok(Json(values))
}

pub fn router() -> Router<ApiContext> {
    Router::new().route("/search", post(search))
}
