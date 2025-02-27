use std::collections::HashMap;

use axum::http;
use http_body_util::BodyExt;
use pgtemp::PgTempDB;
use reqwest::Url;
use serde::de::DeserializeOwned;
use tower::ServiceExt;
use tower_http::trace::TraceLayer;

use crate::{
    http::{
        common::{ApiContext, Origins},
        router,
    },
    smtp::SmtpClient,
};

use super::sql::TempDB;

pub struct TestContext {
    pub context: ApiContext,
    pub db: PgTempDB,
    pub pool: sqlx::PgPool,
}

impl TestContext {
    pub async fn new() -> Self {
        let db = TempDB::new().await;

        let context = ApiContext {
            pool: db.0.clone(),
            reqwest_client: reqwest::Client::new(),
            oauth_clients: HashMap::new(),
            smtp_client: SmtpClient::mock(),
            origins: Origins {
                web: Url::parse("http://localhost").unwrap(),
                android: Url::parse("http://localhost").unwrap(),
                ios: Url::parse("http://localhost").unwrap(),
            },
            jwt_secret: "secret".to_string(),
        };

        Self {
            context,
            pool: db.0,
            db: db.1,
        }
    }

    pub async fn req(
        &self,
        request: http::Request<impl Into<axum::body::Body>>,
    ) -> http::Response<axum::body::Body> {
        let (app, _) = router()
            .layer(TraceLayer::new_for_http())
            .with_state(self.context.clone())
            .split_for_parts();
        let request = request.map(Into::into);
        app.oneshot(request).await.unwrap()
    }

    pub async fn decode<T>(&self, response: http::Response<axum::body::Body>) -> T
    where
        T: DeserializeOwned,
    {
        let body = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice::<T>(&body).unwrap()
    }
}
