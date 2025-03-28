use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::common::ApiContext;

mod create;
mod delete;
mod get;
mod list;
mod update;

pub fn router() -> OpenApiRouter<ApiContext> {
    OpenApiRouter::new()
        .routes(routes!(list::list_indexes, create::create_index))
        .routes(routes!(
            get::get_index,
            delete::delete_index,
            update::update_index
        ))
}
