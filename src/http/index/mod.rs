use axum::routing::get;
use axum::Router;

use super::common::ApiContext;

mod create;
mod get;

pub fn router() -> Router<ApiContext> {
    Router::new().route("/index/:id", get(get::get_index).put(create::create_index))
}
