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

pub fn escape_markdown_url(url: &str) -> String {
    url.chars()
        .map(|c| match c {
            ')' | '\\' => format!("\\{}", c),
            _ => c.to_string(),
        })
        .collect()
}

pub fn bold(text: &str) -> String {
    format!("*{}*", escape_markdown(text))
}

pub fn italic(text: &str) -> String {
    format!("_{}_", escape_markdown(text))
}

pub fn code(text: &str) -> String {
    format!("`{}`", text.replace('`', "\\`"))
}

pub fn link(text: &str, url: &str) -> String {
    format!("[{}]({})", escape_markdown(text), escape_markdown_url(url))
}

pub fn heading(text: &str, level: u8) -> String {
    let prefix = "#".repeat(level.min(6) as usize);
    format!("{} {}", prefix, escape_markdown(text))
}

pub fn list_item(text: &str) -> String {
    format!("• {}", escape_markdown(text))
}

pub fn quote(text: &str) -> String {
    text.lines()
        .map(|line| format!("> {}", escape_markdown(line)))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn separator() -> &'static str {
    "────────────────"
}

pub fn emoji_header(emoji: &str, text: &str) -> String {
    format!("{} *{}*", emoji, escape_markdown(text))
}

pub fn format_article_description(title: &str, description: &str, url: &str) -> String {
    format!(
        "📖 *{}*\n\n{}\n\n🔗 [Читать полностью]({})",
        escape_markdown(title),
        escape_markdown(description),
        escape_markdown_url(url)
    )
}

pub fn format_error_message(error: &str) -> String {
    format!("⚠️ *Ошибка*\n\n{}", escape_markdown(error))
}

pub fn format_no_results_message(query: &str, language: &str) -> String {
    format!(
        "🔍 *Ничего не найдено*\n\nПо запросу \"{}\" ничего не найдено в {} Википедии\n\n💡 Попробуйте изменить запрос",
        escape_markdown(query),
        escape_markdown(language)
    )
}

pub fn format_welcome_message() -> String {
    r#"🌍 *Добро пожаловать в Wikipedia Search Bot\!*

📚 Я помогу вам быстро найти информацию в **любой** Википедии мира\! Поддерживается более 100 языков\. Просто используйте инлайн\-поиск в любом чате или беседе\!

🔍 **Как использовать:**
Наберите `@WikipediaArticlesBot ваш запрос` в любом чате

🌏 **Поддерживаемые языки:**
• `запрос` или `ru:запрос` — 🇷🇺 русская Википедия
• `en:query` — 🇺🇸 English Wikipedia
• `de:suche` — 🇩🇪 Deutsche Wikipedia
• `fr:recherche` — 🇫🇷 Wikipédia français
• `es:búsqueda` — 🇪🇸 Wikipedia español
• `uk:запит` — 🇺🇦 українська Вікіпедія
• `ja:検索` — 🇯🇵 ウィキペディア
• `zh:搜索` — 🇨🇳 维基百科
• И многие другие\!

💡 **Примеры поиска:**
• `Пушкин` — биография поэта \(русская\)
• `en:Albert Einstein` — English biography
• `de:Berlin` — deutsche Artikel
• `fr:Paris` — article français
• `ja:東京` — 日本語の記事
• `es:Madrid` — artículo español

✨ **Возможности:**
📖 Полные статьи с описаниями
🖼️ Превью изображений из статей
🔗 Прямые ссылки на Wikipedia
⚡ Быстрый поиск по всей базе знаний
🌐 Поддержка 100\+ языков мира

🚀 *Начните вводить запрос или выберите язык\!*"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_markdown() {
        assert_eq!(escape_markdown("Hello_world"), "Hello\\_world");
        assert_eq!(escape_markdown("Test*bold*"), "Test\\*bold\\*");
        assert_eq!(escape_markdown("Link[text]"), "Link\\[text\\]");
    }

    #[test]
    fn test_escape_markdown_url() {
        assert_eq!(
            escape_markdown_url("https://example.com"),
            "https://example.com"
        );
        assert_eq!(
            escape_markdown_url("https://example.com)"),
            "https://example.com\\)"
        );
    }

    #[test]
    fn test_bold() {
        assert_eq!(bold("test"), "*test*");
        assert_eq!(bold("special_chars"), "*special\\_chars*");
    }

    #[test]
    fn test_link() {
        assert_eq!(
            link("Google", "https://google.com"),
            "[Google](https://google.com)"
        );
        assert_eq!(
            link("Text_with_underscores", "https://example.com)"),
            "[Text\\_with\\_underscores](https://example.com\\))"
        );
    }

    #[test]
    fn test_format_article_description() {
        let result =
            format_article_description("Test Article", "Test description", "https://example.com");
        assert!(result.contains("📖 *Test Article*"));
        assert!(result.contains("Test description"));
        assert!(result.contains("🔗 [Читать полностью](https://example.com)"));
    }
}
