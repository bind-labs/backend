use utoipa_axum::router::OpenApiRouter;

use crate::http::common::ApiContext;

mod email;
mod history;
mod oauth;

pub fn router() -> OpenApiRouter<ApiContext> {
    OpenApiRouter::new()
        .nest("/oauth", oauth::router())
        .nest("/history", history::router())
        .nest("/email", email::router())
}
