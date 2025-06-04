use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::http::common::ApiContext;

mod login;
mod register;
mod reset_password;
mod send_password_reset_code;
mod verify;

pub fn router() -> OpenApiRouter<ApiContext> {
    OpenApiRouter::new()
        .routes(routes!(login::login))
        .routes(routes!(register::register))
        .routes(routes!(verify::verify))
        .routes(routes!(send_password_reset_code::send_password_reset_code))
        .routes(routes!(reset_password::reset_password))
}
