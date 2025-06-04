use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::http::common::ApiContext;

mod login;
mod send_password_reset_code;
mod register;
mod verify;
mod reset_password;

pub fn router() -> OpenApiRouter<ApiContext> {
    OpenApiRouter::new()
        .routes(routes!(login::login))
        .routes(routes!(register::register))
        .routes(routes!(verify::verify))
        .routes(routes!(send_password_reset_code::send_password_reset_code))
        .routes(routes!(reset_password::reset_password))

}
