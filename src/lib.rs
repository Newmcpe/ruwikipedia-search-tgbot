pub mod config;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod services;
pub mod utils;

pub use config::AppConfig;
pub use errors::{UserFriendlyError, WikiError, WikiResult};
pub use handlers::*;
pub use models::*;
pub use services::*;

pub fn init_logging(config: &config::LoggingConfig) -> Result<(), WikiError> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.level));

    let subscriber = tracing_subscriber::registry().with(env_filter);

    match config.format {
        config::LogFormat::Json => {
            subscriber
                .with(tracing_subscriber::fmt::layer().json())
                .try_init()
                .map_err(|e| {
                    WikiError::config(format!("Failed to initialize JSON logging: {e}"))
                })?;
        }
        config::LogFormat::Pretty => {
            subscriber
                .with(
                    tracing_subscriber::fmt::layer()
                        .pretty()
                        .with_file(false)
                        .with_line_number(false)
                        .with_target(false)
                        .with_thread_ids(false)
                        .with_thread_names(false)
                        .with_ansi(true)
                        .with_level(true)
                        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE),
                )
                .try_init()
                .map_err(|e| {
                    WikiError::config(format!("Failed to initialize pretty logging: {e}"))
                })?;
        }
        config::LogFormat::Compact => {
            subscriber
                .with(
                    tracing_subscriber::fmt::layer()
                        .compact()
                        .with_file(false)
                        .with_line_number(false)
                        .with_target(false)
                        .with_thread_ids(false)
                        .with_thread_names(false)
                        .with_ansi(true)
                        .with_level(true)
                        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE),
                )
                .try_init()
                .map_err(|e| {
                    WikiError::config(format!("Failed to initialize compact logging: {e}"))
                })?;
        }
    }

    Ok(())
}

pub fn create_services(config: AppConfig) -> WikiResult<(WikipediaService, WikidataService)> {
    let wikipedia_service = WikipediaService::new(config.clone())?;
    let wikidata_service = WikidataService::new(config)?;

    Ok((wikipedia_service, wikidata_service))
}

pub fn create_handlers(
    wikipedia_service: std::sync::Arc<WikipediaService>,
    wikidata_service: std::sync::Arc<WikidataService>,
) -> (InlineQueryHandler, MessageHandler) {
    let inline_handler = InlineQueryHandler::new(wikipedia_service, wikidata_service);
    let message_handler = MessageHandler::new();

    (inline_handler, message_handler)
}

#[cfg(test)]
mod logging_tests {
    use super::*;

    #[test]
    fn test_logging_levels() {
        let config = config::LoggingConfig {
            level: "info".to_string(),
            format: config::LogFormat::Pretty,
            console: true,
        };

        init_logging(&config).unwrap();

        tracing::debug!("This debug message should not appear");
        tracing::info!("This info message should appear without stack trace");
        tracing::warn!("This warning message should appear without stack trace");
        tracing::error!("This error message should appear with stack trace");
    }
}
