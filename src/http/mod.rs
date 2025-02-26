pub mod common;
pub mod error;
pub mod feed;
pub mod index;
pub mod items;
pub mod lists;
pub mod search;
pub mod user;

use axum::Router;
use common::ApiContext;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify,
};

#[derive(OpenApi)]
#[openapi(
        modifiers(&SecurityAddon),
        tags(
            (name = "feed", description = "Routes related to the feed")
        )
    )]
struct ApiDocs;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "Authorization Token",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            )
        }
    }
}

pub fn router() -> OpenApiRouter<ApiContext> {
    OpenApiRouter::with_openapi(ApiDocs::openapi()).nest("/feed", feed::router())
    // Router::new()
    //     .nest("/feed", feed::router())
    //     .nest("/index", index::router())
    //     .nest("/item", items::router())
    //     .nest("/list", lists::router())
    //     .nest("/search", search::router())
    //     .nest("/user", user::router())
}
