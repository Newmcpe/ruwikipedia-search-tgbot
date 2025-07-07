use std::sync::Arc;
use teloxide::{prelude::*, types::ParseMode};
use tracing::{error, info, instrument};

use crate::utils::format_welcome_message;

pub struct MessageHandler;

impl MessageHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(&self, bot: Bot, msg: Message) -> ResponseResult<()> {
        let Some(text) = msg.text() else {
            return Ok(());
        };

        match text {
            "/start" => self.handle_start_command(bot, &msg).await,
            "/help" => self.handle_help_command(bot, &msg).await,
            _ => self.handle_unknown_command(bot, &msg).await,
        }
    }

    async fn handle_start_command(&self, bot: Bot, msg: &Message) -> ResponseResult<()> {
        let welcome_text = format_welcome_message();

        bot.send_message(msg.chat.id, welcome_text)
            .parse_mode(ParseMode::MarkdownV2)
            .await
            .map_err(|e| {
                error!("Failed to send welcome message: {:?}", e);
                e
            })?;

        Ok(())
    }

    async fn handle_help_command(&self, bot: Bot, msg: &Message) -> ResponseResult<()> {
        let help_text = self.create_help_message();

        bot.send_message(msg.chat.id, help_text)
            .parse_mode(ParseMode::MarkdownV2)
            .await
            .map_err(|e| {
                error!("Failed to send help message: {:?}", e);
                e
            })?;

        Ok(())
    }

    async fn handle_unknown_command(&self, _bot: Bot, _msg: &Message) -> ResponseResult<()> {
        Ok(())
    }

    fn create_help_message(&self) -> String {
        r#"📖 *Справка по Wikipedia Search Bot*

🔍 **Основные возможности:**
• Поиск статей во всех языковых версиях Wikipedia
• Inline\-поиск прямо в чатах и беседах
• Автоматическое получение изображений и описаний
• Поддержка 100\+ языков мира

💡 **Как использовать inline\-поиск:**
1\. Наберите в любом чате: `@WikipediaArticlesBot`
2\. Добавьте ваш поисковый запрос
3\. Выберите статью из результатов

🌍 **Примеры запросов:**
• `Пушкин` — поиск в русской Wikipedia
• `en:Albert Einstein` — поиск в английской
• `de:Berlin` — поиск в немецкой
• `fr:Paris` — поиск во французской
• `ja:東京` — поиск в японской

⚙️ **Поддерживаемые команды:**
/start — показать приветствие
/help — показать эту справку

🚀 **Начните использовать бота прямо сейчас\!**"#
            .to_string()
    }
}

impl Default for MessageHandler {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn message_handler(
    bot: Bot,
    msg: Message,
    handler: Arc<MessageHandler>,
) -> ResponseResult<()> {
    handler.handle(bot, msg).await
}
