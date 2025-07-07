use once_cell::sync::Lazy;
use regex::Regex;

static HTML_TAG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<[^>]*>").expect("Failed to compile HTML tag regex"));

static MULTIPLE_SPACES_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\s+").expect("Failed to compile multiple spaces regex"));

pub fn clean_html(text: &str) -> String {
    let text = HTML_TAG_REGEX.replace_all(text, " ");
    let text = decode_html_entities(&text);
    let text = MULTIPLE_SPACES_REGEX.replace_all(&text, " ");
    text.trim().to_string()
}

pub fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
        .replace("&hellip;", "…")
}

pub fn truncate_string(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }

    let mut truncated = text.chars().take(max_chars).collect::<String>();

    if let Some(last_space) = truncated.rfind(' ') {
        truncated.truncate(last_space);
    }

    format!("{}...", truncated)
}

pub fn clean_description(text: &str) -> String {
    let cleaned = clean_html(text);

    let cleaned = cleaned
        .replace('\n', " ")
        .replace('\r', " ")
        .replace('\t', " ");

    MULTIPLE_SPACES_REGEX
        .replace_all(&cleaned, " ")
        .trim()
        .to_string()
}

pub fn extract_first_sentence(text: &str, max_length: usize) -> String {
    let cleaned = clean_description(text);

    if let Some(end_pos) = cleaned.find(|c| matches!(c, '.' | '!' | '?')) {
        let sentence = &cleaned[..=end_pos];
        if sentence.len() <= max_length {
            return sentence.trim().to_string();
        }
    }

    truncate_string(&cleaned, max_length)
}

pub fn normalize_whitespace(text: &str) -> String {
    MULTIPLE_SPACES_REGEX
        .replace_all(text.trim(), " ")
        .to_string()
}

pub fn sanitize_search_query(query: &str) -> String {
    query
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || "-_".contains(*c))
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn is_empty_or_whitespace(text: &str) -> bool {
    text.trim().is_empty()
}

pub fn capitalize_first_letter(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_html() {
        assert_eq!(clean_html("<p>Hello <b>world</b>!</p>"), "Hello world!");
        assert_eq!(clean_html("Plain text"), "Plain text");
        assert_eq!(
            clean_html("<span>Multiple   spaces</span>"),
            "Multiple spaces"
        );
    }

    #[test]
    fn test_decode_html_entities() {
        assert_eq!(decode_html_entities("Rock &amp; Roll"), "Rock & Roll");
        assert_eq!(decode_html_entities("&lt;tag&gt;"), "<tag>");
        assert_eq!(decode_html_entities("&quot;quoted&quot;"), "\"quoted\"");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("this is a long text", 10), "this is a...");
        assert_eq!(truncate_string("exactly_ten", 11), "exactly_ten");
    }

    #[test]
    fn test_extract_first_sentence() {
        assert_eq!(
            extract_first_sentence("First sentence. Second sentence.", 50),
            "First sentence."
        );
        assert_eq!(
            extract_first_sentence("No sentence end", 20),
            "No sentence end"
        );
        assert_eq!(
            extract_first_sentence("Very long first sentence that exceeds limit.", 20),
            "Very long first..."
        );
    }

    #[test]
    fn test_sanitize_search_query() {
        assert_eq!(sanitize_search_query("normal query"), "normal query");
        assert_eq!(
            sanitize_search_query("query with @#$% symbols"),
            "query with  symbols"
        );
        assert_eq!(sanitize_search_query("  spaced  query  "), "spaced query");
    }

    #[test]
    fn test_capitalize_first_letter() {
        assert_eq!(capitalize_first_letter("hello"), "Hello");
        assert_eq!(capitalize_first_letter("HELLO"), "HELLO");
        assert_eq!(capitalize_first_letter(""), "");
    }
}
