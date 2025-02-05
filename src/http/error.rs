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
    #[error("This operation is forbidden")]
    Forbidden,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::ValidationError(_) => (http::StatusCode::BAD_REQUEST, format!("{}", self)),
            Error::Forbidden => (http::StatusCode::FORBIDDEN, format!("{}", self)),
            Error::ReqwestError(_)
            | Error::DatabaseError(_)
            | Error::RssFeedParseError(_)
            | Error::AtomFeedParseError(_)
            | Error::ParseFeedCreationError(_) => {
                (http::StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self))
            }
        }
        .into_response()
    }
}
