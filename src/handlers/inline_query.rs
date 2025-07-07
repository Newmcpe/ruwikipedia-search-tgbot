use std::sync::Arc;
use teloxide::{
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResult, InlineQueryResultArticle,
        InputMessageContent, InputMessageContentText, ParseMode,
    },
};
use tracing::{error, info};

use crate::config::languages::SupportedLanguage;
use crate::errors::{UserFriendlyError, WikiError};
use crate::models::EnrichedArticle;
use crate::services::{WikidataApi, WikidataService, WikipediaApi, WikipediaService};
use crate::utils::{format_article_description, format_error_message, format_no_results_message};

pub struct InlineQueryHandler {
    wikipedia_service: Arc<WikipediaService>,
    wikidata_service: Arc<WikidataService>,
}

impl InlineQueryHandler {
    pub fn new(
        wikipedia_service: Arc<WikipediaService>,
        wikidata_service: Arc<WikidataService>,
    ) -> Self {
        Self {
            wikipedia_service,
            wikidata_service,
        }
    }

    pub async fn handle(&self, bot: Bot, q: InlineQuery) -> ResponseResult<()> {
        let query = q.query.trim();

        let user_info = q
            .from
            .username
            .as_ref()
            .map(|u| format!("@{u}"))
            .unwrap_or_else(|| format!("ID:{}", q.from.id));

        if !query.is_empty() {
            info!("🔍 {} ищет: '{}'", user_info, query);
        }

        let results = if query.is_empty() {
            self.handle_empty_query().await
        } else {
            self.handle_search_query(query).await
        };

        match results {
            Ok(inline_results) => {
                bot.answer_inline_query(q.id, inline_results).await?;
            }
            Err(e) => {
                error!("Error handling inline query: {:?}", e);
                let error_result = vec![self.create_error_result(&e)];
                bot.answer_inline_query(q.id, error_result).await?;
            }
        }

        Ok(())
    }

    async fn handle_empty_query(&self) -> Result<Vec<InlineQueryResult>, WikiError> {
        let keyboard = self.create_language_selection_keyboard();

        let result = InlineQueryResultArticle::new(
            "lang_select",
            "🌍 Выберите язык Википедии",
            InputMessageContent::Text(InputMessageContentText::new(
                "Выберите язык для поиска или используйте синтаксис:\n• `en:query` — English Wikipedia\n• `de:suche` — Deutsche Wikipedia\n• `fr:recherche` — Wikipédia français\n• `es:búsqueda` — Wikipedia español\n• `ru:запрос` — русская Википедия\n• `uk:запит` — українська Вікіпедія\n\nИли просто введите запрос (по умолчанию русская)"
            )),
        )
        .description("Поддерживается 100+ языков! Начните с кода языка")
        .reply_markup(keyboard);

        Ok(vec![InlineQueryResult::Article(result)])
    }

    async fn handle_search_query(&self, query: &str) -> Result<Vec<InlineQueryResult>, WikiError> {
        let (language, search_query) = crate::services::parse_query_with_language(query);

        let enriched_articles = match self
            .wikipedia_service
            .get_enriched_articles_optimized(&search_query, language)
            .await
        {
            Ok(articles) => articles,
            Err(_) => {
                self.wikipedia_service
                    .get_enriched_articles(&search_query, language)
                    .await?
            }
        };

        if enriched_articles.is_empty() {
            return Ok(vec![self.create_no_results_result(&search_query, language)]);
        }

        let wikidata_ids: Vec<String> = enriched_articles
            .iter()
            .filter_map(|article| {
                article
                    .batch_info
                    .as_ref()
                    .and_then(|info| info.wikidata_id.clone())
            })
            .collect();

        let wikidata_descriptions = if !wikidata_ids.is_empty() {
            self.wikidata_service
                .get_descriptions(wikidata_ids, language)
                .await
                .unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

        let results = self
            .build_article_results(enriched_articles, wikidata_descriptions)
            .await;

        Ok(results)
    }

    fn create_language_selection_keyboard(&self) -> InlineKeyboardMarkup {
        let popular_languages = SupportedLanguage::popular_languages();

        let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        for chunk in popular_languages.chunks(2) {
            let row: Vec<InlineKeyboardButton> = chunk
                .iter()
                .map(|lang| {
                    let display = format!("{} {}", lang.flag_emoji(), lang.display_name());
                    let query = format!("{}:", lang.code());
                    InlineKeyboardButton::switch_inline_query(display, query)
                })
                .collect();
            rows.push(row);
        }

        InlineKeyboardMarkup::new(rows)
    }

    async fn build_article_results(
        &self,
        mut enriched_articles: Vec<EnrichedArticle>,
        wikidata_descriptions: std::collections::HashMap<String, String>,
    ) -> Vec<InlineQueryResult> {
        tracing::debug!(
            "🏗️ Строим результаты для {} статей",
            enriched_articles.len()
        );

        enriched_articles.sort_by(|a, b| match (a.relevance_index, b.relevance_index) {
            (Some(idx_a), Some(idx_b)) => idx_a.cmp(&idx_b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => {
                let has_image_a = a.image_url().is_some();
                let has_image_b = b.image_url().is_some();

                if has_image_a && !has_image_b {
                    std::cmp::Ordering::Less
                } else if !has_image_a && has_image_b {
                    std::cmp::Ordering::Greater
                } else {
                    let word_count_a = a.word_count().unwrap_or(0);
                    let word_count_b = b.word_count().unwrap_or(0);
                    word_count_b.cmp(&word_count_a)
                }
            }
        });

        let mut results = Vec::new();

        for (idx, mut article) in enriched_articles.into_iter().enumerate() {
            if let Some(batch_info) = &article.batch_info {
                if let Some(wikidata_id) = &batch_info.wikidata_id {
                    if let Some(description) = wikidata_descriptions.get(wikidata_id) {
                        article.wikidata_description = Some(description.clone());
                    }
                }
            }

            let description = article.best_description(100);
            let content = article.best_content(300);

            let message_text = format_article_description(
                &article.basic_info.title,
                &content,
                &article.article_url,
            );

            let mut article_result = InlineQueryResultArticle::new(
                format!("article_{idx}"),
                &article.basic_info.title,
                InputMessageContent::Text(
                    InputMessageContentText::new(message_text).parse_mode(ParseMode::MarkdownV2),
                ),
            )
            .description(description);

            if let Some(image_url) = article.valid_image_url() {
                article_result = article_result.thumb_url(image_url);
            }

            results.push(InlineQueryResult::Article(article_result));
        }

        tracing::info!("✅ Создано {} inline результатов", results.len());
        results
    }

    fn create_no_results_result(
        &self,
        query: &str,
        language: SupportedLanguage,
    ) -> InlineQueryResult {
        let message = format_no_results_message(query, language.display_name());

        InlineQueryResult::Article(
            InlineQueryResultArticle::new(
                "no_results",
                "Ничего не найдено",
                InputMessageContent::Text(
                    InputMessageContentText::new(message).parse_mode(ParseMode::MarkdownV2),
                ),
            )
            .description("Попробуйте изменить запрос"),
        )
    }

    fn create_error_result(&self, error: &WikiError) -> InlineQueryResult {
        let message = format_error_message(&error.user_message());

        InlineQueryResult::Article(
            InlineQueryResultArticle::new(
                "error",
                "Ошибка",
                InputMessageContent::Text(
                    InputMessageContentText::new(message).parse_mode(ParseMode::MarkdownV2),
                ),
            )
            .description("Временная ошибка сервиса"),
        )
    }
}

pub async fn inline_query_handler(
    bot: Bot,
    q: InlineQuery,
    handler: Arc<InlineQueryHandler>,
) -> ResponseResult<()> {
    handler.handle(bot, q).await
}
