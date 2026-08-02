use aidoku::alloc::string::ToString;
use aidoku::alloc::{String, Vec};
use aidoku::helpers::uri::QueryParameters;
use aidoku::imports::net::{HttpMethod, Request};
use aidoku::prelude::*;
use aidoku::{AidokuError, Chapter, Manga, MangaPageResult, Page, Result};

use crate::error;
use crate::models::{ChapterInfo, MangaInfo};
use crate::network;
use crate::sites::shinigami::parser::{
    ChapterDetailResponse, ChapterListResponse, DetailResponse, MangaListResponse,
    chapter_info_from_item, manga_info_from_item, pages_from_detail,
};

mod parser;

/// The public website served to readers.
pub const BASE_URL: &str = "https://shinigami.asia";

/// Shinigami's JSON API, discovered from the Izanami web bundle.
pub const API_BASE_URL: &str = "https://api.shngm.io/v1";

/// Name used when tagging errors with the site that raised them.
const SITE: &str = "shinigami";

const LIST_PAGE_SIZE: i32 = 24;
const CHAPTER_PAGE_SIZE: i32 = 500;

/// Searches the Shinigami API by title, or lists releases when no query is
/// provided. `sort_index` selects the order: 0 = "Terbaru" (latest updates),
/// anything else = "Populer" (the API's popularity ranking).
pub fn search(query: Option<String>, page: i32, sort_index: usize) -> Result<MangaPageResult> {
    search_inner(query, page, sort_index).map_err(|error| error::with_site(SITE, error))
}

fn search_inner(query: Option<String>, page: i32, sort_index: usize) -> Result<MangaPageResult> {
    let mut query_params = QueryParameters::new();
    let page_string = page.to_string();
    let page_size = LIST_PAGE_SIZE.to_string();
    query_params.push("page", Some(&page_string));
    query_params.push("page_size", Some(&page_size));
    match query.as_deref().map(str::trim) {
        Some(query) if !query.is_empty() => {
            query_params.push("q", Some(query));
        }
        _ => {
            if sort_index == 0 {
                query_params.push("is_update", Some("true"));
                query_params.push("sort", Some("latest"));
                query_params.push("sort_order", Some("desc"));
            } else {
                query_params.push("sort", Some("popularity"));
                query_params.push("sort_order", Some("desc"));
            }
        }
    }
    let url = format!("{API_BASE_URL}/manga/list?{query_params}");
    let response: MangaListResponse = network::get_json(&url)?;

    let entries = response
        .data
        .iter()
        .filter_map(|item| manga_info_from_item(item, BASE_URL))
        .map(MangaInfo::into_aidoku)
        .collect::<Vec<_>>();
    let has_next_page = response
        .meta
        .total_page
        .map_or(entries.len() == LIST_PAGE_SIZE as usize, |total_page| {
            page < total_page
        });
    Ok(MangaPageResult {
        entries,
        has_next_page,
    })
}

/// Refreshes manga details and/or chapters from the Shinigami API.
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    manga_update_inner(manga, needs_details, needs_chapters)
        .map_err(|error| error::with_site(SITE, error))
}

fn manga_update_inner(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    let manga_id = manga_id_from_manga(&manga)?;
    let mut updated = manga.clone();

    if needs_details {
        let url = format!("{API_BASE_URL}/manga/detail/{manga_id}");
        let response: DetailResponse = network::get_json(&url)?;
        updated = manga_info_from_item(&response.data, BASE_URL)
            .ok_or_else(|| AidokuError::message("invalid manga payload"))?
            .into_aidoku();
    }

    if needs_chapters {
        updated.chapters = Some(fetch_chapters(&manga_id)?);
    }

    Ok(updated)
}

/// Fetches every chapter of a manga, paging through the chapter list.
fn fetch_chapters(manga_id: &str) -> Result<Vec<Chapter>> {
    let mut chapters = Vec::new();
    let mut page = 1;
    loop {
        let page_string = page.to_string();
        let page_size = CHAPTER_PAGE_SIZE.to_string();
        let mut query_params = QueryParameters::new();
        query_params.push("page", Some(&page_string));
        query_params.push("page_size", Some(&page_size));
        query_params.push("sort_by", Some("chapter_number"));
        query_params.push("sort_order", Some("desc"));
        let url = format!("{API_BASE_URL}/chapter/{manga_id}/list?{query_params}");
        let response: ChapterListResponse = network::get_json(&url)?;
        let total_page = response.meta.total_page.unwrap_or(1);
        chapters.extend(
            response
                .data
                .iter()
                .filter_map(|item| chapter_info_from_item(item, BASE_URL))
                .map(ChapterInfo::into_aidoku),
        );
        if page >= total_page {
            break;
        }
        page += 1;
    }
    Ok(chapters)
}

/// Fetches the page images for a chapter from the Shinigami API.
pub fn page_list(_manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    page_list_inner(&chapter).map_err(|error| error::with_site(SITE, error))
}

fn page_list_inner(chapter: &Chapter) -> Result<Vec<Page>> {
    let chapter_id = chapter_id_from_chapter(chapter)?;
    let url = format!("{API_BASE_URL}/chapter/detail/{chapter_id}");
    let response: ChapterDetailResponse = network::get_json(&url)?;
    Ok(pages_from_detail(&response.data))
}

/// Builds the request used to download chapter/cover images. Shinigami's
/// image CDN serves requests without headers, so no customization is needed.
pub fn image_request(url: String) -> Result<Request> {
    Ok(Request::new(url, HttpMethod::Get)?)
}

/// Extracts the manga id from a manga key or URL.
fn manga_id_from_manga(manga: &Manga) -> Result<String> {
    if !manga.key.is_empty() {
        return Ok(manga.key.clone());
    }
    if let Some(url) = manga.url.as_deref()
        && let Some(id) = manga_id_from_url(url)
    {
        return Ok(id);
    }
    Err(AidokuError::message(format!(
        "missing manga id in key or url ({})",
        manga.url.as_deref().unwrap_or("no url")
    )))
}

/// Extracts the chapter id from a chapter key or URL.
fn chapter_id_from_chapter(chapter: &Chapter) -> Result<String> {
    if !chapter.key.is_empty() {
        return Ok(chapter.key.clone());
    }
    if let Some(url) = chapter.url.as_deref()
        && let Some(id) = chapter_id_from_url(url)
    {
        return Ok(id);
    }
    Err(AidokuError::message(format!(
        "missing chapter id in key or url ({})",
        chapter.url.as_deref().unwrap_or("no url")
    )))
}

/// Extracts the id from a manga URL such as
/// `https://shinigami.asia/series/<manga_id>/`.
pub fn manga_id_from_url(url: &str) -> Option<String> {
    let marker = "/series/";
    let start = url.find(marker)? + marker.len();
    let rest = &url[start..];
    let end = rest.find('/').unwrap_or(rest.len());
    let id = rest[..end].trim();
    if id.is_empty() {
        None
    } else {
        Some(String::from(id))
    }
}

/// Extracts the id from a chapter URL such as
/// `https://shinigami.asia/chapter/<chapter_id>`.
pub fn chapter_id_from_url(url: &str) -> Option<String> {
    let marker = "/chapter/";
    let start = url.find(marker)? + marker.len();
    let rest = &url[start..];
    let end = rest.find('/').unwrap_or(rest.len());
    let id = rest[..end].trim();
    if id.is_empty() {
        None
    } else {
        Some(String::from(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidoku_test::aidoku_test;

    #[aidoku_test]
    fn live_latest_listing() {
        let result = search(None, 1, 0).expect("latest listing");
        assert!(!result.entries.is_empty());
        assert!(result.has_next_page);
        let first = &result.entries[0];
        assert!(!first.key.is_empty());
        assert!(!first.title.is_empty());
    }

    #[aidoku_test]
    fn live_popular_listing() {
        let result = search(None, 1, 1).expect("popular listing");
        assert!(!result.entries.is_empty());
        assert!(result.has_next_page);
        let first = &result.entries[0];
        assert!(!first.key.is_empty());
        assert!(!first.title.is_empty());
    }

    #[aidoku_test]
    fn live_search_by_title() {
        let result = search(Some(String::from("one piece")), 1, 0).expect("search");
        assert!(!result.entries.is_empty());
        let first = &result.entries[0];
        assert!(!first.key.is_empty());
        assert!(first.title.to_lowercase().contains("one piece"));
    }

    #[aidoku_test]
    fn live_manga_update_and_pages() {
        let result = search(None, 1, 0).expect("latest listing");
        let manga = result.entries[0].clone();
        let updated = manga_update(manga, true, true).expect("update");
        assert!(!updated.title.is_empty());
        let chapters = updated.chapters.as_ref().expect("chapters");
        assert!(!chapters.is_empty());
        assert!(chapters[0].url.is_some());
        let pages = page_list(updated.clone(), chapters[0].clone()).expect("pages");
        assert!(!pages.is_empty());
    }

    #[aidoku_test]
    fn id_extracted_from_urls() {
        assert_eq!(
            manga_id_from_url("https://shinigami.asia/series/c0f1d049-ff7f-474d-8c6a-3a55e4c44147"),
            Some(String::from("c0f1d049-ff7f-474d-8c6a-3a55e4c44147"))
        );
        assert_eq!(manga_id_from_url("https://shinigami.asia/"), None);
        assert_eq!(
            chapter_id_from_url(
                "https://shinigami.asia/chapter/825d0326-3dfe-47c8-8468-153725735068"
            ),
            Some(String::from("825d0326-3dfe-47c8-8468-153725735068"))
        );
        assert_eq!(chapter_id_from_url("https://shinigami.asia/"), None);
    }
}
