use serde::Deserialize;
use std::error::Error;

// Структура для результатов поиска в Wikipedia
#[derive(Debug, Deserialize)]
pub struct WikipediaSearchItem {
    pub title: String,
    pub snippet: String,
    pub pageid: Option<u64>,
}

// Структура для батчевого получения информации о статьях
#[derive(Debug)]
pub struct ArticleBatchInfo {
    pub image_url: Option<String>,
    pub extract: Option<String>,
    pub wikidata_id: Option<String>,
}

// Оптимизированная функция для получения всех данных о статьях одним запросом
pub async fn get_articles_batch_info(
    pageids: Vec<u64>,
) -> Result<std::collections::HashMap<u64, ArticleBatchInfo>, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    // Создаем строку с ID статей для запроса
    let pageids_str = pageids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join("|");

    // Объединенный запрос для получения изображений, выдержек и свойств страниц
    let batch_url = format!(
        "https://ru.wikipedia.org/w/api.php?action=query&pageids={}&format=json&prop=pageimages|extracts|pageprops&piprop=thumbnail&pithumbsize=300&exintro=1&explaintext=1&ppprop=wikibase_item",
        urlencoding::encode(&pageids_str)
    );

    let response = client
        .get(&batch_url)
        .header("User-Agent", "WikiArticleFinderBot/1.0")
        .send()
        .await?;

    let batch_result: serde_json::Value = response.json().await?;
    let mut results = std::collections::HashMap::new();

    if let Some(query) = batch_result.get("query").and_then(|q| q.as_object()) {
        if let Some(pages) = query.get("pages").and_then(|p| p.as_object()) {
            for (page_id_str, page_data) in pages {
                if let Ok(page_id) = page_id_str.parse::<u64>() {
                    let image_url = page_data
                        .get("thumbnail")
                        .and_then(|t| t.as_object())
                        .and_then(|thumb| thumb.get("source"))
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string());

                    let extract = page_data
                        .get("extract")
                        .and_then(|e| e.as_str())
                        .map(|s| s.to_string());

                    let wikidata_id = page_data
                        .get("pageprops")
                        .and_then(|pp| pp.as_object())
                        .and_then(|props| props.get("wikibase_item"))
                        .and_then(|wi| wi.as_str())
                        .map(|s| s.to_string());

                    results.insert(
                        page_id,
                        ArticleBatchInfo {
                            image_url,
                            extract,
                            wikidata_id,
                        },
                    );
                }
            }
        }
    }

    Ok(results)
}

// Оптимизированная функция для получения описаний из Wikidata батчем
pub async fn get_wikidata_descriptions_batch(
    wikidata_ids: Vec<String>,
) -> Result<std::collections::HashMap<String, String>, Box<dyn Error + Send + Sync>> {
    if wikidata_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let client = reqwest::Client::new();
    let ids_str = wikidata_ids.join("|");

    let description_url = format!(
        "https://www.wikidata.org/w/api.php?action=wbgetentities&ids={}&format=json&props=descriptions&languages=ru",
        urlencoding::encode(&ids_str)
    );

    let response = client
        .get(&description_url)
        .header("User-Agent", "WikiArticleFinderBot/1.0")
        .send()
        .await?;

    let description_result: serde_json::Value = response.json().await?;
    let mut results = std::collections::HashMap::new();

    if let Some(entities) = description_result
        .get("entities")
        .and_then(|e| e.as_object())
    {
        for (wikidata_id, entity_data) in entities {
            let description = entity_data
                .get("descriptions")
                .and_then(|d| d.as_object())
                .and_then(|descriptions| descriptions.get("ru"))
                .and_then(|ru| ru.as_object())
                .and_then(|ru_desc| ru_desc.get("value"))
                .and_then(|v| v.as_str());

            if let Some(desc) = description {
                results.insert(wikidata_id.clone(), desc.to_string());
            }
        }
    }

    Ok(results)
}

// Функция для поиска статей в русской Википедии
pub async fn search_wikipedia(
    query: &str,
) -> Result<Vec<WikipediaSearchItem>, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    // Поиск статей
    let search_url = format!(
        "https://ru.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&format=json&srlimit=10&srnamespace=0&srwhat=text",
        urlencoding::encode(query)
    );

    let response = client
        .get(&search_url)
        .header("User-Agent", "WikiArticleFinderBot/1.0")
        .send()
        .await?;

    let search_result: serde_json::Value = response.json().await?;

    let mut results = Vec::new();

    if let Some(query_obj) = search_result.get("query").and_then(|q| q.as_object()) {
        if let Some(search) = query_obj.get("search").and_then(|s| s.as_array()) {
            for page in search.iter().take(5) {
                if let (Some(title), Some(snippet), Some(pageid)) = (
                    page.get("title").and_then(|t| t.as_str()),
                    page.get("snippet").and_then(|s| s.as_str()),
                    page.get("pageid").and_then(|p| p.as_u64()),
                ) {
                    results.push(WikipediaSearchItem {
                        title: title.to_string(),
                        snippet: clean_html(snippet),
                        pageid: Some(pageid),
                    });
                }
            }
        }
    }

    Ok(results)
}

// Эти функции заменены на более эффективные батчевые запросы выше

// Функция для получения полного URL статьи
pub fn get_article_url(title: &str) -> String {
    format!(
        "https://ru.wikipedia.org/wiki/{}",
        urlencoding::encode(title)
    )
}

// Функция для очистки HTML-тегов
fn clean_html(text: &str) -> String {
    // Простая очистка HTML-тегов
    let mut result = String::new();
    let mut in_tag = false;

    for c in text.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }

    result
}

// Функция для безопасной обрезки строки по символам
pub fn truncate_string(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        text.chars().take(max_chars).collect::<String>() + "..."
    }
}

// Функция для экранирования символов в MarkdownV2
pub fn escape_markdown(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '_' | '*' | '[' | ']' | '(' | ')' | '~' | '`' | '>' | '#' | '+' | '-' | '=' | '|'
            | '{' | '}' | '.' | '!' => {
                format!("\\{}", c)
            }
            _ => c.to_string(),
        })
        .collect()
}

// Функция для экранирования URL в MarkdownV2
pub fn escape_markdown_url(url: &str) -> String {
    url.chars()
        .map(|c| match c {
            '(' | ')' | '\\' => {
                format!("\\{}", c)
            }
            _ => c.to_string(),
        })
        .collect()
}

// Эти функции заменены на более эффективный батчевый запрос get_wikidata_descriptions_batch выше

// Функция для очистки описаний от ссылок
pub fn clean_description(text: &str) -> String {
    let mut result = String::new();
    let words: Vec<&str> = text.split_whitespace().collect();

    for word in words {
        if word.starts_with("http://") || word.starts_with("https://") {
            continue;
        }

        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(word);
    }

    result
}
