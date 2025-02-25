use lettre::{
    message::Mailbox,
    transport::smtp::{self, response::Response},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

#[derive(Debug, Clone)]
pub struct SmtpClient {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    pub from: Mailbox,
}

impl SmtpClient {
    pub async fn new(uri: &str, from: &str) -> Result<Self, SmtpError> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::from_url(uri)?.build();
        if !transport.test_connection().await? {
            return Err(SmtpError::ConnectionError);
        }

        let from: Mailbox = format!("noreply <{}>", from).parse().unwrap();

        Ok(Self { transport, from })
    }

    pub fn mock() -> Self {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::from_url("smtp://mock")
            .unwrap()
            .build();
        let from: Mailbox = "noreply <mock@localhost>".parse().unwrap();

        Self { transport, from }
    }

    pub async fn send(&self, email: &Message) -> Result<Response, smtp::Error> {
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
