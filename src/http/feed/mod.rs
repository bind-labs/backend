use axum::routing::{get, post};
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

    // Router::new()
    //     .route("/", get(list::list_feeds).put(create::create_feed))
    //     .route("/{id}", get(get::get_feed))
    //     .route("/discover", post(discover::discover_feeds))
}
