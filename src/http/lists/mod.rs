use axum::routing::get;
use axum::Router;

use super::common::ApiContext;

mod create;
mod delete;
mod get;
mod list;

pub fn router() -> Router<ApiContext> {
    Router::new()
        .route("/", get(list::list_indexes).put(create::create_list))
        .route(
            "/index/{id}",
            get(get::get_index).delete(delete::delete_index),
        )
}

