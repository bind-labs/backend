use crate::sql::{FeedFormat, FeedStatus};
use sqlx::PgPool;

use super::FeedToUpdate;

pub async fn get_out_of_date_feeds(pool: &PgPool) -> Result<Vec<FeedToUpdate>, sqlx::Error> {
    let out_of_date_feeds = sqlx::query_as!(
        FeedToUpdate,
        r#"
        SELECT
            id,
            status AS "status: FeedStatus",
            format AS "format: FeedFormat",
            link,

            skip_hours,
            skip_days_of_week,
            ttl_in_minutes,
            etag,

            updated_at,
            fetched_at,
            successful_fetch_at,
            next_fetch_at
        FROM feed
        WHERE next_fetch_at < NOW() AND status = 'active'"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(out_of_date_feeds)
}
