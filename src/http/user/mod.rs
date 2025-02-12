use axum::Router;

use super::common::ApiContext;

mod history;

pub fn router() -> Router<ApiContext> {
  Router::new()
    .nest("/history", history::router())
}
