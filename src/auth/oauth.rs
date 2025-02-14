use anyhow::Context;
use jsonwebtoken::DecodingKey;
use oauth2::{
    basic::{BasicClient, BasicErrorResponseType, BasicTokenType},
    AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    EndpointNotSet, EndpointSet, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl,
    RevocationErrorResponseType, Scope, StandardErrorResponse, StandardRevocableToken,
    StandardTokenIntrospectionResponse, StandardTokenResponse, TokenResponse, TokenUrl,
};
use ormx::Insert;
use reqwest::Url;
use sqlx::PgPool;

use super::jwt::ExternalJwtToken;

use crate::{
    config,
    constants::USER_AGENT,
    sql::{InsertUserOAuthState, OAuthRedirectClient, UserOAuthState},
};

#[derive(Debug, Clone)]
pub struct OAuth2Client {
    config: OAuth2ClientConfig,
    http: reqwest::Client,
    // TODO: ideally we dont need to define all this
    client: Client<
        StandardErrorResponse<BasicErrorResponseType>,
        StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
        StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
        StandardRevocableToken,
        StandardErrorResponse<RevocationErrorResponseType>,
        EndpointSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointSet,
    >,
}

impl OAuth2Client {
    pub fn new(config: OAuth2ClientConfig) -> Result<Self, anyhow::Error> {
        let conf = config.clone();
        let client = BasicClient::new(ClientId::new(conf.client_id))
            .set_client_secret(ClientSecret::new(conf.client_secret))
            .set_auth_uri(AuthUrl::new(conf.auth_url)?)
            .set_token_uri(TokenUrl::new(conf.token_url)?)
            // Set the URL the user will be redirected to after the authorization process.
            .set_redirect_uri(RedirectUrl::new(conf.redirect_uri)?);

        Ok(Self {
            config,
            client,
            http: Self::make_http_client()?,
        })
    }

    fn make_http_client() -> Result<reqwest::Client, reqwest::Error> {
        reqwest::ClientBuilder::new()
            .user_agent(USER_AGENT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
    }

    pub fn client_id(&self) -> &str {
        &self.config.client_id
    }

    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Gets the decoding key from the JWKS URL
    pub async fn decoding_key(&self) -> Result<DecodingKey, anyhow::Error> {
        let response = self.http.get(&self.config.jwks_url).send().await?;

        let jwk_set: jsonwebtoken::jwk::JwkSet = response
            .json::<jsonwebtoken::jwk::JwkSet>()
            .await
            .context("Failed to parse JWK")?;

        let jwk = jwk_set.keys.first().context("No JWK found")?;

        Ok(DecodingKey::from_jwk(jwk)?)
    }

    /// Generates a URL to redirect the user to for authorization
    pub async fn authorize_url(
        &self,
        pool: &PgPool,
        client: &OAuthRedirectClient,
    ) -> Result<Url, sqlx::Error> {
        // Generate a PKCE challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate the full authorization URL
        let mut auth_url_builder = self
            .client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge);
        for scope in self.config.scopes.iter() {
            auth_url_builder = auth_url_builder.add_scope(Scope::new(scope.to_string()));
        }
        let (auth_url, csrf_token) = auth_url_builder.url();

        // Store the PKCE verifier in the database alongside the CSRF token
        UserOAuthState::cleanup_expired(pool).await?;
        InsertUserOAuthState {
            client: *client,
            provider: self.config.name.clone(),
            csrf_token: csrf_token.clone().into_secret(),
            pkce_verifier: pkce_verifier.into_secret(),
        }
        .insert(pool)
        .await?;

        Ok(auth_url)
    }

    /// Exchanges the code for an access token
    pub async fn exchange_code(
        &self,
        oauth_state: &UserOAuthState,
        code: &str,
    ) -> Result<ExternalJwtToken, anyhow::Error> {
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(oauth_state.pkce_verifier.clone()))
            .request_async(&self.http)
            .await
            .context("Failed to exchange code")?;
        let access_token = token_response.access_token().secret().to_string();

        let decoding_key = self
            .decoding_key()
            .await
            .context("Failed to get decoding key")?;
        let jwt = ExternalJwtToken::parse(self, access_token, &decoding_key)
            .context("Failed to parse JWT")?;

        Ok(jwt)
    }
}

#[derive(Debug, Clone)]
pub struct OAuth2ClientConfig {
    name: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    auth_url: String,
    token_url: String,
    jwks_url: String,
    scopes: Vec<String>,
}

impl OAuth2ClientConfig {
    pub fn from_config(
        config: &config::OAuth2ClientConfig,
        name: &str,
        public_origin: &Url,
    ) -> Self {
        Self {
            name: name.to_string(),
            client_id: config.client_id.clone(),
            client_secret: config.client_secret.clone(),
            auth_url: config.auth_url.clone(),
            token_url: config.token_url.clone(),
            redirect_uri: public_origin
                .join("/api/v1/user/oauth/callback")
                .unwrap()
                .to_string(),
            jwks_url: config.jwks_url.clone(),
            scopes: config.scopes.split(',').map(|s| s.to_string()).collect(),
        }
    }
}
