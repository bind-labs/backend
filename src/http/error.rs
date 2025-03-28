use axum::{
    http,
    response::{IntoResponse, Response},
};
use lettre::transport::smtp::Error as SmtpError;

use crate::{feed::daemon::FeedCreationError, scraper::WebParserError};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error("{0}")]
    Forbidden(String),

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    NotFound(String),

    #[error("You are not authorized to perform this action. Ensure you have provided a valid token in the Authorization header")]
    Unauthorized,

    #[error("You are not the owner of this resource")]
    NotOwner,

    #[error("Login failed")]
    LoginFailed,

    #[error(transparent)]
    CreateFeedError(#[from] FeedCreationError),

    #[error(transparent)]
    WebParserError(#[from] WebParserError),

    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),

    #[error(transparent)]
    SmtpError(#[from] SmtpError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::ValidationError(_) => (http::StatusCode::BAD_REQUEST, format!("{}", self)),
            Error::Forbidden(msg) | Error::BadRequest(msg) | Error::Conflict(msg) => {
                (http::StatusCode::FORBIDDEN, msg)
            }
            Error::NotFound(msg) => (http::StatusCode::NOT_FOUND, msg),
            Error::Unauthorized => (http::StatusCode::UNAUTHORIZED, format!("{}", self)),
            Error::NotOwner => (http::StatusCode::FORBIDDEN, format!("{}", self)),
            Error::LoginFailed => (http::StatusCode::UNAUTHORIZED, format!("{}", self)),

            Error::WebParserError(_) => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Could not parse provided page".to_string(),
            ),

            // TODO: log the error in dev mode
            Error::ReqwestError(_)
            | Error::DatabaseError(_)
            | Error::AnyhowError(_)
            | Error::SmtpError(_) => {
                tracing::error!("{:?}", self);
                (
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }

            Error::CreateFeedError(err) => match err {
                FeedCreationError::NotModified => {
                    (http::StatusCode::BAD_REQUEST, format!("{}", err))
                }
                FeedCreationError::RedirectLoop => {
                    (http::StatusCode::BAD_REQUEST, format!("{}", err))
                }
                FeedCreationError::ParsingError(_) => {
                    (http::StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err))
                }
                FeedCreationError::OtherFetchError(_) => {
                    (http::StatusCode::BAD_REQUEST, format!("{}", err,))
                }
                FeedCreationError::NotFound => (http::StatusCode::NOT_FOUND, format!("{}", err)),
                FeedCreationError::SqlxError(_) => (
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                ),
            },
        }
        .into_response()
    }
}
