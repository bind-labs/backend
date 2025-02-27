use crate::http::common::*;
use crate::scraper::extract::Extractor;
use crate::sql::{FeedItem, FeedItemParsed, InsertFeedItemParsed};


/// Get a feed item by ID
#[utoipa::path(
    get,
    path = "/{id}",
    tag = "items",
    params(
        ("id" = i64, Path, description = "Item ID")
    ),
    responses(
        (status = 200, description = "Feed item", body = FeedItem),
        (status = 404, description = "Item not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
pub async fn get_item(
    _user: AuthUser,
    State(state): State<ApiContext>,
    Path(id): Path<i64>,
) -> Result<Json<FeedItem>> {
    let index = FeedItem::get(&state.pool, id).await?;
    Ok(Json(index))
}

/// Get parsed content of a feed item
#[utoipa::path(
    get,
    path = "/{id}/parsed",
    tag = "items",
    params(
        ("id" = i64, Path, description = "Item ID")
    ),
    responses(
        (status = 200, description = "Parsed feed item content", body = FeedItemParsed),
        (status = 404, description = "Item not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("Authorization Token" = [])
    )
)]
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
    }
    .insert(&state.pool)
    .await?;

    Ok(Json(parsed))
}
