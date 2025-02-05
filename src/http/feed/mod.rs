use axum::routing::{get, post};
use axum::Router;

use super::common::ApiContext;

mod create;
mod discover;
mod get;
mod list;

pub fn router() -> Router<ApiContext> {
    Router::new()
        .route("/feed", get(list::list_feeds).put(create::create_feed))
        .route("/feed/{id}", get(get::get_feed))
        .route("/feed/discover", post(discover::discover_feeds))
}
