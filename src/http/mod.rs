pub mod auth;
pub mod common;
pub mod error;
pub mod feed;
pub mod index;
pub mod items;
pub mod lists;
pub mod search;
pub mod user;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Pagination {
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    20
}
