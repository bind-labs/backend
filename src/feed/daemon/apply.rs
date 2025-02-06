use chrono::Utc;
use ormx::{Insert, Table};

use crate::{
    feed::parser::feed_item::ParsedFeedItem,
    sql::{Feed, FeedItem, InsertFeedItem},
};

use super::update::FeedUpdate;

pub async fn apply_feed_update(
    db: &sqlx::PgPool,
    feed: &Feed,
    feed_update: &FeedUpdate,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    let mut did_update_items = false;
    if let Some(items) = feed_update.items.as_ref() {
        did_update_items = apply_feed_items_update(db, &mut *tx, feed.id, items).await?;
    }

    let updated_at = if did_update_items {
        Utc::now()
    } else {
        feed.updated_at
    };

    // TODO: rewrite with ormx
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
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn apply_feed_items_update(
    db: impl sqlx::Executor<'_, Database = ormx::Db>,
    tx: impl sqlx::Executor<'_, Database = ormx::Db>,
    feed_id: i32,
    items: &[ParsedFeedItem],
) -> Result<bool, sqlx::Error> {
    // TODO: fetch only the items in the update
    let existing_items = FeedItem::get_by_feed(db, &feed_id).await?;

    // TODO: does the order of the feed items need to be reversed for insertion?
    // TODO: do we need to sort on date?

    // Add or update feed items
    let mut did_update_items = false;
    for item in items.iter() {
        let existing_item = existing_items
            .iter()
            .find(|existing_item| existing_item.guid == item.guid);

        if let Some(existing_item) = existing_item {
            let mut existing_item = existing_item.clone();
            existing_item.merge_with_parsed(item);
            existing_item.update(tx).await?;
            // TODO: check if items were updated
        } else {
            // TODO: set index_in_feed
            InsertFeedItem::from_parsed(item, feed_id, 0)
                .insert(tx)
                .await?;
            did_update_items = true;
        }
    }

    // Prune oldest items (by id) to limit the number of items to 1000
    // TODO: use created_at before using id. should we use updated_at instead?
    sqlx::query!("DELETE FROM feed_item WHERE id IN (SELECT id FROM feed_item WHERE feed_id = $1 ORDER BY id DESC OFFSET 1000)", feed_id)
        .execute(tx)
        .await?;

    Ok(did_update_items)
}
