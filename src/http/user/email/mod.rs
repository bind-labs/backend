use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::http::common::ApiContext;

mod login;
mod register;
mod verify;

pub fn router() -> OpenApiRouter<ApiContext> {
    OpenApiRouter::new()
        .routes(routes!(login::login))
        .routes(routes!(register::register))
        .routes(routes!(verify::verify))
}
