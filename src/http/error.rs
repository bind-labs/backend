use axum::{
    http,
    response::{IntoResponse, Response},
};

use crate::feed::parser::ParsedFeedCreationError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
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
    ParseFeedCreationError(#[from] ParsedFeedCreationError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::ValidationError(_) => (http::StatusCode::BAD_REQUEST, "Validation error"),
            Error::DatabaseError(_) => (http::StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            Error::ReqwestError(_) => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to make request to third party",
            ),
            Error::RssFeedParseError(_) => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse feed",
            ),
            Error::AtomFeedParseError(_) => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse feed",
            ),
            Error::ParseFeedCreationError(_) => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create parsed feed from internal feed",
            ),
        }
        .into_response()
    }
}
