use axum::Router;

use super::common::ApiContext;

mod history;
mod oauth;

pub fn router() -> Router<ApiContext> {
    Router::new()
        .nest("/oauth", oauth::router())
        .nest("/history", history::router())
}
