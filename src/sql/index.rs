use super::Icon;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents an index created by a user.
/// An **Index** can be thought of as a custom search over a certain set of feeds.

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow, ormx::Table, ToSchema)]
#[ormx(table = "user_index", id = id, insertable, deletable)]
pub struct UserIndex {
    #[ormx(default, get_one)]
    pub id: i32,
    pub owner: i32,

    pub query: String,
    pub sort: String,

    pub title: String,
    pub description: Option<String>,
    #[ormx(custom_type, by_ref)]
    pub icon: Icon,

    #[ormx(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[ormx(default, set)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
