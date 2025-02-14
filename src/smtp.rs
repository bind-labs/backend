use lettre::{
    message::Mailbox,
    transport::smtp::{response::Response, Error},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct SmtpClient {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    pub from: Mailbox,
}

impl SmtpClient {
    pub async fn new(config: &Config) -> Result<Self, SmtpError> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::from_url(&config.smtp_uri)?.build();
        if !transport.test_connection().await? {
            return Err(SmtpError::ConnectionError);
        }

        let from: Mailbox = format!("noreply <{}>", config.smtp_from).parse().unwrap();

        Ok(Self { transport, from })
    }

    pub async fn send(&self, email: &Message) -> Result<Response, Error> {
        self.transport.send(email.clone()).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SmtpError {
    #[error("Failed to connect to SMTP server")]
    ConnectionError,

    #[error(transparent)]
    LettreError(#[from] lettre::transport::smtp::Error),
}
