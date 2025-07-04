use futures::future::{join_all, BoxFuture};
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
    clean_description, escape_markdown, escape_markdown_url, get_article_details, get_article_url,
    get_wikidata_description_by_pageid, search_wikipedia, truncate_string,
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
            let article_data: Vec<_> = articles.into_iter().collect();
            let detail_futures: Vec<
                BoxFuture<
                    '_,
                    Result<
                        (Option<String>, Option<String>),
                        Box<dyn std::error::Error + Send + Sync>,
                    >,
                >,
            > = article_data
                .iter()
                .map(|article| {
                    if let Some(pageid) = article.pageid {
                        Box::pin(get_article_details(pageid)) as BoxFuture<'_, _>
                    } else {
                        Box::pin(async { Ok((Option::<String>::None, Option::<String>::None)) })
                            as BoxFuture<'_, _>
                    }
                })
                .collect();

            let wikidata_futures: Vec<
                BoxFuture<'_, Result<Option<String>, Box<dyn std::error::Error + Send + Sync>>>,
            > = article_data
                .iter()
                .map(|article| {
                    if let Some(pageid) = article.pageid {
                        Box::pin(get_wikidata_description_by_pageid(pageid)) as BoxFuture<'_, _>
                    } else {
                        Box::pin(async { Ok(Option::<String>::None) }) as BoxFuture<'_, _>
                    }
                })
                .collect();

            let (details, wikidata_descriptions): (Vec<_>, Vec<_>) =
                tokio::join!(join_all(detail_futures), join_all(wikidata_futures));

            let mut results = Vec::new();
            for (idx, ((article, details_result), wikidata_description_result)) in article_data
                .into_iter()
                .zip(details)
                .zip(wikidata_descriptions)
                .enumerate()
            {
                let article_url = get_article_url(&article.title);
                let (image_url, extract) = match details_result {
                    Ok(pair) => pair,
                    Err(_) => (Option::<String>::None, Option::<String>::None),
                };
                let wikidata_description = match wikidata_description_result {
                    Ok(desc) => desc,
                    Err(_) => None,
                };
                let description = if let Some(wikidata_desc) = wikidata_description {
                    clean_description(&truncate_string(&wikidata_desc, 100))
                } else if let Some(extract_text) = &extract {
                    clean_description(&truncate_string(extract_text, 100))
                } else {
                    clean_description(&truncate_string(&article.snippet, 100))
                };

                let content_text = if let Some(ref extract_text) = extract {
                    truncate_string(extract_text, 300)
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
                    format!("article_{}", idx),
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

            if results.is_empty() {
                let no_results = vec![InlineQueryResult::Article(
                    InlineQueryResultArticle::new(
                        "no_results",
                        "Ничего не найдено",
                        InputMessageContent::Text(InputMessageContentText::new(format!(
                            "По запросу \"{}\" ничего не найдено в русской Википедии",
                            query
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
