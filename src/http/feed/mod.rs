use axum::routing::{get, post};
use axum::Router;

use super::common::ApiContext;

mod create;
mod discover;
mod get;
mod list;

pub fn router() -> Router<ApiContext> {
    Router::new()
        .route("/", get(list::list_feeds).put(create::create_feed))
        .route("/{id}", get(get::get_feed))
        .route("/discover", post(discover::discover_feeds))
}
