#[cfg(feature = "flaresolverr")]
use super::WebParserError;

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

#[cfg(feature = "flaresolverr")]
pub struct Flaresolverr {
    session_id: String,
    host_url: String,
}

#[cfg(feature = "flaresolverr")]
impl Flaresolverr {
    pub async fn new(client: &reqwest::Client, host: &str) -> Result<Self, WebParserError> {
        tracing::debug!("Creating a new flaresolverr session");
        let response = client
            .post(format!("{}/v1", host))
            .json(&serde_json::json!({
                "cmd": "sessions.create"
            }))
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let session_id = json["session"].as_str().unwrap().to_string();
        tracing::debug!("Created a new flaresolverr session with id {}", session_id);

        Ok(Self {
            session_id,
            host_url: host.to_string(),
        })
    }

    pub async fn make_request(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<String, WebParserError> {
        let response = client
            .post(format!("{}/v1", self.host_url))
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
            .post(format!("{}/v1", self.host_url))
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
