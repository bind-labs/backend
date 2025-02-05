use rand::Rng;
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
use sqlx::{
    encode::IsNull, postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef}, prelude::FromRow, Decode, Encode, Type
};
use std::fmt::Write;
pub use user::*;
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Icon {
    pub icon: String,
    pub color: String,
}

impl Type<sqlx::Postgres> for Icon {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("icon")
    }
}

impl Encode<'_, sqlx::Postgres> for Icon {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let mut output = String::new();
        write!(&mut output, "(\"{}\",\"{}\")", self.icon, self.color).unwrap();

        buf.extend(output.as_bytes());
        Ok(IsNull::No)
    }
}

impl Decode<'_, sqlx::Postgres> for Icon {
    fn decode(
        value: PgValueRef<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = value.as_str()?;
        let trimmed = value.trim_matches(&['(', ')'][..]);
        let parts: Vec<&str> = trimmed.split(',').map(|s| s.trim_matches('"')).collect();

        if parts.len() != 2 {
            return Err("Invalid composite type format".into());
        }

        let icon = parts[0].to_string();
        let color = parts[1].to_string();

        Ok(Icon { icon, color })
    }
}

impl Icon {
    pub fn get_random_icon() -> Self {
        let mut rng = rand::rng();

        // Pick a random emoji from the Unicode range
        let emoji_code = rng.random_range(0x1F300..=0x1F64F);
        let icon = char::from_u32(emoji_code).unwrap_or('❓').to_string();

        // Generate a random hex color
        let color = format!("#{:06X}", rng.random_range(0x000000..=0xFFFFFF));

        Self { icon, color }
    }
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

impl From<String> for SortOrder {
    fn from(s: String) -> Self {
        SortOrder::from(s.as_str())
    }
}

impl Into<&'static str> for SortOrder {
    fn into(self) -> &'static str {
        match self {
            SortOrder::RecentlyUpdated => "recently_updated",
            SortOrder::AsIs => "as_is",
        }
    }
}
