use serde::Deserialize;
use std::time::Duration;

pub mod languages;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub telegram: TelegramConfig,
    pub wikipedia: WikipediaConfig,
    pub cache: CacheConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WikipediaConfig {
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,

    #[serde(default = "default_max_results")]
    pub max_search_results: usize,

    #[serde(default = "default_max_description_length")]
    pub max_description_length: usize,

    #[serde(default = "default_max_content_length")]
    pub max_content_length: usize,

    #[serde(default = "default_user_agent")]
    pub user_agent: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_cache_capacity")]
    pub max_capacity: u64,

    #[serde(default = "default_cache_ttl_secs")]
    pub ttl_secs: u64,

    #[serde(default = "default_enable_cache")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,

    #[serde(default = "default_log_format")]
    pub format: LogFormat,

    #[serde(default = "default_enable_console")]
    pub console: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, crate::errors::WikiError> {
        let bot_token = std::env::var("TELOXIDE_TOKEN")
            .or_else(|_| std::env::var("BOT_TOKEN"))
            .map_err(|_| {
                crate::errors::WikiError::config(
                    "TELOXIDE_TOKEN or BOT_TOKEN environment variable not set",
                )
            })?;

        Ok(AppConfig {
            telegram: TelegramConfig {
                bot_token,
                request_timeout_secs: default_request_timeout(),
            },
            wikipedia: WikipediaConfig {
                request_timeout_secs: default_request_timeout(),
                max_search_results: default_max_results(),
                max_description_length: default_max_description_length(),
                max_content_length: default_max_content_length(),
                user_agent: default_user_agent(),
            },
            cache: CacheConfig {
                max_capacity: default_cache_capacity(),
                ttl_secs: default_cache_ttl_secs(),
                enabled: default_enable_cache(),
            },
            logging: LoggingConfig {
                level: std::env::var("RUST_LOG").unwrap_or_else(|_| default_log_level()),
                format: default_log_format(),
                console: default_enable_console(),
            },
        })
    }

    pub fn http_timeout(&self) -> Duration {
        Duration::from_secs(self.wikipedia.request_timeout_secs)
    }

    pub fn cache_ttl(&self) -> Duration {
        Duration::from_secs(self.cache.ttl_secs)
    }
}

fn default_request_timeout() -> u64 {
    30
}
fn default_max_results() -> usize {
    50
}
fn default_max_description_length() -> usize {
    100
}
fn default_max_content_length() -> usize {
    300
}
fn default_cache_capacity() -> u64 {
    1000
}
fn default_cache_ttl_secs() -> u64 {
    300
}
fn default_enable_cache() -> bool {
    true
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> LogFormat {
    LogFormat::Pretty
}
fn default_enable_console() -> bool {
    true
}
fn default_user_agent() -> String {
    "WikipediaArticlesBot/1.1.0 (https://github.com/Newmcpe/wiki-article-finder-telegram)"
        .to_string()
}
