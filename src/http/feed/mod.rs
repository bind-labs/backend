use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::common::ApiContext;

pub mod create;
pub mod discover;
mod get;
mod list;

pub fn router() -> OpenApiRouter<ApiContext> {
    OpenApiRouter::new()
        .routes(routes!(create::create_feed, discover::discover_feeds))
        .routes(routes!(list::list_feeds))
        .routes(routes!(get::get_feed))
}
