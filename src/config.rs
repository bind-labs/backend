use clap::Parser;
use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use reqwest::Url;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

// CLI arguments struct - only for non-OAuth settings
#[derive(Parser, Debug, Default)]
pub struct CliArgs {
    /// Path to config file
    #[arg(long)]
    pub config_file: Option<PathBuf>,

    /// Postgres Database URL
    #[arg(long)]
    pub database_url: Option<String>,

    /// Host to bind to
    #[arg(long)]
    pub host: Option<String>,
}

// Main config struct
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Postgres Database URL
    pub database_url: String,

    /// Host to bind to
    pub host: String,

    #[cfg(feature = "flaresolverr")]
    pub flaresolverr_host: String,

    /// Used for OAuth redirects to the Web client
    pub web_origin: Url,
    /// Used for OAuth redirects to the Android client
    pub android_origin: Url,
    /// Used for OAuth redirects to the iOS client
    pub ios_origin: Url,

    /// Secret to use when signing JWTs
    pub jwt_secret: String,

    #[serde(default)]
    pub oauth: HashMap<String, OAuth2ClientConfig>,

    /// SMTP URI to use for sending emails
    pub smtp_uri: String,
    /// SMTP email that will be used as the sender
    pub smtp_from: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OAuth2ClientConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub jwks_url: String,
    pub scopes: String,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let cli_args = CliArgs::parse();

        let mut builder = ConfigBuilder::builder();

        // Add config file specified via CLI if provided
        if let Some(config_path) = cli_args.config_file {
            builder = builder.add_source(File::with_name(config_path.to_str().unwrap()));
        }

        // Add environment variables with prefix "BIND_"
        builder = builder.add_source(
            Environment::with_prefix("bind")
                .prefix_separator("_")
                .separator("__")
                .list_separator(",")
                .with_list_parse_key("oauth"),
        );

        // Override with CLI arguments if provided
        if let Some(database_url) = cli_args.database_url {
            builder = builder.set_override("database_url", database_url)?;
        }
        if let Some(host) = cli_args.host {
            builder = builder.set_override("host", host)?;
        }

        // Build and deserialize the config
        builder.build()?.try_deserialize()
    }
}
