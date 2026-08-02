use aidoku::alloc::string::ToString;
use aidoku::alloc::{String, Vec};
use aidoku::helpers::uri::QueryParameters;
use aidoku::imports::net::{HttpMethod, Request};
use aidoku::prelude::*;
use aidoku::{AidokuError, Chapter, Manga, MangaPageResult, Page, Result};

use crate::error;
use crate::models::{ChapterInfo, MangaInfo};
use crate::network;
use crate::sites::komikcast::parser::{
    ChapterListResponse, DetailResponse, ReaderResponse, SeriesListResponse,
    chapter_info_from_item, manga_info_from_item, pages_from_reader,
};

mod parser;

/// The public website served by KomikCast v3.
pub const BASE_URL: &str = "https://v3.komikcast.fit";

/// KomikCast's JSON API, discovered from the v3 web bundle.
pub const API_BASE_URL: &str = "https://be.komikcast.cc";

/// Name used when tagging errors with the site that raised them.
const SITE: &str = "komikcast";

const LATEST_PRESET: &str = "rilisan_terbaru";
const LIST_TAKE: i32 = 30;

/// Searches the KomikCast API by title, or lists the latest releases when no
/// query is provided.
pub fn search(query: Option<String>, page: i32) -> Result<MangaPageResult> {
    search_inner(query, page).map_err(|error| error::with_site(SITE, error))
}

fn search_inner(query: Option<String>, page: i32) -> Result<MangaPageResult> {
    let mut query_params = QueryParameters::new();
    match query.as_deref().map(str::trim) {
        Some(query) if !query.is_empty() => {
            query_params.push("title", Some(query));
        }
        _ => {
            query_params.push("preset", Some(LATEST_PRESET));
        }
    }
    let take = LIST_TAKE.to_string();
    let page_string = page.to_string();
    query_params.push("take", Some(&take));
    query_params.push("page", Some(&page_string));

    let url = format!("{API_BASE_URL}/series?{query_params}");
    let response: SeriesListResponse = network::get_json(&url)?;

    let entries = response
        .data
        .iter()
        .filter_map(|item| manga_info_from_item(item, BASE_URL))
        .map(MangaInfo::into_aidoku)
        .collect::<Vec<_>>();
    let has_next_page = response.meta.as_ref().map_or_else(
        || entries.len() == LIST_TAKE as usize,
        |meta| page < meta.last_page,
    );
    Ok(MangaPageResult {
        entries,
        has_next_page,
    })
}

/// Refreshes manga details and/or chapters from the KomikCast API.
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    manga_update_inner(manga, needs_details, needs_chapters)
        .map_err(|error| error::with_site(SITE, error))
}

fn manga_update_inner(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    let slug = slug_from_manga(&manga)?;
    let mut updated = manga.clone();

    if needs_details {
        let url = format!("{API_BASE_URL}/series/{slug}");
        let response: DetailResponse = network::get_json(&url)?;
        let info = manga_info_from_item(&response.data, BASE_URL)
            .ok_or_else(|| AidokuError::message("invalid series payload"))?;
        updated = info.into_aidoku();
    }

    if needs_chapters {
        let url = format!("{API_BASE_URL}/series/{slug}/chapters");
        let response: ChapterListResponse = network::get_json(&url)?;
        updated.chapters = Some(
            response
                .data
                .iter()
                .filter_map(|item| chapter_info_from_item(item, &slug, BASE_URL))
                .map(ChapterInfo::into_aidoku)
                .collect::<Vec<_>>(),
        );
    }

    Ok(updated)
}

/// Fetches the page images for a chapter from the KomikCast API.
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    page_list_inner(&manga, &chapter).map_err(|error| error::with_site(SITE, error))
}

fn page_list_inner(manga: &Manga, chapter: &Chapter) -> Result<Vec<Page>> {
    let slug = slug_from_manga(manga)?;
    if chapter.key.is_empty() {
        return Err(AidokuError::message("missing chapter key"));
    }
    let url = format!("{API_BASE_URL}/series/{slug}/chapters/{}", chapter.key);
    let response: ReaderResponse = network::get_json(&url)?;
    Ok(pages_from_reader(&response))
}

/// Builds the request used to download chapter/cover images, which are
/// hotlink-protected and reject requests without a site Referer.
pub fn image_request(url: String) -> Result<Request> {
    Ok(Request::new(url, HttpMethod::Get)?.header("Referer", &format!("{BASE_URL}/")))
}

/// Extracts the series slug from a manga key or URL.
fn slug_from_manga(manga: &Manga) -> Result<String> {
    if !manga.key.is_empty() {
        return Ok(manga.key.clone());
    }
    let prefix = format!("{BASE_URL}/series/");
    if let Some(url) = manga.url.as_deref() {
        if let Some(slug) = url.strip_prefix(&prefix).filter(|slug| !slug.is_empty()) {
            return Ok(String::from(slug));
        }
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
    fn live_latest_releases() {
        let result = search(None, 1).expect("latest listing");
        assert_eq!(result.entries.len(), LIST_TAKE as usize);
        assert!(result.has_next_page);
    }

    #[aidoku_test]
    fn live_search_by_title() {
        let result = search(Some(String::from("human")), 1).expect("search");
        assert!(!result.entries.is_empty());
        let first = &result.entries[0];
        assert!(!first.key.is_empty());
        assert!(!first.title.is_empty());
        assert!(
            first
                .url
                .as_deref()
                .is_some_and(|url| url.contains("/series/"))
        );
    }

    #[aidoku_test]
    fn live_manga_update_and_pages() {
        let result = search(Some(String::from("human table")), 1).expect("search");
        let manga = result.entries[0].clone();
        let updated = manga_update(manga, true, true).expect("update");
        assert!(updated.description.is_some());
        let chapters = updated.chapters.as_ref().expect("chapters");
        assert!(!chapters.is_empty());
        let pages = page_list(updated.clone(), chapters[0].clone()).expect("pages");
        assert!(!pages.is_empty());
    }
}
