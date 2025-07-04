// Неиспользуемые импорты удалены - теперь используем батчевые запросы вместо множественных futures
use teloxide::{
    prelude::*,
    types::{
        InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
        ParseMode,
    },
};
use url::Url;

mod wikipedia;
use wikipedia::{
    clean_description, escape_markdown, escape_markdown_url, get_article_url,
    get_articles_batch_info, get_wikidata_descriptions_batch, search_wikipedia, truncate_string,
};

async fn inline_query_handler(bot: Bot, q: InlineQuery) -> ResponseResult<()> {
    let query = q.query.trim();

    if query.is_empty() {
        let results = vec![InlineQueryResult::Article(
            InlineQueryResultArticle::new(
                "help",
                "🔍 Поиск в Википедии",
                InputMessageContent::Text(InputMessageContentText::new(
                    "Введите запрос для поиска статей в русской Википедии",
                )),
            )
            .description("Начните вводить запрос для поиска"),
        )];

        bot.answer_inline_query(q.id, results).await?;
        return Ok(());
    }

    match search_wikipedia(query).await {
        Ok(articles) => {
            // Собираем PageID для батчевого запроса
            let pageids: Vec<u64> = articles
                .iter()
                .filter_map(|article| article.pageid)
                .collect();

            if pageids.is_empty() {
                let no_results = vec![InlineQueryResult::Article(
                    InlineQueryResultArticle::new(
                        "no_results",
                        "Ничего не найдено",
                        InputMessageContent::Text(InputMessageContentText::new(format!(
                            "По запросу \"{query}\" ничего не найдено в русской Википедии"
                        ))),
                    )
                    .description("Попробуйте изменить запрос"),
                )];
                bot.answer_inline_query(q.id, no_results).await?;
                return Ok(());
            }

            // Получаем все данные одним батчевым запросом
            let batch_info = match get_articles_batch_info(pageids).await {
                Ok(info) => info,
                Err(_) => {
                    let error_result = vec![InlineQueryResult::Article(
                        InlineQueryResultArticle::new(
                            "error",
                            "Ошибка получения данных",
                            InputMessageContent::Text(InputMessageContentText::new(
                                "Произошла ошибка при получении данных статей. Попробуйте позже.",
                            )),
                        )
                        .description("Временная ошибка сервиса"),
                    )];
                    bot.answer_inline_query(q.id, error_result).await?;
                    return Ok(());
                }
            };

            // Собираем Wikidata ID для получения описаний
            let wikidata_ids: Vec<String> = batch_info
                .values()
                .filter_map(|info| info.wikidata_id.clone())
                .collect();

            // Получаем описания из Wikidata батчем
            let wikidata_descriptions = get_wikidata_descriptions_batch(wikidata_ids)
                .await
                .unwrap_or_else(|_| std::collections::HashMap::new());

            let mut results = Vec::new();
            for (idx, article) in articles.into_iter().enumerate() {
                if let Some(pageid) = article.pageid {
                    let article_url = get_article_url(&article.title);
                    let info = batch_info.get(&pageid);

                    // Получаем изображение
                    let image_url = info.and_then(|i| i.image_url.clone());

                    // Получаем описание (приоритет: Wikidata -> extract -> snippet)
                    let description = if let Some(info) = info {
                        if let Some(wikidata_id) = &info.wikidata_id {
                            if let Some(wikidata_desc) = wikidata_descriptions.get(wikidata_id) {
                                clean_description(&truncate_string(wikidata_desc, 100))
                            } else if let Some(extract) = &info.extract {
                                clean_description(&truncate_string(extract, 100))
                            } else {
                                clean_description(&truncate_string(&article.snippet, 100))
                            }
                        } else if let Some(extract) = &info.extract {
                            clean_description(&truncate_string(extract, 100))
                        } else {
                            clean_description(&truncate_string(&article.snippet, 100))
                        }
                    } else {
                        clean_description(&truncate_string(&article.snippet, 100))
                    };

                    // Получаем текст контента для сообщения
                    let content_text = if let Some(info) = info {
                        if let Some(extract) = &info.extract {
                            truncate_string(extract, 300)
                        } else {
                            truncate_string(&article.snippet, 200)
                        }
                    } else {
                        truncate_string(&article.snippet, 200)
                    };

                    let message_text = format!(
                        "📖 *{}*\n\n{}\n\n🔗 [Читать полностью]({})",
                        escape_markdown(&article.title),
                        escape_markdown(&content_text),
                        escape_markdown_url(&article_url)
                    );

                    let mut article_result = InlineQueryResultArticle::new(
                        format!("article_{idx}"),
                        &article.title,
                        InputMessageContent::Text(
                            InputMessageContentText::new(message_text)
                                .parse_mode(ParseMode::MarkdownV2),
                        ),
                    )
                    .description(description);

                    if let Some(img_url) = image_url {
                        if let Ok(parsed_img_url) = Url::parse(&img_url) {
                            article_result = article_result.thumb_url(parsed_img_url);
                        }
                    }

                    results.push(InlineQueryResult::Article(article_result));
                }
            }

            if results.is_empty() {
                let no_results = vec![InlineQueryResult::Article(
                    InlineQueryResultArticle::new(
                        "no_results",
                        "Ничего не найдено",
                        InputMessageContent::Text(InputMessageContentText::new(format!(
                            "По запросу \"{query}\" ничего не найдено в русской Википедии"
                        ))),
                    )
                    .description("Попробуйте изменить запрос"),
                )];
                bot.answer_inline_query(q.id, no_results).await?;
            } else {
                bot.answer_inline_query(q.id, results).await?;
            }
        }
        Err(_) => {
            let error_result = vec![InlineQueryResult::Article(
                InlineQueryResultArticle::new(
                    "error",
                    "Ошибка поиска",
                    InputMessageContent::Text(InputMessageContentText::new(
                        "Произошла ошибка при поиске. Попробуйте позже.",
                    )),
                )
                .description("Временная ошибка сервиса"),
            )];
            bot.answer_inline_query(q.id, error_result).await?;
        }
    }

    Ok(())
}

async fn message_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    let Some(text) = msg.text() else {
        return Ok(());
    };

    if text == "/start" {
        bot.send_message(
            msg.chat.id,
            "🌍 *Добро пожаловать в Wikipedia Search Bot\\!*\n\n\
             📚 Я помогу вам быстро найти информацию в русской Википедии\\. \
             Просто используйте инлайн\\-поиск в любом чате или беседе\\!\n\n\
             🔍 **Как использовать:**\n\
             Наберите `@WikipediaArticlesBot ваш запрос` в любом чате\n\n\
             💡 **Примеры поиска:**\n\
             • `@WikipediaArticlesBot Пушкин` — биография поэта\n\
             • `@WikipediaArticlesBot квантовая физика` — научные статьи\n\
             • `@WikipediaArticlesBot Москва` — информация о городе\n\
             • `@WikipediaArticlesBot космос` — статьи о космосе\n\n\
             ✨ **Возможности:**\n\
             📖 Полные статьи с описаниями\n\
             🖼️ Превью изображений из статей\n\
             🔗 Прямые ссылки на Wikipedia\n\
             ⚡ Быстрый поиск по всей базе знаний\n\n\
             🚀 *Начните вводить запрос прямо сейчас\\!*",
        )
        .parse_mode(ParseMode::MarkdownV2)
        .await?;
        return Ok(());
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Load .env file if it exists (for development)
    dotenv::dotenv().ok();

    // Set log level to info
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_inline_query().endpoint(inline_query_handler))
        .branch(Update::filter_message().endpoint(message_handler));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
