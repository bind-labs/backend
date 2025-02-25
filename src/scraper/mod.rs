// TODO(@aaditya-sahay): This is a bit of a mess, but I think it's fine for now. I'll come back to it later.

pub mod extract;
pub mod flaresolverr;

#[derive(Debug, thiserror::Error)]
pub enum WebParserError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Encountered a captcha that we don't have a solver for")]
    CaptchaError,
    #[error(transparent)]
    ReadabilityError(#[from] readability::ReadabilityError),
}
