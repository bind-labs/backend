use axum::Router;

use crate::http::common::ApiContext;

mod email;
mod history;
mod oauth;

pub fn router() -> Router<ApiContext> {
    Router::new()
        .nest("/oauth", oauth::router())
        .nest("/history", history::router())
        .nest("/email", email::router())
}
