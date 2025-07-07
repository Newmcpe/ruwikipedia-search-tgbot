pub use crate::config::languages::SupportedLanguage;

#[derive(Debug, Clone)]
pub struct WikipediaLanguage {
    language: SupportedLanguage,
}

impl WikipediaLanguage {
    pub fn new(code: &str) -> Self {
        Self {
            language: SupportedLanguage::from_code(code).unwrap_or_default(),
        }
    }

    pub fn from_supported(language: SupportedLanguage) -> Self {
        Self { language }
    }

    pub fn code(&self) -> &str {
        self.language.code()
    }

    pub fn display_name(&self) -> &str {
        self.language.display_name()
    }

    pub fn flag_emoji(&self) -> &str {
        self.language.flag_emoji()
    }

    pub fn inner(&self) -> SupportedLanguage {
        self.language
    }

    pub fn russian() -> Self {
        Self::from_supported(SupportedLanguage::Russian)
    }

    pub fn ukrainian() -> Self {
        Self::from_supported(SupportedLanguage::Ukrainian)
    }

    pub fn english() -> Self {
        Self::from_supported(SupportedLanguage::English)
    }

    pub fn german() -> Self {
        Self::from_supported(SupportedLanguage::German)
    }

    pub fn french() -> Self {
        Self::from_supported(SupportedLanguage::French)
    }

    pub fn spanish() -> Self {
        Self::from_supported(SupportedLanguage::Spanish)
    }
}

impl Default for WikipediaLanguage {
    fn default() -> Self {
        Self::from_supported(SupportedLanguage::default())
    }
}

impl From<SupportedLanguage> for WikipediaLanguage {
    fn from(language: SupportedLanguage) -> Self {
        Self::from_supported(language)
    }
}

impl From<&str> for WikipediaLanguage {
    fn from(code: &str) -> Self {
        Self::new(code)
    }
}
