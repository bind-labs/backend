use axum_extra::extract::Query;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::http::common::*;
use crate::query::Query as SearchQuery;
use crate::sql::{FeedItem, SortOrder};

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct SearchRequest {
    #[validate(custom(function = "crate::query::validate_query"))]
    query: String,
    sort: SortOrder,
}

/// Search for feed items
#[utoipa::path(
    post,
    path = "/",
    tag = "search",
    request_body = SearchRequest,
    params(
        Pagination
    ),
    responses(
        (status = 200, description = "List of feed items matching the search query", body = Vec<FeedItem>),
        (status = 400, description = "Invalid search query"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
pub async fn search(
    _user: AuthUser,
    State(state): State<ApiContext>,
    Query(pagination): Query<Pagination>,
    Json(body): Json<SearchRequest>,
) -> Result<Json<Vec<FeedItem>>> {
    body.validate()?;
    let query = SearchQuery::try_from(body.query)
        .expect("This should not be possible because the query is validated");
    // TODO: Think about this behavior of querying
    let (sql, mut params) = query.to_sql();
    let order_by_column = match body.sort {
        SortOrder::AsIs => "index_in_feed",
        SortOrder::RecentlyUpdated => "updated_at",
    };
    let paginated_sql = format!("{} ORDER BY {} LIMIT ? OFFSET ?", sql, order_by_column);

    params.push(pagination.limit.to_string());
    params.push(((pagination.page - 1) * pagination.limit).to_string());

    let mut query = sqlx::query_as::<_, FeedItem>(&paginated_sql);

    for param in params {
        query = query.bind(param);
    }

    let values = query.fetch_all(&state.pool).await?;
    Ok(Json(values))
}

pub fn router() -> OpenApiRouter<ApiContext> {
    OpenApiRouter::new().routes(routes!(search))
}
