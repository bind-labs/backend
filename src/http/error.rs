use axum::{
    http,
    response::{IntoResponse, Response},
};

use crate::feed::daemon::FeedCreationError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error("This operation is forbidden")]
    Forbidden,

    #[error(transparent)]
    CreateFeedError(#[from] FeedCreationError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::ValidationError(_) => (http::StatusCode::BAD_REQUEST, format!("{}", self)),
            Error::Forbidden => (http::StatusCode::FORBIDDEN, format!("{}", self)),
            Error::ReqwestError(_) | Error::DatabaseError(_) => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),

            Error::CreateFeedError(err) => match err {
                FeedCreationError::NotModified => (http::StatusCode::BAD_REQUEST, format!("{}", err))
                FeedCreationError::RedirectLoop => (http::StatusCode::BAD_REQUEST, format!("{}", err))
                FeedCreationError::ParsingError(_) => (http::StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err))
                FeedCreationError::OtherFetchError(_) => (http::StatusCode::BAD_REQUEST, format!("{}", err,))
                FeedCreationError::NotFound => (http::StatusCode::NOT_FOUND, format!("{}", err)),
                FeedCreationError::SqlxError(_) => (http::StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            },
        }
        .into_response()
    }
}
