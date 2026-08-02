use aidoku::alloc::{String, Vec};
use aidoku::imports::html::Html;
use aidoku::prelude::*;
use aidoku::{AidokuError, Chapter, Manga, MangaPageResult, Page, Result};

use crate::error;
use crate::models::{ChapterInfo, MangaInfo};
use crate::network;
use crate::sites::natsu::parser::{
    chapters_from_document, extract_nonce, manga_info_from_detail, manga_info_from_listing,
    manga_info_from_search_results, pages_from_document, slug_from_url,
};
use crate::utils;

mod parser;

/// The Natsu website served to readers.
pub const BASE_URL: &str = "https://natsu.one";

/// Name used when tagging errors with the site that raised them.
const SITE: &str = "natsu";

const SEARCH_AJAX_PATH: &str = "/wp-admin/admin-ajax.php";
const SEARCH_AJAX_ACTION: &str = "search";
const LATEST_PATH: &str = "/latest-update/";
const LIST_PAGE_SIZE: usize = 24;

/// Searches Natsu, or lists the latest updates when no query is provided.
///
/// Keyword search runs through the site's AJAX endpoint and therefore first
/// fetches a page to obtain the current search nonce.
pub fn search(query: Option<String>, page: i32) -> Result<MangaPageResult> {
    search_inner(query, page).map_err(|error| error::with_site(SITE, error))
}

fn search_inner(query: Option<String>, page: i32) -> Result<MangaPageResult> {
    let query = query.as_deref().map(str::trim).unwrap_or("");
    let entries = if query.is_empty() {
        latest_listing(page)?
    } else {
        ajax_search(query)?
    };
    let has_next_page = if query.is_empty() {
        entries.len() == LIST_PAGE_SIZE
    } else {
        false
    };
    Ok(MangaPageResult {
        entries,
        has_next_page,
    })
}

fn latest_listing(page: i32) -> Result<Vec<Manga>> {
    let path = if page > 1 {
        format!("{LATEST_PATH}page/{page}/")
    } else {
        String::from(LATEST_PATH)
    };
    let url = format!("{BASE_URL}{path}");
    let document = network::get_html(&url)?;
    Ok(manga_info_from_listing(&document, BASE_URL)
        .into_iter()
        .map(MangaInfo::into_aidoku)
        .collect())
}

fn ajax_search(query: &str) -> Result<Vec<Manga>> {
    let nonce = fetch_search_nonce()?;
    let url = format!("{BASE_URL}{SEARCH_AJAX_PATH}?nonce={nonce}&action={SEARCH_AJAX_ACTION}");
    let body = format!("query={}", utils::form_encode(query));
    let headers = [
        ("X-Requested-With", "XMLHttpRequest"),
        ("Referer", BASE_URL),
    ];
    let html = network::post_form_string(&url, &body, &headers)?;
    let document = Html::parse(html.as_bytes())
        .map_err(|html_error| error::with_url(url, html_error.into()))?;
    Ok(manga_info_from_search_results(&document, BASE_URL)
        .into_iter()
        .map(MangaInfo::into_aidoku)
        .collect())
}

/// Fetches the home page and extracts the search nonce from the AJAX form.
fn fetch_search_nonce() -> Result<String> {
    let document = network::get_html(BASE_URL)?;
    let element = document
        .select_first("[hx-post*='admin-ajax.php']")
        .ok_or_else(|| AidokuError::message("search form not found"))?;
    let attr = element
        .attr("hx-post")
        .ok_or_else(|| AidokuError::message("search form missing hx-post"))?;
    extract_nonce(&attr).ok_or_else(|| AidokuError::message("nonce not found"))
}

/// Refreshes manga details and/or chapters from the detail page.
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    manga_update_inner(manga, needs_details, needs_chapters)
        .map_err(|error| error::with_site(SITE, error))
}

fn manga_update_inner(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    let slug = slug_from_manga(&manga)?;
    let url = format!("{BASE_URL}/manga/{slug}/");
    let document = network::get_html(&url)?;
    let mut updated = manga.clone();

    if needs_details && let Some(info) = manga_info_from_detail(&document, BASE_URL, &slug) {
        updated = info.into_aidoku();
    }

    if needs_chapters {
        updated.chapters = Some(
            chapters_from_document(&document, BASE_URL)
                .into_iter()
                .map(ChapterInfo::into_aidoku)
                .collect(),
        );
    }

    Ok(updated)
}

/// Fetches the page images for a chapter from its reader page URL.
pub fn page_list(_manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    page_list_inner(&chapter).map_err(|error| error::with_site(SITE, error))
}

fn page_list_inner(chapter: &Chapter) -> Result<Vec<Page>> {
    let url = chapter
        .url
        .as_deref()
        .ok_or_else(|| AidokuError::message("missing chapter url"))?;
    let document = network::get_html(url)?;
    Ok(pages_from_document(&document))
}

/// Extracts the manga slug from a manga key or URL.
fn slug_from_manga(manga: &Manga) -> Result<String> {
    if !manga.key.is_empty() {
        return Ok(manga.key.clone());
    }
    if let Some(url) = manga.url.as_deref()
        && let Some(slug) = slug_from_url(url)
    {
        return Ok(slug);
    }
    Err(AidokuError::message(format!(
        "missing series slug in key or url ({})",
        manga.url.as_deref().unwrap_or("no url")
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidoku_test::aidoku_test;

    #[aidoku_test]
    fn live_latest_listing() {
        let result = search(None, 1).expect("latest listing");
        assert!(!result.entries.is_empty());
        assert!(result.has_next_page);
        let first = &result.entries[0];
        assert!(!first.key.is_empty());
        assert!(!first.title.is_empty());
    }

    #[aidoku_test]
    fn live_search_by_title() {
        let result = search(Some(String::from("one piece")), 1).expect("search");
        assert!(!result.entries.is_empty());
        let first = &result.entries[0];
        assert_eq!(first.key, "one-piece");
        assert!(!first.title.is_empty());
    }

    #[aidoku_test]
    fn live_manga_update_and_pages() {
        let result = search(None, 1).expect("latest listing");
        let manga = result.entries[0].clone();
        let updated = manga_update(manga, true, true).expect("update");
        assert!(updated.description.is_some() || !updated.title.is_empty());
        let chapters = updated.chapters.as_ref().expect("chapters");
        assert!(!chapters.is_empty());
        assert!(chapters[0].url.is_some());
        let pages = page_list(updated.clone(), chapters[0].clone()).expect("pages");
        assert!(!pages.is_empty());
    }
}
