use aidoku::alloc::string::ToString;
use aidoku::alloc::{String, Vec};
use aidoku::imports::html::{Document, Element};
use aidoku::{Page, PageContent, Viewer};

use crate::models::{ChapterInfo, MangaInfo, Status};
use crate::parser as html;
use crate::utils;

pub const MANGA_LINK_SELECTOR: &str = "a[href*='/manga/']";
pub const FORMAT_BADGE_SELECTOR: &str = "img[src*='/static/svg/']";
const CHAPTER_MARKER: &str = "/chapter-";

/// Extracts the manga slug from a manga page URL such as
/// `https://v7.kiryuu.to/manga/martial-peak/`.
pub fn slug_from_url(url: &str) -> Option<String> {
    let marker = "/manga/";
    let start = url.find(marker)? + marker.len();
    let rest = &url[start..];
    let end = rest.find('/').unwrap_or(rest.len());
    let slug = rest[..end].trim();
    if slug.is_empty() {
        None
    } else {
        Some(String::from(slug))
    }
}

/// Extracts the chapter identifier from a chapter URL such as
/// `https://v7.kiryuu.to/manga/martial-peak/chapter-3855.686109/`.
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

/// Parses the manga grid on listing or search result pages.
pub fn manga_info_from_listing(document: &Document, base_url: &str) -> Vec<MangaInfo> {
    let mut entries = Vec::new();
    let mut seen_keys = Vec::new();

    // Select candidate cards: either search-results div or elements containing manga links
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

                // Title from link child h1, title attr, or img alt
                let title = link
                    .select_first("h1")
                    .and_then(|h| html::text(&h))
                    .or_else(|| html::attr(&link, "title"))
                    .or_else(|| {
                        link.select_first("img")
                            .and_then(|img| html::attr(&img, "alt"))
                    })
                    .unwrap_or_else(|| key.clone());

                let cover = link
                    .select_first("img")
                    .and_then(|img| {
                        html::attr(&img, "src")
                            .or_else(|| html::attr(&img, "data-src"))
                            .or_else(|| html::attr(&img, "srcset"))
                    })
                    .map(|src| {
                        let first = src.split_whitespace().next().unwrap_or(&src);
                        utils::remove_size_suffix(first)
                    });

                let viewer = viewer_from_badge(&link);

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

/// Updates manga details and chapter list from the manga detail page.
pub fn manga_info_from_detail(document: &Document, base_url: &str) -> Option<MangaInfo> {
    let title = document
        .select_first("h1[itemprop='name']")
        .or_else(|| document.select_first(".entry-title"))
        .or_else(|| document.select_first("h1"))
        .and_then(|el| html::text(&el))?;

    let cover = document
        .select_first(".thumb img")
        .or_else(|| document.select_first("img.wp-post-image"))
        .and_then(|img| html::attr(&img, "src").or_else(|| html::attr(&img, "data-src")))
        .map(|src| utils::remove_size_suffix(&src));

    let description = document
        .select_first("div[itemprop='description']")
        .or_else(|| document.select_first(".entry-content"))
        .and_then(|el| html::text(&el));

    let mut authors = Vec::new();
    let mut artists = Vec::new();
    let mut status = Status::Unknown;

    if let Some(meta_spans) = document.select(".spe span") {
        for span in meta_spans {
            if let Some(text) = html::text(&span) {
                let lower = text.to_lowercase();
                if lower.contains("author") || lower.contains("pengarang") {
                    if let Some((_, val)) = text.split_once(':') {
                        authors.push(utils::trim(val));
                    }
                } else if lower.contains("artist") || lower.contains("ilustrator") {
                    if let Some((_, val)) = text.split_once(':') {
                        artists.push(utils::trim(val));
                    }
                } else if lower.contains("status") {
                    status = status_from_str(Some(&text));
                }
            }
        }
    }

    if status == Status::Unknown {
        if let Some(p) = document.select_first("p.font-normal.text-xs") {
            status = status_from_str(html::text(&p).as_deref());
        }
    }

    let mut tags = Vec::new();
    if let Some(genres) = document.select(".mgen a, .seriestugenre a") {
        for g in genres {
            if let Some(name) = html::text(&g) {
                tags.push(name);
            }
        }
    }

    let viewer = viewer_from_badge_document(document);

    Some(MangaInfo {
        title,
        cover,
        description,
        authors: if authors.is_empty() {
            None
        } else {
            Some(authors)
        },
        artists: if artists.is_empty() {
            None
        } else {
            Some(artists)
        },
        tags: if tags.is_empty() { None } else { Some(tags) },
        status,
        viewer,
        url: Some(base_url.to_string()),
        ..MangaInfo::default()
    })
}

/// Parses the chapter list from the manga detail page.
pub fn chapters_from_document(document: &Document, base_url: &str) -> Vec<ChapterInfo> {
    let mut chapters = Vec::new();
    if let Some(links) = document.select("a[href*='/chapter-']") {
        for link in links {
            if let Some(href) = html::attr(&link, "href")
                && let Some(url) = html::resolve_url(&href, base_url)
                && let Some(key) = chapter_key_from_url(&url)
            {
                let title = html::text(&link);
                let chapter_number = parse_chapter_number(&key);
                chapters.push(ChapterInfo {
                    key,
                    title,
                    chapter_number,
                    date_uploaded: None,
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
    if let Some(images) = document.select("div[data-image-data] img, #readerarea img") {
        for image in images {
            if let Some(src) = html::attr(&image, "src").or_else(|| html::attr(&image, "data-src"))
            {
                let trimmed = src.trim();
                if !trimmed.is_empty() && !trimmed.ends_with(".gif") && !trimmed.ends_with(".svg") {
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

fn parse_chapter_number(key: &str) -> Option<f32> {
    let num_part = key.split('.').next()?;
    num_part.parse::<f32>().ok()
}

fn status_from_str(status: Option<&str>) -> Status {
    match status.map(|s| s.to_lowercase()).as_deref() {
        Some(s) if s.contains("ongoing") || s.contains("berjalan") => Status::Ongoing,
        Some(s) if s.contains("completed") || s.contains("tamat") => Status::Completed,
        Some(s) if s.contains("hiatus") => Status::Hiatus,
        Some(s) if s.contains("cancelled") || s.contains("drop") => Status::Cancelled,
        _ => Status::Unknown,
    }
}

fn viewer_from_badge(element: &Element) -> Viewer {
    if let Some(badge) = element.select_first(FORMAT_BADGE_SELECTOR)
        && let Some(src) = html::attr(&badge, "src")
    {
        return viewer_from_str(&src);
    }
    Viewer::RightToLeft
}

fn viewer_from_badge_document(document: &Document) -> Viewer {
    if let Some(badge) = document.select_first(FORMAT_BADGE_SELECTOR)
        && let Some(src) = html::attr(&badge, "src")
    {
        return viewer_from_str(&src);
    }
    Viewer::RightToLeft
}

fn viewer_from_str(src: &str) -> Viewer {
    let src = src.to_lowercase();
    if src.contains("manhwa") || src.contains("webtoon") {
        Viewer::Webtoon
    } else if src.contains("manhua") {
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
    fn slug_extracted_from_url() {
        assert_eq!(
            slug_from_url("https://v7.kiryuu.to/manga/martial-peak/"),
            Some(String::from("martial-peak"))
        );
    }

    #[aidoku_test]
    fn chapter_key_extracted_from_url() {
        assert_eq!(
            chapter_key_from_url("https://v7.kiryuu.to/manga/martial-peak/chapter-3855.686109/"),
            Some(String::from("3855.686109"))
        );
    }

    #[aidoku_test]
    fn pages_from_markup() {
        let doc = Html::parse(
            r#"<div id="readerarea">
                <img src="https://yuucdn.com/1.jpg" />
                <img src="https://yuucdn.com/2.jpg" />
            </div>"#,
        )
        .expect("html");
        let pages = pages_from_document(&doc);
        assert_eq!(pages.len(), 2);
    }
}
