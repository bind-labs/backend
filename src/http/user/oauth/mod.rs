use axum::{routing::get, Router};

use crate::http::common::ApiContext;

mod authorize;
mod callback;
mod providers;

pub fn router() -> Router<ApiContext> {
    Router::new()
        .route("/providers", get(providers::list_providers))
        .route("/authorize", get(authorize::authorize))
        .route("/callback", get(callback::callback))
}
