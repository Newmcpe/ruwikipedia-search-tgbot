use thiserror::Error;

#[derive(Debug, Error)]
pub enum WikiError {
    #[error("Сетевая ошибка: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Ошибка парсинга JSON: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("Ошибка парсинга URL: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Не найдено результатов по запросу: '{query}'")]
    NoResults { query: String },

    #[error("Неподдерживаемый код языка: '{code}'")]
    InvalidLanguage { code: String },

    #[error("Превышено время ожидания запроса")]
    Timeout,

    #[error("Ответ API содержит неожиданную структуру")]
    UnexpectedApiResponse,

    #[error("Ошибка кэша: {message}")]
    Cache { message: String },

    #[error("Ошибка конфигурации: {message}")]
    Config { message: String },

    #[error("Внутренняя ошибка: {message}")]
    Internal { message: String },
}

impl WikiError {
    pub fn cache(message: impl Into<String>) -> Self {
        Self::Cache {
            message: message.into(),
        }
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

pub type WikiResult<T> = Result<T, WikiError>;

pub trait UserFriendlyError {
    fn user_message(&self) -> String;
}

impl UserFriendlyError for WikiError {
    fn user_message(&self) -> String {
        match self {
            WikiError::Network(_) => "🔌 Проблемы с подключением. Попробуйте позже.".to_string(),
            WikiError::Parse(_) => "⚠️ Ошибка обработки данных от Wikipedia.".to_string(),
            WikiError::UrlParse(_) => "🔗 Неверный формат ссылки.".to_string(),
            WikiError::NoResults { query } => {
                format!("🔍 По запросу \"{}\" ничего не найдено.", query)
            }
            WikiError::InvalidLanguage { code } => format!("🌍 Язык '{}' не поддерживается.", code),
            WikiError::Timeout => "⏱️ Превышено время ожидания. Попробуйте позже.".to_string(),
            WikiError::UnexpectedApiResponse => {
                "📡 Неожиданный ответ от Wikipedia API.".to_string()
            }
            WikiError::Cache { .. } => "💾 Проблемы с кэшем данных.".to_string(),
            WikiError::Config { .. } => "⚙️ Ошибка конфигурации бота.".to_string(),
            WikiError::Internal { .. } => {
                "🛠️ Внутренняя ошибка. Обратитесь к администратору.".to_string()
            }
        }
    }
}
