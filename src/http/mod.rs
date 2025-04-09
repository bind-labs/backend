pub mod common;
pub mod error;
pub mod feed;
pub mod index;
pub mod items;
pub mod lists;
pub mod search;
pub mod tags;
pub mod user;

use common::ApiContext;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::auth::user::AuthUser;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify,
};

/// API Documentation for the Bind Feed Aggregator Backend.
///
/// Bind is a modern feed aggregator that allows users to subscribe to RSS/Atom feeds,
/// organize content into personalized indexes, create reading lists, and discover new content.
///
/// # API Design Principles
///
/// This API follows REST principles with resource-based endpoints organized by domain:
///
/// - **Prefixed Base Path**: All endpoints are prefixed with `/api/v1` for versioning
/// - **Resource Organization**: Endpoints are organized by domain (feeds, indexes, items, etc.)
/// - **Consistent Response Format**: Responses follow a consistent structure with appropriate status codes
/// - **Authentication**: JWT-based authentication with Bearer token scheme
/// - **Pagination**: List endpoints support pagination parameters
///
/// # Authentication
///
/// The API uses JWT (JSON Web Token) for authentication. Most endpoints require an authenticated user.
///
/// To authenticate:
/// 1. Obtain a token via login (`POST /user/email/login`) or OAuth (`GET /user/oauth/callback`)
/// 2. Include the token in requests with the `Authorization: Bearer <token>` header
///
/// Tokens are valid for 7 days by default. If a token expires, use the refresh endpoint to obtain a new one.
///
/// # Rate Limiting
///
/// API requests are subject to rate limiting to ensure service stability.
/// When rate limited, the API will return 429 Too Many Requests with a Retry-After header.
#[derive(OpenApi)]
#[openapi(
        modifiers(&SecurityAddon),
        paths(
            // You can list specific paths here to include in docs
        ),
        components(
            schemas(
                // Common response types
                AuthUser
            )
        ),
        tags(
            (name = "feed", description = "Routes related to feed subscriptions and discovery"),
            (name = "index", description = "Routes related to feed indexes and aggregation"),
            (name = "items", description = "Routes related to feed items and content"),
            (name = "lists", description = "Routes related to user-created lists"),
            (name = "search", description = "Routes related to search functionality"),
            (name = "user", description = "Routes related to user management"),
            (name = "user:email", description = "Routes related to email authentication"),
            (name = "user:oauth", description = "Routes related to OAuth authentication"),
            (name = "user:history", description = "Routes related to user reading history"),
            (name = "tags", description = "Routes related to user tags management")
        ),
        info(
            title = "Bind Feed Aggregator API",
            version = "1.0",
            description = "API for Bind, a modern feed aggregator service",
            contact(
                name = "Bind Team",
                email = "support@bind.sh"
            )
        )
    )]
pub struct ApiDocs;

/// Security addon that adds JWT Bearer authentication to the OpenAPI documentation.
///
/// This addon configures the OpenAPI documentation to advertise the JWT Bearer
/// authentication scheme used by the API. It specifies that the Authorization
/// header should contain a Bearer token.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            // Add JWT Bearer authentication scheme
            components.add_security_scheme(
                "Authorization Token",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            );

            // Add a description of how authentication works
            openapi.info.description = Some(format!(
                "## Authentication\n\nThis API uses JWT Bearer tokens for authentication. \
                To authenticate, include an `Authorization: Bearer <token>` header in your requests. \
                Tokens can be obtained from the login or OAuth endpoints and are valid for 7 days.\n\n\
                Protected endpoints will return 401 Unauthorized if a valid token is not provided.",
            ));
        }
    }
}

pub fn router() -> OpenApiRouter<ApiContext> {
    OpenApiRouter::with_openapi(ApiDocs::openapi())
        .nest("/feed", feed::router())
        .nest("/index", index::router())
        .nest("/item", items::router())
        .nest("/list", lists::router())
        .nest("/search", search::router())
        .nest("/tag", tags::router())
        .nest("/user", user::router())
}
