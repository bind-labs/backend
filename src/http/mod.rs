pub mod auth;
pub mod common;
pub mod error;
pub mod feed;
pub mod index;
pub mod items;
pub mod lists;
pub mod search;
pub mod user;

use axum::Router;
use common::ApiContext;

pub fn router() -> Router<ApiContext> {
    Router::new()
        .nest("/feed", feed::router())
        .nest("/index", index::router())
        .nest("/item", items::router())
        .nest("/list", lists::router())
        .nest("/search", search::router())
        .nest("/user", user::router())
}
