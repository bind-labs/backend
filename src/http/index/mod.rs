use axum::routing::get;
use axum::Router;

use crate::ApiContext;

mod index;

pub fn router() -> Router<ApiContext> {
    Router::new().route("/index/:id", get(index::get_index))
}
