use async_trait::async_trait;
use moka::future::Cache;
use std::collections::HashMap;

use crate::config::AppConfig;
use crate::errors::{WikiError, WikiResult};
use crate::models::{SupportedLanguage, WikidataResponse, WikipediaLanguage};
use crate::utils::clean_description;

#[async_trait]
pub trait WikidataApi {
    async fn get_descriptions(
        &self,
        wikidata_ids: Vec<String>,
        language: SupportedLanguage,
    ) -> WikiResult<HashMap<String, String>>;
}

pub struct WikidataService {
    client: reqwest::Client,
    cache: Cache<String, HashMap<String, String>>,
}

impl WikidataService {
    pub fn new(config: AppConfig) -> WikiResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(config.http_timeout())
            .user_agent(&config.wikipedia.user_agent)
            .build()
            .map_err(|e| WikiError::internal(format!("Failed to create HTTP client: {e}")))?;

        let cache = Cache::builder()
            .time_to_live(config.cache_ttl())
            .max_capacity(config.cache.max_capacity)
            .build();

        Ok(Self { client, cache })
    }

    fn cache_key(&self, wikidata_ids: &[String], language: SupportedLanguage) -> String {
        let mut sorted_ids = wikidata_ids.to_vec();
        sorted_ids.sort();
        format!("wikidata:{}:{:?}", language.code(), sorted_ids)
    }

    async fn get_descriptions_internal(
        &self,
        wikidata_ids: Vec<String>,
        language: SupportedLanguage,
    ) -> WikiResult<HashMap<String, String>> {
        if wikidata_ids.is_empty() {
            return Ok(HashMap::new());
        }

        const WIKIDATA_API_URL: &str = "https://www.wikidata.org/w/api.php";

        let ids_str = wikidata_ids.join("|");

        let params = [
            ("action", "wbgetentities"),
            ("format", "json"),
            ("ids", &ids_str),
            ("props", "descriptions"),
            ("languages", language.code()),
        ];

        let response = self
            .client
            .get(WIKIDATA_API_URL)
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(WikiError::Network(response.error_for_status().unwrap_err()));
        }

        let wikidata_response: WikidataResponse = response.json().await?;

        let mut descriptions = HashMap::new();

        for (entity_id, entity) in wikidata_response.entities {
            if let Some(entity_descriptions) = entity.descriptions {
                if let Some(description) = entity_descriptions.get(language.code()) {
                    let cleaned_description = clean_description(&description.value);
                    if !cleaned_description.is_empty() {
                        descriptions.insert(entity_id, cleaned_description);
                    }
                }
            }
        }

        Ok(descriptions)
    }
}

#[async_trait]
impl WikidataApi for WikidataService {
    async fn get_descriptions(
        &self,
        wikidata_ids: Vec<String>,
        language: SupportedLanguage,
    ) -> WikiResult<HashMap<String, String>> {
        if wikidata_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let cache_key = self.cache_key(&wikidata_ids, language);

        if let Some(cached_result) = self.cache.get(&cache_key).await {
            return Ok(cached_result);
        }
        let descriptions = self
            .get_descriptions_internal(wikidata_ids, language)
            .await?;

        self.cache.insert(cache_key, descriptions.clone()).await;

        Ok(descriptions)
    }
}

pub async fn get_wikidata_descriptions_batch_lang(
    wikidata_ids: Vec<String>,
    language: &WikipediaLanguage,
) -> WikiResult<HashMap<String, String>> {
    let config = crate::config::AppConfig::from_env()?;
    let service = WikidataService::new(config)?;

    service
        .get_descriptions(wikidata_ids, language.inner())
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_wikidata_ids() {
        let config = AppConfig::from_env().unwrap();
        let service = WikidataService::new(config).unwrap();

        let result = service
            .get_descriptions(vec![], SupportedLanguage::English)
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_cache_key_generation() {
        let config = AppConfig::from_env().unwrap();
        let service = WikidataService::new(config).unwrap();

        let key1 = service.cache_key(
            &["Q123".to_string(), "Q456".to_string()],
            SupportedLanguage::English,
        );
        let key2 = service.cache_key(
            &["Q456".to_string(), "Q123".to_string()],
            SupportedLanguage::English,
        );

        assert_eq!(key1, key2); // Должны быть одинаковыми (порядок не важен)

        let key3 = service.cache_key(
            &["Q123".to_string(), "Q456".to_string()],
            SupportedLanguage::Russian,
        );
        assert_ne!(key1, key3); // Разные языки
    }
}
