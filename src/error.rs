use axum::{
    http,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Failed to parse rss feed")]
    RssFeedParseError(#[from] rss::Error),
    #[error("Failed to parse atom feed")]
    AtomFeedParseError(#[from] atom_syndication::Error),
    #[error("Failed to create parsed feed")]
    ParseFeedCreationError(#[from] crate::feed::ParsedFeedCreationError),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::ValidationError(_) => (http::StatusCode::BAD_REQUEST, "Validation error"),
            ServerError::DatabaseError(_) => {
                (http::StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
            ServerError::ReqwestError(_) => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to make request to third party",
            ),
            ServerError::RssFeedParseError(_) => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse feed",
            ),
            ServerError::AtomFeedParseError(_) => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse feed",
            ),
            ServerError::ParseFeedCreationError(_) => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create parsed feed from internal feed",
            ),
        }
        .into_response()
    }
}
