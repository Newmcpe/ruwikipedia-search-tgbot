use serde::Deserialize;
use std::error::Error;

// Структура для результатов поиска в Wikipedia
#[derive(Debug, Deserialize)]
pub struct WikipediaSearchItem {
    pub title: String,
    pub snippet: String,
    pub pageid: Option<u64>,
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

// Функция для получения изображения статьи
pub async fn get_article_image(
    pageid: u64,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let image_url = format!(
        "https://ru.wikipedia.org/w/api.php?action=query&pageids={}&format=json&prop=pageimages&piprop=thumbnail&pithumbsize=300",
        pageid
    );

    let response = client
        .get(&image_url)
        .header("User-Agent", "WikiArticleFinderBot/1.0")
        .send()
        .await?;

    let image_result: serde_json::Value = response.json().await?;

    if let Some(query) = image_result.get("query").and_then(|q| q.as_object()) {
        if let Some(pages) = query.get("pages").and_then(|p| p.as_object()) {
            if let Some(page) = pages.values().next() {
                if let Some(thumbnail) = page.get("thumbnail").and_then(|t| t.as_object()) {
                    if let Some(source) = thumbnail.get("source").and_then(|s| s.as_str()) {
                        return Ok(Some(source.to_string()));
                    }
                }
            }
        }
    }

    Ok(None)
}

// Функция для получения краткого описания статьи
pub async fn get_article_extract(
    pageid: u64,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let extract_url = format!(
        "https://ru.wikipedia.org/w/api.php?action=query&pageids={}&format=json&prop=extracts&exintro=1&explaintext=1",
        pageid
    );

    let response = client
        .get(&extract_url)
        .header("User-Agent", "WikiArticleFinderBot/1.0")
        .send()
        .await?;

    let extract_result: serde_json::Value = response.json().await?;

    if let Some(query) = extract_result.get("query").and_then(|q| q.as_object()) {
        if let Some(pages) = query.get("pages").and_then(|p| p.as_object()) {
            if let Some(page) = pages.values().next() {
                if let Some(extract) = page.get("extract").and_then(|e| e.as_str()) {
                    return Ok(Some(extract.to_string()));
                }
            }
        }
    }

    Ok(None)
}

// Функция для получения изображения и краткого описания статьи
pub async fn get_article_details(
    pageid: u64,
) -> Result<(Option<String>, Option<String>), Box<dyn Error + Send + Sync>> {
    // Получаем изображение и описание параллельно
    let (image_result, extract_result) =
        tokio::join!(get_article_image(pageid), get_article_extract(pageid));

    let image_url = image_result?;
    let extract = extract_result?;

    Ok((image_url, extract))
}

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

// Функция для получения Wikidata ID из Wikipedia статьи
pub async fn get_wikidata_id(pageid: u64) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let wikidata_url = format!(
        "https://ru.wikipedia.org/w/api.php?action=query&pageids={}&format=json&prop=pageprops&ppprop=wikibase_item",
        pageid
    );

    let response = client
        .get(&wikidata_url)
        .header("User-Agent", "WikiArticleFinderBot/1.0")
        .send()
        .await?;

    let wikidata_result: serde_json::Value = response.json().await?;

    if let Some(query) = wikidata_result.get("query").and_then(|q| q.as_object()) {
        if let Some(pages) = query.get("pages").and_then(|p| p.as_object()) {
            if let Some(page) = pages.values().next() {
                if let Some(pageprops) = page.get("pageprops").and_then(|pp| pp.as_object()) {
                    if let Some(wikibase_item) =
                        pageprops.get("wikibase_item").and_then(|wi| wi.as_str())
                    {
                        return Ok(Some(wikibase_item.to_string()));
                    }
                }
            }
        }
    }

    Ok(None)
}

// Функция для получения описания из Wikidata
pub async fn get_wikidata_description(
    wikidata_id: &str,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let description_url = format!(
        "https://www.wikidata.org/w/api.php?action=wbgetentities&ids={}&format=json&props=descriptions&languages=ru",
        wikidata_id
    );

    let response = client
        .get(&description_url)
        .header("User-Agent", "WikiArticleFinderBot/1.0")
        .send()
        .await?;

    let description_result: serde_json::Value = response.json().await?;

    let description = description_result
        .get("entities")
        .and_then(|e| e.as_object())
        .and_then(|entities| entities.get(wikidata_id))
        .and_then(|entity| entity.get("descriptions").and_then(|d| d.as_object()))
        .and_then(|descriptions| descriptions.get("ru").and_then(|ru| ru.as_object()))
        .and_then(|ru_desc| ru_desc.get("value").and_then(|v| v.as_str()));

    match description {
        Some(value) => Ok(Some(value.to_string())),
        None => Ok(None),
    }
}

// Функция для получения описания из Wikidata через pageid
pub async fn get_wikidata_description_by_pageid(
    pageid: u64,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    if let Some(wikidata_id) = get_wikidata_id(pageid).await? {
        get_wikidata_description(&wikidata_id).await
    } else {
        Ok(None)
    }
}

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
