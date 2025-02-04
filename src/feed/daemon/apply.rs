use chrono::Utc;

use crate::{
    feed::parser::feed_item::ParsedFeedItem,
    sql::{Feed, FeedItem},
};

use super::update::FeedUpdate;

pub async fn apply_feed_update(
    pool: &sqlx::PgPool,
    feed: &Feed,
    feed_update: &FeedUpdate,
) -> Result<(), sqlx::Error> {
    let mut did_update_items = false;
    // TODO: fetch only the items in the update
    if let Some(items) = feed_update.items.as_ref() {
        did_update_items = apply_feed_items_update(pool, feed, feed_update, items).await?;
    }

    let updated_at = if did_update_items {
        Utc::now()
    } else {
        feed.updated_at
    };

    sqlx::query!(
        r#"
        UPDATE feed
        SET
          status = $2,
          format = $3,
          link = $4,
          domain = $5,
          title = $6,
          description = $7,
          icon = $8,
          skip_hours = $9,
          skip_days_of_week = $10,
          ttl_in_minutes = $11,
          etag = $12,
          updated_at = $13,
          fetched_at = $14,
          successful_fetch_at = $15,
          next_fetch_at = $16
        WHERE id = $1
        "#,
        feed.id,
        feed_update.status.as_ref().unwrap_or(&feed.status) as _,
        feed_update.format.as_ref().unwrap_or(&feed.format) as _,
        feed_update.link.as_deref().unwrap_or(&feed.link),
        feed_update.domain.as_deref().or(feed.domain.as_deref()),
        feed_update.title.as_deref().unwrap_or(&feed.title),
        feed_update
            .description
            .as_deref()
            .unwrap_or(&feed.description),
        feed_update.icon.as_deref().or(feed.icon.as_deref()),
        feed_update
            .skip_hours
            .as_deref()
            .unwrap_or(&feed.skip_hours),
        feed_update
            .skip_days_of_week
            .as_deref()
            .unwrap_or(&feed.skip_days_of_week),
        feed_update.ttl_in_minutes.or(feed.ttl_in_minutes),
        feed_update.etag.as_deref().or(feed.etag.as_deref()),
        updated_at,
        feed_update.fetched_at.unwrap_or(feed.fetched_at),
        feed_update
            .successful_fetch_at
            .unwrap_or(feed.successful_fetch_at),
        feed_update.next_fetch_at.unwrap_or(feed.next_fetch_at),
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn apply_feed_items_update(
    pool: &sqlx::PgPool,
    feed: &Feed,
    feed_update: &FeedUpdate,
    feed_items: &[ParsedFeedItem],
) -> Result<bool, sqlx::Error> {
    Ok(false)
}
// let existing_items = sqlx::query_as!(
//     FeedItem,
//     r#"
//         SELECT
//           id, feed_id, index_in_feed, guid, title, link, description, enclosure as "enclosure: _",
//           content, categories, comments_link, published_at, created_at, updated_at
//         FROM feed_item
//         WHERE feed_id = $1
//     "#,
//     feed.id
// )
// .fetch_all(pool)
// .await?;
//
// let new_items = feed_update
//     .items
//     .as_ref()
//     .unwrap_or(&Vec::new())
//     .iter()
//     .enumerate()
//     .map(|(index, item)| {
//         let existing_item = existing_items
//             .iter()
//             .find(|existing_item| existing_item.index_in_feed == index);
//
//         let existing_item = existing_item.unwrap_or_else(|| {
//             panic!(
//                 "Item {} not found in existing items for feed {}",
//                 index, feed.id
//             )
//         });
//
//         let new_item = FeedItem {
//             id: existing_item.id,
//             feed_id: existing_item.feed
