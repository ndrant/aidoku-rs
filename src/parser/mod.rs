use aidoku::alloc::String;
use aidoku::imports::html::Element;
use aidoku::prelude::*;

use crate::utils;

/// Returns the normalized text of an element, or `None` when it is empty.
pub fn text(element: &Element) -> Option<String> {
    element.text().and_then(|value| {
        let value = utils::normalize_whitespace(&value);
        if value.is_empty() { None } else { Some(value) }
    })
}

/// Returns a non-empty attribute value, or `None` when missing or empty.
pub fn attr(element: &Element, key: &str) -> Option<String> {
    element.attr(key).and_then(|value| {
        let value = utils::trim(&value);
        if value.is_empty() { None } else { Some(value) }
    })
}

/// Resolves a possibly relative URL against a base URL.
///
/// When `href` is already absolute it is returned unchanged.
pub fn resolve_url(href: &str, base: &str) -> Option<String> {
    if href.starts_with("http://") || href.starts_with("https://") {
        return Some(String::from(href));
    }
    let href = href.trim_start_matches('/');
    if base.ends_with('/') {
        Some(format!("{base}{href}"))
    } else {
        Some(format!("{base}/{href}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidoku_test::aidoku_test;

    #[aidoku_test]
    fn absolute_url_unchanged() {
        assert_eq!(
            resolve_url("https://a.example/x", "https://b.example"),
            Some(String::from("https://a.example/x"))
        );
    }

    #[aidoku_test]
    fn relative_url_joined() {
        assert_eq!(
            resolve_url("/manga/one-piece/", "https://natsu.one"),
            Some(String::from("https://natsu.one/manga/one-piece/"))
        );
    }

    #[aidoku_test]
    fn relative_url_joined_with_slash() {
        assert_eq!(
            resolve_url("manga/x", "https://natsu.one/"),
            Some(String::from("https://natsu.one/manga/x"))
        );
    }
}
