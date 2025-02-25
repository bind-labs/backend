use super::WebParserError;
use readability::{ExtractOptions, Readable};
use reqwest::Url;

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

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_extract() {}
}
