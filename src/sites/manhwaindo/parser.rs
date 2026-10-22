use aidoku::alloc::string::ToString;
use aidoku::alloc::{String, Vec, format};
use aidoku::imports::html::Document;
use aidoku::serde::Deserialize;
use aidoku::{Page, PageContent, Viewer};

use crate::models::{ChapterInfo, MangaInfo, Status};
use crate::parser as html;
use crate::utils;

pub const MANGA_LINK_SELECTOR: &str = "a[href*='/series/']";
const SERIES_MARKER: &str = "/series/";

#[derive(Deserialize, Debug)]
pub struct AgcChapterItem {
    pub id: i64,
    pub url: String,
    pub chapter: String,
    pub time: Option<String>,
}

/// Extracts the manga slug from a manga page URL such as
/// `https://www.manhwaindo.my/series/eleceed/`.
pub fn slug_from_url(url: &str) -> Option<String> {
    let start = url.find(SERIES_MARKER)? + SERIES_MARKER.len();
    let rest = &url[start..];
    let end = rest.find('/').unwrap_or(rest.len());
    let slug = rest[..end].trim();
    if slug.is_empty() {
        None
    } else {
        Some(String::from(slug))
    }
}

/// Extracts seriesId from the raw HTML document string.
pub fn series_id_from_html(html_str: &str) -> Option<String> {
    let marker = "seriesId:";
    let pos = html_str.find(marker)?;
    let rest = &html_str[pos + marker.len()..];
    let digits = rest
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        None
    } else {
        Some(digits)
    }
}

/// Parses the manga cards on listing or search result pages.
pub fn manga_info_from_listing(document: &Document, base_url: &str) -> Vec<MangaInfo> {
    let mut entries = Vec::new();
    let mut seen_keys = Vec::new();

    if let Some(links) = document.select(MANGA_LINK_SELECTOR) {
        for link in links {
            if let Some(href) = html::attr(&link, "href")
                && let Some(url) = html::resolve_url(&href, base_url)
                && let Some(key) = slug_from_url(&url)
            {
                if seen_keys.contains(&key) {
                    continue;
                }
                seen_keys.push(key.clone());

                let cover = link
                    .select_first("img")
                    .and_then(|img| {
                        html::attr(&img, "src").or_else(|| html::attr(&img, "data-src"))
                    })
                    .map(|src| utils::remove_size_suffix(&src));

                let title = link
                    .select_first("img")
                    .and_then(|img| html::attr(&img, "alt").or_else(|| html::attr(&img, "title")))
                    .or_else(|| html::attr(&link, "title"))
                    .unwrap_or_else(|| key.clone());

                entries.push(MangaInfo {
                    key,
                    title,
                    cover,
                    url: Some(url),
                    status: Status::Ongoing,
                    viewer: Viewer::Webtoon,
                    ..MangaInfo::default()
                });
            }
        }
    }
    entries
}

/// Updates manga details from the detail page.
pub fn manga_info_from_detail(document: &Document, base_url: &str) -> Option<MangaInfo> {
    let title = document
        .select_first("h1")
        .or_else(|| document.select_first(".entry-title"))
        .and_then(|el| html::text(&el))?;

    let cover = document
        .select_first("img[src*='/manga-images/']")
        .or_else(|| document.select_first(".thumb img"))
        .and_then(|img| html::attr(&img, "src"))
        .map(|src| utils::remove_size_suffix(&src));

    let description = document
        .select_first(".entry-content p")
        .or_else(|| document.select_first("div[itemprop='description']"))
        .and_then(|el| html::text(&el));

    let mut tags = Vec::new();
    if let Some(genres) = document.select("a[href*='/genre/']") {
        for g in genres {
            if let Some(name) = html::text(&g) {
                tags.push(name);
            }
        }
    }

    Some(MangaInfo {
        title,
        cover,
        description,
        authors: None,
        artists: None,
        tags: if tags.is_empty() { None } else { Some(tags) },
        status: Status::Ongoing,
        viewer: Viewer::Webtoon,
        url: Some(base_url.to_string()),
        ..MangaInfo::default()
    })
}

/// Converts AGC API chapter items into [ChapterInfo] records.
pub fn chapters_from_agc(items: &[AgcChapterItem]) -> Vec<ChapterInfo> {
    items
        .iter()
        .map(|item| {
            let key = item.id.to_string();
            let chapter_number = item.chapter.parse::<f32>().ok();
            let date_uploaded = item.time.as_deref().and_then(utils::relative_time_seconds);
            ChapterInfo {
                key,
                title: Some(format!("Chapter {}", item.chapter)),
                chapter_number,
                date_uploaded,
                url: Some(item.url.clone()),
            }
        })
        .collect()
}

/// Extracts page images from the reader document.
pub fn pages_from_document(document: &Document) -> Vec<Page> {
    let mut pages = Vec::new();
    if let Some(images) = document.select("div#top-chapter ~ p img, p img, #readerarea img") {
        for image in images {
            if let Some(src) = html::attr(&image, "src") {
                let trimmed = src.trim();
                if !trimmed.is_empty()
                    && !trimmed.contains("thumbnail.")
                    && !trimmed.ends_with(".gif")
                    && !trimmed.ends_with(".svg")
                {
                    pages.push(Page {
                        content: PageContent::url(trimmed),
                        ..Page::default()
                    });
                }
            }
        }
    }
    pages
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidoku_test::aidoku_test;

    #[aidoku_test]
    fn slug_from_series_url() {
        assert_eq!(
            slug_from_url("https://www.manhwaindo.my/series/eleceed/"),
            Some(String::from("eleceed"))
        );
    }

    #[aidoku_test]
    fn series_id_extracted_from_script() {
        let snippet = "window.agcChapterListConfig = { seriesId: 13, total: 424 };";
        assert_eq!(series_id_from_html(snippet), Some(String::from("13")));
    }
}
