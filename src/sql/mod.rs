use serde::{Deserialize, Serialize};

mod feed;
mod history;
mod index;
mod list;
mod user;

pub use feed::*;
pub use history::*;
pub use index::*;
pub use list::*;
pub use user::*;

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "icon")]
pub struct Icon {
    pub icon: String,
    pub color: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "sort_order")]
pub enum SortOrder {
    RecentlyUpdated,
    AsIs,
}

impl From<&str> for SortOrder {
    fn from(s: &str) -> Self {
        match s {
            "recently_updated" => SortOrder::RecentlyUpdated,
            "as_is" => SortOrder::AsIs,
            _ => SortOrder::RecentlyUpdated,
        }
    }
}
