use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::http::common::ApiContext;

mod authorize;
mod callback;
mod providers;

pub fn router() -> OpenApiRouter<ApiContext> {
    OpenApiRouter::new()
        .routes(routes!(providers::list_providers))
        .routes(routes!(authorize::authorize))
        .routes(routes!(callback::callback))
}
