use std::sync::Arc;
use teloxide::{dispatching::Dispatcher, prelude::*};
use tracing::{error, info};

use wiki_article_finder_telegram::{
    create_handlers, create_services, init_logging, inline_query_handler, AppConfig,
    InlineQueryHandler, MessageHandler, WikiError,
};

fn create_dispatcher(
    bot: Bot,
    inline_handler: Arc<InlineQueryHandler>,
    message_handler: Arc<MessageHandler>,
) -> Dispatcher<Bot, teloxide::RequestError, teloxide::dispatching::DefaultKey> {
    let handler = dptree::entry()
        .branch(Update::filter_inline_query().endpoint({
            let inline_handler = Arc::clone(&inline_handler);
            move |bot: Bot, query: InlineQuery| {
                let handler = Arc::clone(&inline_handler);
                async move {
                    if let Err(e) = inline_query_handler(bot, query, handler).await {
                        error!("Error in inline query handler: {:?}", e);
                    }
                    Ok(())
                }
            }
        }))
        .branch(Update::filter_message().endpoint({
            let message_handler = Arc::clone(&message_handler);
            move |bot: Bot, msg: Message| {
                let handler = Arc::clone(&message_handler);
                async move {
                    if let Err(e) = handler.handle(bot, msg).await {
                        error!("Error in message handler: {:?}", e);
                    }
                    Ok(())
                }
            }
        }));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
}

#[tokio::main]
async fn main() -> Result<(), WikiError> {
    dotenv::dotenv().ok();

    let config = AppConfig::from_env()?;

    init_logging(&config.logging)?;

    info!(
        "Starting Wikipedia Articles Bot v{}",
        env!("CARGO_PKG_VERSION")
    );

    let (wikipedia_service, wikidata_service) = create_services(config.clone())?;
    let wikipedia_service = Arc::new(wikipedia_service);
    let wikidata_service = Arc::new(wikidata_service);

    let (inline_handler, message_handler) = create_handlers(
        Arc::clone(&wikipedia_service),
        Arc::clone(&wikidata_service),
    );
    let inline_handler = Arc::new(inline_handler);
    let message_handler = Arc::new(message_handler);

    let bot = Bot::new(&config.telegram.bot_token);

    let mut dispatcher = create_dispatcher(bot, inline_handler, message_handler);

    dispatcher.dispatch().await;

    Ok(())
}
