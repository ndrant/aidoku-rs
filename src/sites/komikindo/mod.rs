use aidoku::alloc::string::ToString;
use aidoku::alloc::{String, Vec, format};
use aidoku::imports::net::{HttpMethod, Request};
use aidoku::{AidokuError, Chapter, Manga, MangaPageResult, Page, Result};

use crate::error;
use crate::models::{ChapterInfo, MangaInfo};
use crate::network;
use crate::sites::komikindo::parser::{
    chapters_from_document, manga_info_from_detail, manga_info_from_listing, pages_from_document,
    slug_from_url,
};
use crate::utils;

mod parser;

pub const BASE_URL: &str = "https://komikindo.ch";
const SITE: &str = "komikindo";
const LIST_PAGE_SIZE: usize = 20;

/// Searches Komikindo or lists releases. `sort_index`: 0 = "Terbaru", 1 = "Populer".
pub fn search(query: Option<String>, page: i32, sort_index: usize) -> Result<MangaPageResult> {
    search_inner(query, page, sort_index).map_err(|error| error::with_site(SITE, error))
}

fn search_inner(query: Option<String>, page: i32, sort_index: usize) -> Result<MangaPageResult> {
    let query = query.as_deref().map(str::trim).unwrap_or("");
    let url = if !query.is_empty() {
        let encoded = utils::form_encode(query);
        if page <= 1 {
            format!("{BASE_URL}/?s={encoded}")
        } else {
            format!("{BASE_URL}/page/{page}/?s={encoded}")
        }
    } else {
        let order = if sort_index == 0 { "latest" } else { "popular" };
        if page <= 1 {
            format!("{BASE_URL}/manga/?order={order}")
        } else {
            format!("{BASE_URL}/manga/page/{page}/?order={order}")
        }
    };

    let document = network::get_html(&url)?;
    let entries = manga_info_from_listing(&document, BASE_URL)
        .into_iter()
        .map(MangaInfo::into_aidoku)
        .collect::<Vec<_>>();
    let has_next_page = entries.len() >= LIST_PAGE_SIZE;
    Ok(MangaPageResult {
        entries,
        has_next_page,
    })
}

/// Refreshes manga details and chapters.
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    manga_update_inner(manga, needs_details, needs_chapters)
        .map_err(|error| error::with_site(SITE, error))
}

fn manga_update_inner(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    let slug = slug_from_manga(&manga)?;
    let url = manga
        .url
        .as_deref()
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("{BASE_URL}/komik/{slug}/"));
    let mut updated = manga.clone();

    if needs_details || needs_chapters {
        let document = network::get_html(&url)?;
        if needs_details {
            if let Some(info) = manga_info_from_detail(&document, &url) {
                updated = info.into_aidoku();
                updated.key = slug.clone();
                updated.url = Some(url.clone());
            }
        }
        if needs_chapters {
            let chapters = chapters_from_document(&document, BASE_URL)
                .into_iter()
                .map(ChapterInfo::into_aidoku)
                .collect::<Vec<_>>();
            updated.chapters = Some(chapters);
        }
    }
    Ok(updated)
}

/// Fetches the page list for a chapter.
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

/// Builds an image request for Komikindo.
pub fn image_request(url: String) -> Result<Request> {
    let mut request = Request::new(url, HttpMethod::Get)?;
    let ref_header = format!("{BASE_URL}/");
    request = request.header("Referer", &ref_header);
    Ok(request)
}

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
        let result = search(None, 1, 0).expect("latest listing");
        assert!(!result.entries.is_empty());
        let first = &result.entries[0];
        assert!(!first.key.is_empty());
        assert!(!first.title.is_empty());
    }

    #[aidoku_test]
    fn live_search_by_title() {
        let result = search(Some(String::from("eleceed")), 1, 0).expect("search");
        assert!(!result.entries.is_empty());
    }
}
