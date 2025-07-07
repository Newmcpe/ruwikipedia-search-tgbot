use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikipediaSearchItem {
    pub title: String,
    pub snippet: String,
    pub pageid: Option<u64>,
    #[serde(default)]
    pub size: Option<u32>,
    #[serde(default)]
    pub wordcount: Option<u32>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleBatchInfo {
    pub image_url: Option<String>,
    pub extract: Option<String>,
    pub wikidata_id: Option<String>,
    #[serde(default)]
    pub coordinates: Option<Coordinates>,
    #[serde(default)]
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone)]
pub struct EnrichedArticle {
    pub basic_info: WikipediaSearchItem,
    pub batch_info: Option<ArticleBatchInfo>,
    pub wikidata_description: Option<String>,
    pub article_url: String,
    pub relevance_index: Option<i32>,
}

impl EnrichedArticle {
    pub fn new(
        basic_info: WikipediaSearchItem,
        batch_info: Option<ArticleBatchInfo>,
        wikidata_description: Option<String>,
        article_url: String,
    ) -> Self {
        Self {
            basic_info,
            batch_info,
            wikidata_description,
            article_url,
            relevance_index: None,
        }
    }

    pub fn best_description(&self, max_length: usize) -> String {
        // Всегда используем текст статьи для краткого описания для консистентности
        // Wikidata описание доступно через self.wikidata_description если нужно отдельно
        if let Some(ref batch_info) = self.batch_info {
            if let Some(ref extract) = batch_info.extract {
                if !extract.trim().is_empty() {
                    return truncate_string(extract, max_length);
                }
            }
        }

        // Fallback на snippet из search API
        if !self.basic_info.snippet.trim().is_empty() {
            return truncate_string(&self.basic_info.snippet, max_length);
        }

        // Последний fallback - название статьи
        format!("Статья из Википедии: {}", self.basic_info.title)
    }

    /// Получить Wikidata описание если доступно
    pub fn get_wikidata_description(&self) -> Option<&str> {
        self.wikidata_description.as_deref()
    }

    pub fn best_content(&self, max_length: usize) -> String {
        if let Some(ref batch_info) = self.batch_info {
            if let Some(ref extract) = batch_info.extract {
                if !extract.trim().is_empty() {
                    return truncate_string(extract, max_length);
                }
            }
        }

        truncate_string(&self.basic_info.snippet, max_length)
    }

    pub fn image_url(&self) -> Option<&str> {
        self.batch_info
            .as_ref()
            .and_then(|info| info.image_url.as_deref())
    }

    pub fn valid_image_url(&self) -> Option<Url> {
        self.image_url().and_then(|url| Url::parse(url).ok())
    }

    pub fn has_coordinates(&self) -> bool {
        self.batch_info
            .as_ref()
            .and_then(|info| info.coordinates.as_ref())
            .is_some()
    }

    pub fn word_count(&self) -> Option<u32> {
        self.basic_info.wordcount
    }

    pub fn with_relevance_index(mut self, index: Option<i32>) -> Self {
        self.relevance_index = index;
        self
    }
}

#[derive(Debug, Deserialize)]
pub struct WikipediaSearchResponse {
    pub query: WikipediaSearchQuery,
}

#[derive(Debug, Deserialize)]
pub struct WikipediaSearchQuery {
    pub search: Vec<WikipediaSearchItem>,
}

#[derive(Debug, Deserialize)]
pub struct WikipediaBatchResponse {
    pub query: WikipediaBatchQuery,
}

#[derive(Debug, Deserialize)]
pub struct WikipediaBatchQuery {
    pub pages: HashMap<String, WikipediaPageInfo>,
}

#[derive(Debug, Deserialize)]
pub struct WikipediaPageInfo {
    pub pageid: u64,
    pub title: String,
    #[serde(default)]
    pub extract: Option<String>,
    #[serde(default)]
    pub thumbnail: Option<WikipediaThumbnail>,
    #[serde(default)]
    pub pageimage: Option<String>,
    #[serde(default)]
    pub pageprops: Option<WikipediaPageProps>,
    #[serde(default)]
    pub coordinates: Option<Vec<WikipediaCoordinate>>,
    #[serde(default)]
    pub categories: Option<Vec<WikipediaCategory>>,
}

#[derive(Debug, Deserialize)]
pub struct WikipediaThumbnail {
    pub source: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct WikipediaPageProps {
    pub wikibase_item: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WikipediaCoordinate {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Deserialize)]
pub struct WikipediaCategory {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct WikidataResponse {
    pub entities: HashMap<String, WikidataEntity>,
}

#[derive(Debug, Deserialize)]
pub struct WikidataEntity {
    pub descriptions: Option<HashMap<String, WikidataDescription>>,
}

#[derive(Debug, Deserialize)]
pub struct WikidataDescription {
    pub language: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct UnifiedWikipediaResponse {
    pub query: UnifiedWikipediaQuery,
}

#[derive(Debug, Deserialize)]
pub struct UnifiedWikipediaQuery {
    pub pages: HashMap<String, UnifiedWikipediaPage>,
}

#[derive(Debug, Deserialize)]
pub struct UnifiedWikipediaPage {
    pub pageid: u64,
    pub title: String,
    pub index: Option<i32>,
    #[serde(default)]
    pub extract: Option<String>,
    #[serde(default)]
    pub thumbnail: Option<WikipediaThumbnail>,
    #[serde(default)]
    pub pageimage: Option<String>,
    #[serde(default)]
    pub pageprops: Option<WikipediaPageProps>,
    #[serde(default)]
    pub coordinates: Option<Vec<WikipediaCoordinate>>,
    #[serde(default)]
    pub categories: Option<Vec<WikipediaCategory>>,
}

fn truncate_string(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        text.to_string()
    } else {
        let mut truncated = text.chars().take(max_chars).collect::<String>();

        if let Some(last_space) = truncated.rfind(' ') {
            truncated.truncate(last_space);
        }

        format!("{truncated}...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("this is a long text", 10), "this is a...");
        assert_eq!(truncate_string("exactly_ten", 11), "exactly_ten");
    }

    #[test]
    fn test_enriched_article_best_description() {
        let basic_info = WikipediaSearchItem {
            title: "Test".to_string(),
            snippet: "Basic snippet".to_string(),
            pageid: Some(123),
            size: None,
            wordcount: None,
            timestamp: None,
        };

        let batch_info = ArticleBatchInfo {
            image_url: None,
            extract: Some("Better extract".to_string()),
            wikidata_id: None,
            coordinates: None,
            categories: vec![],
        };

        let article = EnrichedArticle::new(
            basic_info,
            Some(batch_info),
            Some("Best description".to_string()),
            "http://example.com".to_string(),
        );

        assert_eq!(article.best_description(100), "Better extract");
    }
}
