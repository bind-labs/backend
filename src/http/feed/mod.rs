use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::AppState;
mod feed;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/feed-information", post(feed::get_feed_information))
        .route("/feed", put(feed::create_feed))
}
