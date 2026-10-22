use aidoku::alloc::string::ToString;
use aidoku::alloc::{String, Vec};
use aidoku::imports::html::{Document, Element};
use aidoku::{Page, PageContent, Viewer};

use crate::models::{ChapterInfo, MangaInfo, Status};
use crate::parser as html;
use crate::utils;

pub const MANGA_LINK_SELECTOR: &str = "a[href*='/komik/']";
const KOMIK_MARKER: &str = "/komik/";

/// Extracts the manga slug from a manga page URL such as
/// `https://komikindo.ch/komik/846048-eleceed/`.
pub fn slug_from_url(url: &str) -> Option<String> {
    let start = url.find(KOMIK_MARKER)? + KOMIK_MARKER.len();
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
/// `https://komikindo.ch/eleceed-chapter-423/`.
pub fn chapter_key_from_url(url: &str) -> Option<String> {
    let clean = url.trim_end_matches('/');
    let key = clean.rsplit('/').next()?.trim();
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

    if let Some(posts) = document.select(".animepost") {
        for post in posts {
            if let Some(info) = manga_info_from_card(&post, base_url) {
                if !seen_keys.contains(&info.key) {
                    seen_keys.push(info.key.clone());
                    entries.push(info);
                }
            }
        }
    }
    entries
}

fn manga_info_from_card(post: &Element, base_url: &str) -> Option<MangaInfo> {
    let link = post.select_first(MANGA_LINK_SELECTOR)?;
    let href = html::attr(&link, "href")?;
    let url = html::resolve_url(&href, base_url)?;
    let key = slug_from_url(&url)?;

    let title = post
        .select_first(".animposx .tt h4")
        .or_else(|| post.select_first(".tt h4"))
        .and_then(|h| html::text(&h))
        .or_else(|| html::attr(&link, "title"))
        .unwrap_or_else(|| key.clone());

    let cover = post
        .select_first("img")
        .and_then(|img| {
            html::attr(&img, "src")
                .or_else(|| html::attr(&img, "data-src"))
                .or_else(|| html::attr(&img, "data-lazy-src"))
        })
        .map(|src| utils::remove_size_suffix(&src));

    let viewer = post
        .select_first(".typeflag")
        .and_then(|tf| html::text(&tf))
        .map(|t| viewer_from_str(&t))
        .unwrap_or(Viewer::RightToLeft);

    Some(MangaInfo {
        key,
        title,
        cover,
        url: Some(url),
        status: Status::Unknown,
        viewer,
        ..MangaInfo::default()
    })
}

/// Updates manga details from the detail page.
pub fn manga_info_from_detail(document: &Document, base_url: &str) -> Option<MangaInfo> {
    let title = document
        .select_first(".infoanime h1.entry-title")
        .or_else(|| document.select_first("h1.entry-title"))
        .or_else(|| document.select_first("h1"))
        .and_then(|el| html::text(&el))?;

    let cover = document
        .select_first(".thumb img")
        .and_then(|img| {
            html::attr(&img, "src")
                .or_else(|| html::attr(&img, "data-src"))
                .or_else(|| html::attr(&img, "data-lazy-src"))
        })
        .map(|src| utils::remove_size_suffix(&src));

    let description = document
        .select_first(".entry-content.entry-content-single")
        .or_else(|| document.select_first("#sinopsis p"))
        .and_then(|el| html::text(&el));

    let mut authors = Vec::new();
    let mut artists = Vec::new();
    let mut status = Status::Unknown;
    let mut viewer = Viewer::RightToLeft;

    if let Some(specs) = document.select(".spe span") {
        for spec in specs {
            if let Some(text) = html::text(&spec) {
                let lower = text.to_lowercase();
                if lower.contains("pengarang") || lower.contains("author") {
                    if let Some((_, val)) = text.split_once(':') {
                        authors.push(utils::trim(val));
                    }
                } else if lower.contains("ilustrator") || lower.contains("artist") {
                    if let Some((_, val)) = text.split_once(':') {
                        artists.push(utils::trim(val));
                    }
                } else if lower.contains("status") {
                    status = status_from_str(Some(&text));
                } else if lower.contains("jenis komik") {
                    viewer = viewer_from_str(&text);
                }
            }
        }
    }

    let mut tags = Vec::new();
    if let Some(genres) = document.select(".genre-info a") {
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

/// Parses chapters from the detail page.
pub fn chapters_from_document(document: &Document, base_url: &str) -> Vec<ChapterInfo> {
    let mut chapters = Vec::new();
    if let Some(items) = document.select("#chapter_list li") {
        for item in items {
            if let Some(link) = item.select_first(".lchx a")
                && let Some(href) = html::attr(&link, "href")
                && let Some(url) = html::resolve_url(&href, base_url)
                && let Some(key) = chapter_key_from_url(&url)
            {
                let title = html::text(&link);
                let chapter_number = parse_chapter_number(&key);
                let date_uploaded = item
                    .select_first(".dt a")
                    .and_then(|dt| html::text(&dt))
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
    if let Some(images) = document.select("#chimg-auh img, #readerarea img") {
        for image in images {
            if let Some(src) = html::attr(&image, "src")
                .or_else(|| html::attr(&image, "data-src"))
                .or_else(|| html::attr(&image, "data-lazy-src"))
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
    let digits = key
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect::<String>();
    digits.parse::<f32>().ok()
}

fn status_from_str(status: Option<&str>) -> Status {
    match status.map(|s| s.to_lowercase()).as_deref() {
        Some(s) if s.contains("berjalan") || s.contains("ongoing") => Status::Ongoing,
        Some(s) if s.contains("tamat") || s.contains("completed") => Status::Completed,
        Some(s) if s.contains("hiatus") => Status::Hiatus,
        _ => Status::Unknown,
    }
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
    fn slug_from_komik_url() {
        assert_eq!(
            slug_from_url("https://komikindo.ch/komik/846048-eleceed/"),
            Some(String::from("846048-eleceed"))
        );
    }

    #[aidoku_test]
    fn chapter_key_from_url_test() {
        assert_eq!(
            chapter_key_from_url("https://komikindo.ch/eleceed-chapter-423/"),
            Some(String::from("eleceed-chapter-423"))
        );
    }

    #[aidoku_test]
    fn pages_from_chimg() {
        let doc = Html::parse(
            r#"<div id="chimg-auh">
                <img src="https://cdn.example/p1.webp" />
                <img src="https://cdn.example/p2.webp" />
            </div>"#,
        )
        .expect("html");
        let pages = pages_from_document(&doc);
        assert_eq!(pages.len(), 2);
    }
}
