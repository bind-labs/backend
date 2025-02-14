use axum::{routing::post, Router};

use crate::http::common::ApiContext;

mod login;
mod register;
mod verify;

pub fn router() -> Router<ApiContext> {
    Router::new()
        .route("/login", post(login::login))
        .route("/register", post(register::register))
        .route("/verify", post(verify::verify))
}
