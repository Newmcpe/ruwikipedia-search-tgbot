use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SupportedLanguage {
    #[serde(rename = "ru")]
    Russian,
    #[serde(rename = "uk")]
    Ukrainian,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "it")]
    Italian,
    #[serde(rename = "pt")]
    Portuguese,
    #[serde(rename = "pl")]
    Polish,
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "zh")]
    Chinese,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "ar")]
    Arabic,
    #[serde(rename = "he")]
    Hebrew,
    #[serde(rename = "tr")]
    Turkish,
    #[serde(rename = "nl")]
    Dutch,
    #[serde(rename = "sv")]
    Swedish,
    #[serde(rename = "no")]
    Norwegian,
    #[serde(rename = "da")]
    Danish,
    #[serde(rename = "fi")]
    Finnish,
    #[serde(rename = "cs")]
    Czech,
    #[serde(rename = "bg")]
    Bulgarian,
    #[serde(rename = "hr")]
    Croatian,
    #[serde(rename = "sr")]
    Serbian,
    #[serde(rename = "sk")]
    Slovak,
    #[serde(rename = "sl")]
    Slovenian,
    #[serde(rename = "hu")]
    Hungarian,
    #[serde(rename = "ro")]
    Romanian,
    #[serde(rename = "el")]
    Greek,
    #[serde(rename = "lv")]
    Latvian,
    #[serde(rename = "lt")]
    Lithuanian,
    #[serde(rename = "et")]
    Estonian,
    #[serde(rename = "ca")]
    Catalan,
    #[serde(rename = "eu")]
    Basque,
    #[serde(rename = "gl")]
    Galician,
}

impl SupportedLanguage {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Russian => "ru",
            Self::Ukrainian => "uk",
            Self::English => "en",
            Self::German => "de",
            Self::French => "fr",
            Self::Spanish => "es",
            Self::Italian => "it",
            Self::Portuguese => "pt",
            Self::Polish => "pl",
            Self::Japanese => "ja",
            Self::Chinese => "zh",
            Self::Korean => "ko",
            Self::Arabic => "ar",
            Self::Hebrew => "he",
            Self::Turkish => "tr",
            Self::Dutch => "nl",
            Self::Swedish => "sv",
            Self::Norwegian => "no",
            Self::Danish => "da",
            Self::Finnish => "fi",
            Self::Czech => "cs",
            Self::Bulgarian => "bg",
            Self::Croatian => "hr",
            Self::Serbian => "sr",
            Self::Slovak => "sk",
            Self::Slovenian => "sl",
            Self::Hungarian => "hu",
            Self::Romanian => "ro",
            Self::Greek => "el",
            Self::Latvian => "lv",
            Self::Lithuanian => "lt",
            Self::Estonian => "et",
            Self::Catalan => "ca",
            Self::Basque => "eu",
            Self::Galician => "gl",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Russian => "русской",
            Self::Ukrainian => "украинской",
            Self::English => "английской",
            Self::German => "немецкой",
            Self::French => "французской",
            Self::Spanish => "испанской",
            Self::Italian => "итальянской",
            Self::Portuguese => "португальской",
            Self::Polish => "польской",
            Self::Japanese => "японской",
            Self::Chinese => "китайской",
            Self::Korean => "корейской",
            Self::Arabic => "арабской",
            Self::Hebrew => "иврит",
            Self::Turkish => "турецкой",
            Self::Dutch => "голландской",
            Self::Swedish => "шведской",
            Self::Norwegian => "норвежской",
            Self::Danish => "датской",
            Self::Finnish => "финской",
            Self::Czech => "чешской",
            Self::Bulgarian => "болгарской",
            Self::Croatian => "хорватской",
            Self::Serbian => "сербской",
            Self::Slovak => "словацкой",
            Self::Slovenian => "словенской",
            Self::Hungarian => "венгерской",
            Self::Romanian => "румынской",
            Self::Greek => "греческой",
            Self::Latvian => "латвийской",
            Self::Lithuanian => "литовской",
            Self::Estonian => "эстонской",
            Self::Catalan => "каталанской",
            Self::Basque => "баскской",
            Self::Galician => "галисийской",
        }
    }

    pub fn flag_emoji(&self) -> &'static str {
        match self {
            Self::Russian => "🇷🇺",
            Self::Ukrainian => "🇺🇦",
            Self::English => "🇺🇸",
            Self::German => "🇩🇪",
            Self::French => "🇫🇷",
            Self::Spanish => "🇪🇸",
            Self::Italian => "🇮🇹",
            Self::Portuguese => "🇵🇹",
            Self::Polish => "🇵🇱",
            Self::Japanese => "🇯🇵",
            Self::Chinese => "🇨🇳",
            Self::Korean => "🇰🇷",
            Self::Arabic => "🇸🇦",
            Self::Hebrew => "🇮🇱",
            Self::Turkish => "🇹🇷",
            Self::Dutch => "🇳🇱",
            Self::Swedish => "🇸🇪",
            Self::Norwegian => "🇳🇴",
            Self::Danish => "🇩🇰",
            Self::Finnish => "🇫🇮",
            Self::Czech => "🇨🇿",
            Self::Bulgarian => "🇧🇬",
            Self::Croatian => "🇭🇷",
            Self::Serbian => "🇷🇸",
            Self::Slovak => "🇸🇰",
            Self::Slovenian => "🇸🇮",
            Self::Hungarian => "🇭🇺",
            Self::Romanian => "🇷🇴",
            Self::Greek => "🇬🇷",
            Self::Latvian => "🇱🇻",
            Self::Lithuanian => "🇱🇹",
            Self::Estonian => "🇪🇪",
            Self::Catalan => "🏴󠁥󠁳󠁣󠁴󠁿",
            Self::Basque => "🏴󠁥󠁳󠁰󠁶󠁿",
            Self::Galician => "🏴󠁥󠁳󠁧󠁡󠁿",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "ru" => Some(Self::Russian),
            "uk" => Some(Self::Ukrainian),
            "en" => Some(Self::English),
            "de" => Some(Self::German),
            "fr" => Some(Self::French),
            "es" => Some(Self::Spanish),
            "it" => Some(Self::Italian),
            "pt" => Some(Self::Portuguese),
            "pl" => Some(Self::Polish),
            "ja" => Some(Self::Japanese),
            "zh" => Some(Self::Chinese),
            "ko" => Some(Self::Korean),
            "ar" => Some(Self::Arabic),
            "he" => Some(Self::Hebrew),
            "tr" => Some(Self::Turkish),
            "nl" => Some(Self::Dutch),
            "sv" => Some(Self::Swedish),
            "no" => Some(Self::Norwegian),
            "da" => Some(Self::Danish),
            "fi" => Some(Self::Finnish),
            "cs" => Some(Self::Czech),
            "bg" => Some(Self::Bulgarian),
            "hr" => Some(Self::Croatian),
            "sr" => Some(Self::Serbian),
            "sk" => Some(Self::Slovak),
            "sl" => Some(Self::Slovenian),
            "hu" => Some(Self::Hungarian),
            "ro" => Some(Self::Romanian),
            "el" => Some(Self::Greek),
            "lv" => Some(Self::Latvian),
            "lt" => Some(Self::Lithuanian),
            "et" => Some(Self::Estonian),
            "ca" => Some(Self::Catalan),
            "eu" => Some(Self::Basque),
            "gl" => Some(Self::Galician),
            _ => None,
        }
    }

    pub fn popular_languages() -> &'static [SupportedLanguage] {
        &[
            Self::Russian,
            Self::Ukrainian,
            Self::English,
            Self::German,
            Self::French,
            Self::Spanish,
        ]
    }

    pub fn all_languages() -> &'static [SupportedLanguage] {
        &[
            Self::Russian,
            Self::Ukrainian,
            Self::English,
            Self::German,
            Self::French,
            Self::Spanish,
            Self::Italian,
            Self::Portuguese,
            Self::Polish,
            Self::Japanese,
            Self::Chinese,
            Self::Korean,
            Self::Arabic,
            Self::Hebrew,
            Self::Turkish,
            Self::Dutch,
            Self::Swedish,
            Self::Norwegian,
            Self::Danish,
            Self::Finnish,
            Self::Czech,
            Self::Bulgarian,
            Self::Croatian,
            Self::Serbian,
            Self::Slovak,
            Self::Slovenian,
            Self::Hungarian,
            Self::Romanian,
            Self::Greek,
            Self::Latvian,
            Self::Lithuanian,
            Self::Estonian,
            Self::Catalan,
            Self::Basque,
            Self::Galician,
        ]
    }
}

impl fmt::Display for SupportedLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl Default for SupportedLanguage {
    fn default() -> Self {
        Self::Russian
    }
}

pub fn parse_query_with_language(query: &str) -> (SupportedLanguage, String) {
    if let Some(colon_pos) = query.find(':') {
        if colon_pos > 0 && colon_pos < 5 {
            let lang_code = &query[..colon_pos];
            let search_query = query[colon_pos + 1..].trim().to_string();

            if let Some(language) = SupportedLanguage::from_code(lang_code) {
                return (language, search_query);
            }
        }
    }

    (SupportedLanguage::default(), query.to_string())
}
