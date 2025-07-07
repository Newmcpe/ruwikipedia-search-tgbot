use async_trait::async_trait;
use moka::future::Cache;
use std::collections::HashMap;

use crate::config::{AppConfig, WikipediaConfig};
use crate::errors::{WikiError, WikiResult};
use crate::models::{
    ArticleBatchInfo, Coordinates, EnrichedArticle, SupportedLanguage, UnifiedWikipediaResponse,
    WikipediaBatchResponse, WikipediaLanguage, WikipediaSearchItem, WikipediaSearchResponse,
};
use crate::utils::clean_html;

#[async_trait]
pub trait WikipediaApi {
    async fn search(
        &self,
        query: &str,
        language: SupportedLanguage,
    ) -> WikiResult<Vec<WikipediaSearchItem>>;

    async fn get_batch_info(
        &self,
        pageids: Vec<u64>,
        language: SupportedLanguage,
    ) -> WikiResult<HashMap<u64, ArticleBatchInfo>>;

    async fn get_enriched_articles(
        &self,
        query: &str,
        language: SupportedLanguage,
    ) -> WikiResult<Vec<EnrichedArticle>>;

    async fn get_enriched_articles_optimized(
        &self,
        query: &str,
        language: SupportedLanguage,
    ) -> WikiResult<Vec<EnrichedArticle>>;

    fn get_article_url(&self, title: &str, language: SupportedLanguage) -> String;
}

pub struct WikipediaService {
    client: reqwest::Client,
    config: WikipediaConfig,
    search_cache: Cache<String, Vec<WikipediaSearchItem>>,
    batch_cache: Cache<String, HashMap<u64, ArticleBatchInfo>>,
    unified_cache: Cache<String, Vec<EnrichedArticle>>,
}

impl WikipediaService {
    pub fn new(config: AppConfig) -> WikiResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(config.http_timeout())
            .user_agent(&config.wikipedia.user_agent)
            .build()
            .map_err(|e| WikiError::internal(format!("Failed to create HTTP client: {}", e)))?;

        let search_cache = Cache::builder()
            .time_to_live(config.cache_ttl())
            .max_capacity(config.cache.max_capacity)
            .build();

        let batch_cache = Cache::builder()
            .time_to_live(config.cache_ttl())
            .max_capacity(config.cache.max_capacity / 2)
            .build();

        let unified_cache = Cache::builder()
            .time_to_live(config.cache_ttl())
            .max_capacity(config.cache.max_capacity / 4)
            .build();

        Ok(Self {
            client,
            config: config.wikipedia,
            search_cache,
            batch_cache,
            unified_cache,
        })
    }

    fn search_cache_key(&self, query: &str, language: SupportedLanguage) -> String {
        format!("search:{}:{}", language.code(), query.to_lowercase())
    }

    fn batch_cache_key(&self, pageids: &[u64], language: SupportedLanguage) -> String {
        let mut sorted_pageids = pageids.to_vec();
        sorted_pageids.sort();
        format!("batch:{}:{:?}", language.code(), sorted_pageids)
    }

    async fn search_internal(
        &self,
        query: &str,
        language: SupportedLanguage,
    ) -> WikiResult<Vec<WikipediaSearchItem>> {
        let url = format!("https://{}.wikipedia.org/w/api.php", language.code());

        let params = [
            ("action", "query"),
            ("list", "search"),
            ("srsearch", query),
            ("format", "json"),
            ("srlimit", &self.config.max_search_results.to_string()),
            ("srprop", "snippet|titlesnippet|size|wordcount|timestamp"),
        ];

        let response = self.client.get(&url).query(&params).send().await?;

        if !response.status().is_success() {
            return Err(WikiError::Network(reqwest::Error::from(
                response.error_for_status().unwrap_err(),
            )));
        }

        let search_response: WikipediaSearchResponse = response.json().await?;

        let articles: Vec<WikipediaSearchItem> = search_response
            .query
            .search
            .into_iter()
            .map(|mut item| {
                item.snippet = clean_html(&item.snippet);
                item
            })
            .collect();

        Ok(articles)
    }

    async fn get_batch_info_internal(
        &self,
        pageids: Vec<u64>,
        language: SupportedLanguage,
    ) -> WikiResult<HashMap<u64, ArticleBatchInfo>> {
        if pageids.is_empty() {
            return Ok(HashMap::new());
        }

        let url = format!("https://{}.wikipedia.org/w/api.php", language.code());

        let pageids_str = pageids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join("|");

        let params = [
            ("action", "query"),
            ("format", "json"),
            ("pageids", &pageids_str),
            (
                "prop",
                "extracts|pageimages|pageprops|coordinates|categories",
            ),
            ("exintro", "1"),
            ("explaintext", "1"),
            ("exlimit", "max"),
            ("piprop", "thumbnail"),
            ("pithumbsize", "300"),
            ("pilimit", "max"),
            ("coprop", "lat|lon"),
            ("cllimit", "10"),
        ];

        let response = self.client.get(&url).query(&params).send().await?;

        if !response.status().is_success() {
            return Err(WikiError::Network(reqwest::Error::from(
                response.error_for_status().unwrap_err(),
            )));
        }

        let batch_response: WikipediaBatchResponse = response.json().await?;

        let mut result = HashMap::new();

        for (page_id_str, page_info) in batch_response.query.pages {
            if let Ok(page_id) = page_id_str.parse::<u64>() {
                let image_url = page_info
                    .thumbnail
                    .as_ref()
                    .map(|thumb| thumb.source.clone());

                let coordinates = page_info
                    .coordinates
                    .as_ref()
                    .and_then(|coords| coords.first())
                    .map(|coord| Coordinates {
                        lat: coord.lat,
                        lon: coord.lon,
                    });

                let categories = page_info
                    .categories
                    .unwrap_or_default()
                    .into_iter()
                    .map(|cat| cat.title)
                    .collect();

                let wikidata_id = page_info
                    .pageprops
                    .as_ref()
                    .and_then(|props| props.wikibase_item.clone());

                let batch_info = ArticleBatchInfo {
                    image_url,
                    extract: page_info.extract,
                    wikidata_id,
                    coordinates,
                    categories,
                };

                result.insert(page_id, batch_info);
            }
        }

        Ok(result)
    }

    async fn search_and_get_info_unified(
        &self,
        query: &str,
        language: SupportedLanguage,
    ) -> WikiResult<Vec<EnrichedArticle>> {
        if query.trim().is_empty() {
            return Err(WikiError::NoResults {
                query: query.to_string(),
            });
        }

        let url = format!("https://{}.wikipedia.org/w/api.php", language.code());

        let params = [
            ("action", "query"),
            ("format", "json"),
            ("generator", "search"),
            ("gsrsearch", query),
            ("gsrlimit", &self.config.max_search_results.to_string()),
            ("gsrprop", "snippet|titlesnippet|size|wordcount|timestamp"),
            (
                "prop",
                "extracts|pageimages|pageprops|coordinates|categories",
            ),
            ("exintro", "1"),
            ("explaintext", "1"),
            ("exlimit", "max"),
            ("piprop", "thumbnail"),
            ("pithumbsize", "300"),
            ("pilimit", "max"),
            ("coprop", "lat|lon"),
            ("cllimit", "10"),
        ];

        let response = self.client.get(&url).query(&params).send().await?;

        if !response.status().is_success() {
            return Err(WikiError::Network(reqwest::Error::from(
                response.error_for_status().unwrap_err(),
            )));
        }

        let unified_response: UnifiedWikipediaResponse = response.json().await?;

        let mut enriched_articles = Vec::new();

        for (_, page_info) in unified_response.query.pages {
            let image_url = page_info
                .thumbnail
                .as_ref()
                .map(|thumb| thumb.source.clone());

            let coordinates = page_info
                .coordinates
                .as_ref()
                .and_then(|coords| coords.first())
                .map(|coord| Coordinates {
                    lat: coord.lat,
                    lon: coord.lon,
                });

            let categories = page_info
                .categories
                .unwrap_or_default()
                .into_iter()
                .map(|cat| cat.title)
                .collect();

            let wikidata_id = page_info
                .pageprops
                .as_ref()
                .and_then(|props| props.wikibase_item.clone());

            let batch_info = ArticleBatchInfo {
                image_url,
                extract: page_info.extract,
                wikidata_id,
                coordinates,
                categories,
            };

            let basic_info = WikipediaSearchItem {
                title: page_info.title.clone(),
                snippet: String::new(),
                pageid: Some(page_info.pageid),
                size: None,
                wordcount: None,
                timestamp: None,
            };

            let article_url = self.get_article_url(&page_info.title, language);

            let enriched_article =
                EnrichedArticle::new(basic_info, Some(batch_info), None, article_url)
                    .with_relevance_index(page_info.index);

            enriched_articles.push(enriched_article);
        }

        enriched_articles.sort_by(|a, b| match (a.relevance_index, b.relevance_index) {
            (Some(idx_a), Some(idx_b)) => idx_a.cmp(&idx_b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => {
                let score_a = Self::calculate_article_score(a);
                let score_b = Self::calculate_article_score(b);
                score_b
                    .partial_cmp(&score_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        Ok(enriched_articles)
    }

    fn calculate_article_score(article: &EnrichedArticle) -> f64 {
        let mut score = 0.0;

        if let Some(batch_info) = &article.batch_info {
            if batch_info.image_url.is_some() {
                score += 10.0;
            }

            if let Some(extract) = &batch_info.extract {
                score += (extract.len() as f64 / 100.0).min(20.0);
            }

            if batch_info.wikidata_id.is_some() {
                score += 15.0;
            }

            if batch_info.coordinates.is_some() {
                score += 5.0;
            }

            score += batch_info.categories.len() as f64;
        }

        if let Some(wordcount) = article.basic_info.wordcount {
            score += (wordcount as f64 / 1000.0).min(30.0);
        }

        score
    }
}

#[async_trait]
impl WikipediaApi for WikipediaService {
    async fn search(
        &self,
        query: &str,
        language: SupportedLanguage,
    ) -> WikiResult<Vec<WikipediaSearchItem>> {
        if query.trim().is_empty() {
            return Err(WikiError::NoResults {
                query: query.to_string(),
            });
        }

        let cache_key = self.search_cache_key(query, language);

        if let Some(cached_result) = self.search_cache.get(&cache_key).await {
            return Ok(cached_result);
        }

        let articles = self.search_internal(query, language).await?;

        self.search_cache.insert(cache_key, articles.clone()).await;

        Ok(articles)
    }

    async fn get_batch_info(
        &self,
        pageids: Vec<u64>,
        language: SupportedLanguage,
    ) -> WikiResult<HashMap<u64, ArticleBatchInfo>> {
        if pageids.is_empty() {
            return Ok(HashMap::new());
        }

        let cache_key = self.batch_cache_key(&pageids, language);

        if let Some(cached_result) = self.batch_cache.get(&cache_key).await {
            return Ok(cached_result);
        }

        let batch_info = self.get_batch_info_internal(pageids, language).await?;

        self.batch_cache.insert(cache_key, batch_info.clone()).await;

        Ok(batch_info)
    }

    async fn get_enriched_articles(
        &self,
        query: &str,
        language: SupportedLanguage,
    ) -> WikiResult<Vec<EnrichedArticle>> {
        let articles = self.search(query, language).await?;

        if articles.is_empty() {
            return Err(WikiError::NoResults {
                query: query.to_string(),
            });
        }

        let pageids: Vec<u64> = articles
            .iter()
            .filter_map(|article| article.pageid)
            .collect();

        let batch_info = if !pageids.is_empty() {
            self.get_batch_info(pageids, language).await?
        } else {
            HashMap::new()
        };

        let enriched_articles = articles
            .into_iter()
            .enumerate()
            .filter_map(|(index, article)| {
                if let Some(pageid) = article.pageid {
                    let article_url = self.get_article_url(&article.title, language);
                    let batch_data = batch_info.get(&pageid).cloned();

                    let enriched_article =
                        EnrichedArticle::new(article, batch_data, None, article_url)
                            .with_relevance_index(Some(index as i32));

                    Some(enriched_article)
                } else {
                    None
                }
            })
            .collect();

        Ok(enriched_articles)
    }

    async fn get_enriched_articles_optimized(
        &self,
        query: &str,
        language: SupportedLanguage,
    ) -> WikiResult<Vec<EnrichedArticle>> {
        let cache_key = format!("unified:{}:{}", language.code(), query.to_lowercase());

        if let Some(cached_result) = self.unified_cache.get(&cache_key).await {
            return Ok(cached_result);
        }

        let result = self.search_and_get_info_unified(query, language).await;

        match &result {
            Ok(enriched_articles) => {
                self.unified_cache
                    .insert(cache_key, enriched_articles.clone())
                    .await;
            }
            Err(_) => {
                return self.get_enriched_articles(query, language).await;
            }
        }

        result
    }

    fn get_article_url(&self, title: &str, language: SupportedLanguage) -> String {
        format!(
            "https://{}.wikipedia.org/wiki/{}",
            language.code(),
            urlencoding::encode(title)
        )
    }
}

pub fn parse_query_with_language(query: &str) -> (SupportedLanguage, String) {
    crate::config::languages::parse_query_with_language(query)
}

pub fn get_article_url_lang(title: &str, language: &WikipediaLanguage) -> String {
    format!(
        "https://{}.wikipedia.org/wiki/{}",
        language.code(),
        urlencoding::encode(title)
    )
}

pub async fn search_wikipedia_lang(
    query: &str,
    language: &WikipediaLanguage,
) -> WikiResult<Vec<WikipediaSearchItem>> {
    let config = crate::config::AppConfig::from_env()?;
    let service = WikipediaService::new(config)?;

    service.search(query, language.inner()).await
}

pub async fn get_articles_batch_info_lang(
    pageids: Vec<u64>,
    language: &WikipediaLanguage,
) -> WikiResult<HashMap<u64, ArticleBatchInfo>> {
    let config = crate::config::AppConfig::from_env()?;
    let service = WikipediaService::new(config)?;

    service.get_batch_info(pageids, language.inner()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_key_generation() {
        let config = AppConfig::from_env().unwrap();
        let service = WikipediaService::new(config).unwrap();

        let key1 = service.search_cache_key("test", SupportedLanguage::English);
        let key2 = service.search_cache_key("Test", SupportedLanguage::English);

        assert_eq!(key1, key2); // Должны быть одинаковыми (case-insensitive)

        let key3 = service.search_cache_key("test", SupportedLanguage::Russian);
        assert_ne!(key1, key3); // Разные языки
    }

    #[test]
    fn test_get_article_url() {
        let config = AppConfig::from_env().unwrap();
        let service = WikipediaService::new(config).unwrap();

        let url = service.get_article_url("Test Article", SupportedLanguage::English);
        assert_eq!(url, "https://en.wikipedia.org/wiki/Test%20Article");

        let url_ru = service.get_article_url("Тест", SupportedLanguage::Russian);
        assert_eq!(
            url_ru,
            "https://ru.wikipedia.org/wiki/%D0%A2%D0%B5%D1%81%D1%82"
        );
    }
}
