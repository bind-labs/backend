use crate::http::common::ApiContext;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

mod delete;
mod get;
mod update;

pub fn router() -> OpenApiRouter<ApiContext> {
    OpenApiRouter::new()
        .routes(routes!(get::get_user_history))
        .routes(routes!(
            get::get_user_history_item,
            delete::delete_history_item,
            update::update_history_item
        ))
}
