use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::common::ApiContext;

pub mod create;
pub mod delete;
pub mod list;
pub mod update;
pub fn router() -> OpenApiRouter<ApiContext> {
    OpenApiRouter::new()
        .routes(routes!(list::list_tags, create::create_tag))
        .routes(routes!(update::modify_tag))
        .routes(routes!(delete::delete_tag))
}
