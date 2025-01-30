use axum::Router;
use axum::routing::get;

use crate::AppState;

mod index;


pub fn router() -> Router<AppState> {
    Router::new().route("/index/:id", get(index::get_index))
}
