use aidoku::alloc::string::ToString;
use aidoku::alloc::{String, Vec};
use aidoku::imports::html::{Document, Element};
use aidoku::{Page, PageContent, Viewer};

use crate::models::{ChapterInfo, MangaInfo, Status};
use crate::parser as html;
use crate::utils;

pub const MANGA_LINK_SELECTOR: &str = "a[href*='/manga/']";
const MANGA_MARKER: &str = "/manga/";
const CHAPTER_MARKER: &str = "/chapter-";

/// Extracts the manga slug from a manga page URL such as
/// `https://www.westmanga.shop/manga/solo-max-level-newbie`.
pub fn slug_from_url(url: &str) -> Option<String> {
    let start = url.find(MANGA_MARKER)? + MANGA_MARKER.len();
    let rest = &url[start..];
    let end = rest.find('/').unwrap_or(rest.len());
    let slug = rest[..end].trim();
    if slug.is_empty() {
        None
    } else {
        Some(String::from(slug))
    }
}

/// Extracts chapter slug/key from chapter URL such as
/// `https://www.westmanga.shop/read/solo-max-level-newbie/chapter-280`.
pub fn chapter_key_from_url(url: &str) -> Option<String> {
    let start = url.find(CHAPTER_MARKER)? + CHAPTER_MARKER.len();
    let rest = &url[start..];
    let end = rest.find('/').unwrap_or(rest.len());
    let key = rest[..end].trim();
    if key.is_empty() {
        None
    } else {
        Some(String::from(key))
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
                    .and_then(|img| html::attr(&img, "src"))
                    .map(|src| utils::remove_size_suffix(&src));

                let title = link
                    .select_first("img")
                    .and_then(|img| html::attr(&img, "alt"))
                    .or_else(|| html::attr(&link, "title"))
                    .or_else(|| {
                        link.select_first("h1, h2, h3, h4, span")
                            .and_then(|el| html::text(&el))
                    })
                    .unwrap_or_else(|| key.clone());

                let viewer = viewer_from_card(&link);

                entries.push(MangaInfo {
                    key,
                    title,
                    cover,
                    url: Some(url),
                    status: Status::Unknown,
                    viewer,
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
        .select_first("img[src*='/mangas/'][src*='/cover.']")
        .or_else(|| document.select_first("img[src*='cover.']"))
        .or_else(|| document.select_first(".thumb img"))
        .and_then(|img| html::attr(&img, "src"))
        .map(|src| utils::remove_size_suffix(&src));

    let description = document
        .select_first("p[class*='text-sm'][class*='leading-relaxed']")
        .or_else(|| document.select_first("p.description"))
        .or_else(|| document.select_first(".entry-content p"))
        .and_then(|el| html::text(&el));

    let mut tags = Vec::new();
    if let Some(genres) = document.select("a[href*='/genres/'], a[href*='/genre/']") {
        for g in genres {
            if let Some(name) = html::text(&g) {
                tags.push(name);
            }
        }
    }

    let viewer = viewer_from_document(document);

    Some(MangaInfo {
        title,
        cover,
        description,
        authors: None,
        artists: None,
        tags: if tags.is_empty() { None } else { Some(tags) },
        status: Status::Ongoing,
        viewer,
        url: Some(base_url.to_string()),
        ..MangaInfo::default()
    })
}

/// Parses chapters from the detail page.
pub fn chapters_from_document(document: &Document, base_url: &str) -> Vec<ChapterInfo> {
    let mut chapters = Vec::new();
    let mut seen_keys = Vec::new();

    if let Some(links) = document.select("a[href*='/chapter-']") {
        for link in links {
            if let Some(href) = html::attr(&link, "href")
                && let Some(url) = html::resolve_url(&href, base_url)
                && let Some(key) = chapter_key_from_url(&url)
            {
                if seen_keys.contains(&key) {
                    continue;
                }
                seen_keys.push(key.clone());

                let title = html::text(&link);
                let chapter_number = key.parse::<f32>().ok();
                let date_uploaded = link
                    .select_first("span:last-child")
                    .and_then(|span| html::text(&span))
                    .and_then(|text| utils::relative_time_seconds(&text));

                chapters.push(ChapterInfo {
                    key,
                    title,
                    chapter_number,
                    date_uploaded,
                    url: Some(url),
                });
            }
        }
    }
    chapters
}

/// Extracts page images from the reader document.
pub fn pages_from_document(document: &Document) -> Vec<Page> {
    let mut pages = Vec::new();
    if let Some(images) = document.select("div[class*='overflow-hidden bg-black'] img, #readerarea img, img[src*='proxy.westmanga.shop']") {
        for image in images {
            if let Some(src) = html::attr(&image, "src") {
                let trimmed = src.trim();
                // Avoid cover thumbnail or icon
                if !trimmed.is_empty()
                    && !trimmed.contains("cover.")
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

fn viewer_from_card(link: &Element) -> Viewer {
    if let Some(badge) = link.select_first("div[class*='uppercase']")
        && let Some(text) = html::text(&badge)
    {
        return viewer_from_str(&text);
    }
    Viewer::RightToLeft
}

fn viewer_from_document(doc: &Document) -> Viewer {
    if let Some(badge) = doc.select_first("span[class*='uppercase'], div[class*='uppercase']")
        && let Some(text) = html::text(&badge)
    {
        return viewer_from_str(&text);
    }
    Viewer::RightToLeft
}

fn viewer_from_str(text: &str) -> Viewer {
    let lower = text.to_lowercase();
    if lower.contains("manhwa") || lower.contains("webtoon") || lower.contains("manhua") {
        Viewer::Webtoon
    } else {
        Viewer::RightToLeft
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidoku::imports::html::Html;
    use aidoku_test::aidoku_test;

    #[aidoku_test]
    fn slug_from_westmanga_url() {
        assert_eq!(
            slug_from_url("https://www.westmanga.shop/manga/solo-max-level-newbie"),
            Some(String::from("solo-max-level-newbie"))
        );
    }

    #[aidoku_test]
    fn chapter_key_from_westmanga_url() {
        assert_eq!(
            chapter_key_from_url(
                "https://www.westmanga.shop/read/solo-max-level-newbie/chapter-280"
            ),
            Some(String::from("280"))
        );
    }

    #[aidoku_test]
    fn pages_from_westmanga_markup() {
        let doc = Html::parse(
            r#"<div class="overflow-hidden bg-black">
                <img src="https://proxy.westmanga.shop/api/img-proxy?url=1.webp" />
                <img src="https://proxy.westmanga.shop/api/img-proxy?url=2.webp" />
            </div>"#,
        )
        .expect("html");
        let pages = pages_from_document(&doc);
        assert_eq!(pages.len(), 2);
    }
}
