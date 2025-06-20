//! Parse Config from config file

use std::{fs::read_to_string, path::Path, str::FromStr};

use leptos::config::LeptosOptions;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use tracing::{error, level_filters::LevelFilter};
use tracing_subscriber::filter::LevelParseError;

#[derive(Debug)]
pub(crate) enum ConfigError {
    TomlParse(toml::de::Error),
    ConfigFileRead(std::io::Error),
    PoolCreate(sqlx::Error),
    LogLevel(LevelParseError),
}
impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::TomlParse(e) => {
                write!(f, "Unable to parse config file as toml: {e}")
            }
            Self::ConfigFileRead(e) => {
                write!(f, "Unable to read config file: {e}")
            }
            Self::PoolCreate(e) => {
                write!(f, "Unable to create postgres DB pool: {e}")
            }
            Self::LogLevel(e) => {
                write!(f, "Unable to parse log level: {e}")
            }
        }
    }
}
impl From<sqlx::Error> for ConfigError {
    fn from(value: sqlx::Error) -> Self {
        Self::PoolCreate(value)
    }
}
impl From<LevelParseError> for ConfigError {
    fn from(value: LevelParseError) -> Self {
        Self::LogLevel(value)
    }
}
impl std::error::Error for ConfigError {}

#[derive(Deserialize)]
struct DbConfigData {
    user: String,
    password: String,
    host: String,
    port: Option<u16>,
    database: String,
}

#[derive(Deserialize)]
struct WebConfigData {
    /// The address to host the website on (e.g. 127.0.0.1:8080)
    site_addr: String,
}

#[derive(Deserialize)]
struct ConfigData {
    db: DbConfigData,
    web: WebConfigData,
    log_level: Option<String>,
}

pub struct Config {
    // DB pool to use
    pub db: Pool<Postgres>,
    pub leptos_options: LeptosOptions,
    pub log_level: LevelFilter,
}
impl Config {
    async fn try_from_config_data(value: ConfigData) -> Result<Self, ConfigError> {
        // postgres settings
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            value.db.user,
            value.db.password,
            value.db.host,
            value.db.port.unwrap_or(5432),
            value.db.database
        );
        let db = match sqlx::postgres::PgPool::connect(&url).await {
            Ok(x) => x,
            Err(e) => {
                error!("Could not connect to postgres: {e}");
                return Err(e.into());
            }
        };

        let addr = std::net::SocketAddr::from_str(&value.web.site_addr)
            .expect("Should be able to parse socket addr");

        let leptos_options = LeptosOptions::builder()
            .output_name("critic")
            .site_root("target/site")
            .site_pkg_dir("pkg")
            .site_addr(addr)
            .build();
        let log_level = tracing_subscriber::filter::LevelFilter::from_str(&value.log_level.unwrap_or("INFO".to_string()))?;

        Ok(Self { db, leptos_options, log_level, })
    }

    pub async fn try_create() -> Result<Self, ConfigError> {
        let path = Path::new("/etc/critic/config.toml");
        let content = read_to_string(path).map_err(ConfigError::ConfigFileRead)?;
        let config_data: ConfigData = toml::from_str(&content).map_err(ConfigError::TomlParse)?;
        Self::try_from_config_data(config_data).await
    }
}
