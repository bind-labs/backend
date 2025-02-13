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
use serde::Deserialize;

pub fn router() -> Router<ApiContext> {
    Router::new()
        .nest("/feed", feed::router())
        .nest("/index", index::router())
        // .nest("/item", items::router())
        .nest("/list", lists::router())
        .nest("/search", search::router())
        .nest("/user", user::router())
}

#[derive(Deserialize, Debug)]
pub struct Pagination {
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    20
}
