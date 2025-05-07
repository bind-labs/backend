use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, sqlx::Type, Deserialize, ToSchema)]
#[sqlx(type_name = "tag_child")]
pub struct TagChild {
    #[serde(rename = "type")]
    pub type_: TagChildType,
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq, sqlx::Type, Serialize, Deserialize, ToSchema)]
#[sqlx(type_name = "tag_child_type")]
pub enum TagChildType {
    Feed,
    Index,
}

// CREATE TYPE tag_child_type AS ENUM ('feed', 'index');
// CREATE TYPE tag_child AS (type tag_child_type, id integer);
#[derive(Debug, Clone, sqlx::FromRow, PartialEq, Serialize, Deserialize, ormx::Table, ToSchema)]
#[ormx(table = "user_tag", id = id, insertable, deletable)]
pub struct UserTag {
    #[ormx(default)]
    pub id: i32,
    pub owner: i32,
    pub title: String,
    #[ormx(custom_type, by_ref)]
    pub children: Vec<TagChild>,
    #[ormx(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
