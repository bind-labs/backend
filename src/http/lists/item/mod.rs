use crate::http::common::ApiContext;
use axum::Router;
mod create;
mod delete;
mod get;

pub fn router() -> Router<ApiContext> {
    Router::new()
        .route(
            "/{list_id}/item",
            axum::routing::get(get::get_list_items).post(create::create_list_item),
        )
        .route(
            "/{list_id}/item/{item_id}",
            axum::routing::get(get::get_list_item)
                .delete(delete::delete_list_item),
        )
}
