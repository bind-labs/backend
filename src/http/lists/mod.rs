use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::common::ApiContext;

mod create;
mod delete;
mod get;
mod item;
mod list;
mod update;

pub fn router() -> OpenApiRouter<ApiContext> {
    OpenApiRouter::new()
        .routes(routes!(list::list_lists, create::create_list))
        .routes(routes!(
            get::get_list,
            delete::delete_list,
            update::update_list
        ))
        .nest("/{list_id}", item::router())
}
