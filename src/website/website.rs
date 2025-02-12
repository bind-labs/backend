// TODO(@aaditya-sahay): This is a bit of a mess, but I think it's fine for now. I'll come back to it later.

use readability::{ExtractOptions, Readable};
use reqwest::Url;

#[derive(Debug, thiserror::Error)]
pub enum WebParserError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Encountered a captcha that we don't have a solver for")]
    CaptchaError,
    #[error(transparent)]
    ReadabilityError(#[from] readability::ReadabilityError),
}

#[cfg(feature = "flaresolverr")]
struct Flaresolverr {
    session_id: String,
    host_url: String,
}

#[cfg(feature = "flaresolverr")]
impl Flaresolverr {
    pub async fn new(client: &reqwest::Client, host: &str) -> Result<Self, WebParserError> {
        tracing::debug!("Creating a new flaresolverr session");
        let response = client
            .post(&format!("{}/v1", host))
            .json(&serde_json::json!({
                "cmd": "sessions.create"
            }))
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let session_id = json["session"].as_str().unwrap().to_string();
        tracing::debug!("Created a new flaresolverr session with id {}", session_id);

        Ok(Self {
            session_id: session_id,
            host_url: host.to_string(),
        })
    }

    pub async fn make_request(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<String, WebParserError> {
        let response = client
            .post(&format!("{}/v1", self.host_url))
            .json(&serde_json::json!({
                "cmd": "request.get",
                "url": url,
                "session": self.session_id,
            }))
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let body = json["body"].as_str().unwrap().to_string();

        Ok(body)
    }
}

#[cfg(feature = "flaresolverr")]
impl Drop for Flaresolverr {
    fn drop(&mut self) {
        let response = reqwest::Client::new()
            .post(&format!("{}/v1", self.host_url))
            .json(&serde_json::json!({
                "cmd": "sessions.destroy",
                "session": self.session_id
            }))
            .send();

        tokio::spawn(async move {
            let _ = response.await;
        });
    }
}

pub struct Extractor {
    pub client: reqwest::Client,
    #[cfg(feature = "flaresolverr")]
    solver: Flaresolverr,
}

impl Extractor {
    pub async fn new() -> Result<Self, WebParserError> {
        Ok(Self {
            client: reqwest::Client::new(),
            #[cfg(feature = "flaresolverr")]
            solver: Flaresolverr::new(&reqwest::Client::new(), "http://localhost:8191").await?,
        })
    }

    pub async fn extract(&self, url: &str) -> Result<Readable, WebParserError> {
        let response = self.client.get(url).send().await?;

        // check if the response is a captcha
        if response.status() == 403 {
            #[cfg(feature = "flaresolverr")]
            {
                tracing::info!("Encountered a captcha, using flaresolverr to solve it");
                let headers = response.headers().clone();
                let body = response.text().await?;
                if is_cloudflare_response(&headers) && is_cf_captcha(&body) {
                    let body = self.solver.make_request(&self.client, url).await?;
                    return Ok(body);
                }
            }

            return Err(WebParserError::CaptchaError);
        }

        let body = response.text().await?;
        let mut content_buffer = body.as_bytes();
        let url = Url::parse(url).unwrap();
        let content = readability::extract(&mut content_buffer, &url, ExtractOptions::default())?;

        Ok(content)
    }
}

pub fn is_cloudflare_response(headers: &reqwest::header::HeaderMap) -> bool {
    if let Some(server_header) = headers.get("server") {
        return server_header == "cloudflare";
    }

    false
}

pub fn is_cf_captcha(body: &str) -> bool {
    body.contains("cf-chl-bypass") || body.contains("captcha") || body.contains("challenge")
}

pub fn is_recaptcha(body: &str) -> bool {
    body.contains("recaptcha")
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_extract() {}
}
