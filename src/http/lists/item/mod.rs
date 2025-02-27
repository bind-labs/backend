use crate::http::common::ApiContext;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

mod create;
mod delete;
mod get;

pub fn router() -> OpenApiRouter<ApiContext> {
    OpenApiRouter::new()
        .routes(routes!(get::get_list_items, create::create_list_item))
        .routes(routes!(delete::delete_list_item))
}
