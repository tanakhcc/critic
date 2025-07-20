//! Parse Config from config file

use std::{fs::read_to_string, path::Path, str::FromStr};

use leptos::config::LeptosOptions;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use tracing::{error, level_filters::LevelFilter};
use tracing_subscriber::filter::LevelParseError;

#[derive(Debug)]
pub enum ConfigError {
    TomlParse(toml::de::Error),
    ConfigFileRead(std::io::Error),
    PoolCreate(sqlx::Error),
    LogLevel(LevelParseError),
    GitlabAddrParse(oauth2::url::ParseError),
    PublicAddrParse(oauth2::url::ParseError),
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
                write!(f, "Unable to parse log_level: {e}")
            }
            Self::GitlabAddrParse(e) => {
                write!(
                    f,
                    "Unable to interpret gitlab_addr as addr while using it to build a url: {e}"
                )
            }
            Self::PublicAddrParse(e) => {
                write!(
                    f,
                    "Unable to interpret public_addr as addr while using it to build a url: {e}"
                )
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
    /// Where is this website called from on the internet (including any reverse-proxies, NAT etc.)
    /// Gitlab must be able to communicate with critic via this FQDN, using https
    public_addr: String,
}

#[derive(Deserialize)]
struct OauthConfigData {
    client_id: String,
    client_secret: String,
}

/// The OauthConfig that will be usable to create clients on the server side
#[derive(Deserialize)]
struct OauthConfig {
    /// the client ID we use to authenticate to gitlab
    client_id: oauth2::ClientId,
    /// the client secret we use to authenticate to gitlab
    client_secret: oauth2::ClientSecret,
    auth_url: oauth2::AuthUrl,
    token_url: oauth2::TokenUrl,
    redirect_url: oauth2::RedirectUrl,
}
impl OauthConfig {
    fn try_from_config_data(
        value: OauthConfigData,
        gitlab_addr: &str,
        public_addr: &str,
    ) -> Result<Self, ConfigError> {
        Ok(Self {
            client_id: oauth2::ClientId::new(value.client_id),
            client_secret: oauth2::ClientSecret::new(value.client_secret),
            auth_url: oauth2::AuthUrl::new(format!("https://{gitlab_addr}/oauth/authorize"))
                .map_err(ConfigError::GitlabAddrParse)?,
            token_url: oauth2::TokenUrl::new(format!("https://{gitlab_addr}/oauth/token"))
                .map_err(ConfigError::GitlabAddrParse)?,
            redirect_url: oauth2::RedirectUrl::new(format!("https://{public_addr}/oauth/redirect"))
                .map_err(ConfigError::PublicAddrParse)?,
        })
    }
}
pub type OauthClient = oauth2::Client<
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::StandardTokenIntrospectionResponse<
        oauth2::EmptyExtraTokenFields,
        oauth2::basic::BasicTokenType,
    >,
    oauth2::StandardRevocableToken,
    oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
>;
impl From<OauthConfig> for OauthClient {
    fn from(value: OauthConfig) -> Self {
        oauth2::basic::BasicClient::new(value.client_id)
            .set_client_secret(value.client_secret)
            .set_auth_uri(value.auth_url)
            .set_token_uri(value.token_url)
            .set_redirect_uri(value.redirect_url)
    }
}

/// Config partaining to the gitlab instance
#[derive(Deserialize, Debug)]
pub struct GitlabConfig {
    /// the address where we can talk to gitlab
    pub addr: String,
    /// The name of the group, access to which configures access to the project (Dev / Maintainer)
    ///
    /// NOT url-encoded, e.g. `supergroup/subgroup`
    pub group_name: String,
    /// The url-encoded name of the main project to interact with
    ///
    /// This project MUST live in the namespace given by group_name
    pub project_name: String,
}

/// The config data as it is present in (a well-formed) toml config file
#[derive(Deserialize)]
struct ConfigData {
    db: DbConfigData,
    web: WebConfigData,
    log_level: Option<String>,
    oauth: OauthConfigData,
    /// used as server part for determining where to communicate to gitlab
    gitlab: GitlabConfig,
    /// The directory where xml and image files should live
    ///
    /// critic will create the required substructure there
    /// This directory needs to be backed up together with the DB to restore from a backup
    data_directory: String,
    /// how many worker threads should we use at most?
    ///
    /// Note that these will run CPU-heavy tasks and this number of threads will be fully utilised
    /// from time to time
    #[serde(default = "default_worker_threads")]
    worker_threads: u8,
}
fn default_worker_threads() -> u8 {
    4
}

/// The main config object that will be available across the Serverside application
#[derive(Debug)]
pub struct Config {
    // DB pool to use
    pub db: Pool<Postgres>,
    pub leptos_options: LeptosOptions,
    pub log_level: LevelFilter,
    pub oauth_client: OauthClient,
    /// used as server part for determining where to communicate to gitlab
    pub gitlab: GitlabConfig,
    pub data_directory: String,
    pub worker_threads: u8,
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
        let log_level = tracing_subscriber::filter::LevelFilter::from_str(
            &value.log_level.unwrap_or("INFO".to_string()),
        )?;

        Ok(Self {
            db,
            leptos_options,
            log_level,
            oauth_client: OauthConfig::try_from_config_data(
                value.oauth,
                &value.gitlab.addr,
                &value.web.public_addr,
            )?
            .into(),
            gitlab: value.gitlab,
            data_directory: value.data_directory,
            worker_threads: value.worker_threads,
        })
    }

    pub async fn try_create() -> Result<Self, ConfigError> {
        let path = Path::new("/etc/critic/config.toml");
        let content = read_to_string(path).map_err(ConfigError::ConfigFileRead)?;
        let config_data: ConfigData = toml::from_str(&content).map_err(ConfigError::TomlParse)?;
        Self::try_from_config_data(config_data).await
    }
}
