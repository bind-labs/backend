use crate::http::auth::AuthUser;
use crate::http::common::*;
use crate::sql::{FeedItem, FeedItemParsed, InsertFeedItemParsed};
use crate::website::website::Extractor;
use ormx::{Insert, Table};

pub async fn get_item(
    _user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i64>,
) -> Result<Json<FeedItem>> {
    let index = FeedItem::get(&state.pool, id).await?;
    Ok(Json(index))
}

pub async fn get_parsed(
    _user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i64>,
) -> Result<Json<FeedItemParsed>> {
    let (feed_item, feed_item_parsed) = tokio::join!(
        FeedItem::get(&state.pool, id),
        FeedItemParsed::get_by_feed_item(&state.pool, &id)
    );
    let (feed_item, feed_item_parsed) = (feed_item?, feed_item_parsed?);
    if let Some(feed_item_parsed) = feed_item_parsed {
        if feed_item_parsed.updated_at > feed_item.updated_at {
            return Ok(Json(feed_item_parsed));
        }
    }
    let url = feed_item.guid;
    let extractor = Extractor::new().await?;

    let html = extractor.extract(&url).await?;
    let parsed = InsertFeedItemParsed { 
        feed_item_id: id,
        content: html.content,
        content_type: "text/html".to_string(),
    }.insert(&state.pool).await?;


    Ok(Json(parsed))
}
