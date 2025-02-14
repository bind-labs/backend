use crate::http::common::ApiContext;
use axum::Router;

mod delete;
mod get;
mod update;

pub fn router() -> Router<ApiContext> {
    Router::new()
        .route("/", axum::routing::get(get::get_user_history))
        .route(
            "/{id}",
            axum::routing::get(get::get_user_history_item)
                .delete(delete::delete_history_item)
                .patch(update::update_history_item),
        )
}
