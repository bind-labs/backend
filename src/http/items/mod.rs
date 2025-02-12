use axum::routing::get;
use axum::Router;

use super::common::ApiContext;

mod get;

pub fn router() -> Router<ApiContext> {
    Router::new()
        .route("/{id}", get(get::get_item))
        .route("/{id}/parsed", get(get::get_parsed))
}
