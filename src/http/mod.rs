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
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    20
}
