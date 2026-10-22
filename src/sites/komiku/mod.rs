use aidoku::alloc::{String, Vec, format};
use aidoku::imports::net::{HttpMethod, Request};
use aidoku::{AidokuError, Chapter, Manga, MangaPageResult, Page, Result};

use crate::error;
use crate::models::{ChapterInfo, MangaInfo};
use crate::network;
use crate::sites::komiku::parser::{
    KomikuChapter, KomikuChapterDetail, KomikuComic, KomikuListResponse, chapters_from_items,
    manga_info_from_comic, pages_from_detail,
};
use crate::utils;

mod parser;

pub const BASE_URL: &str = "https://01.komiku.asia";
pub const API_BASE_URL: &str = "https://01.komiku.asia/api/v2";
const SITE: &str = "komiku";

/// Searches Komiku or lists releases. `sort_index`: 0 = "Terbaru", 1 = "Populer".
pub fn search(query: Option<String>, page: i32, sort_index: usize) -> Result<MangaPageResult> {
    search_inner(query, page, sort_index).map_err(|error| error::with_site(SITE, error))
}

fn search_inner(query: Option<String>, page: i32, sort_index: usize) -> Result<MangaPageResult> {
    let query = query.as_deref().map(str::trim).unwrap_or("");
    if !query.is_empty() {
        let encoded = utils::form_encode(query);
        let url = format!("{API_BASE_URL}/comics/search?q={encoded}");
        let comics: Vec<KomikuComic> = network::get_json(&url)?;
        let entries = comics
            .iter()
            .map(|c| manga_info_from_comic(c, BASE_URL))
            .map(MangaInfo::into_aidoku)
            .collect();
        return Ok(MangaPageResult {
            entries,
            has_next_page: false,
        });
    }

    let sort = if sort_index == 0 { "latest" } else { "popular" };
    let url = format!("{API_BASE_URL}/comics?sort={sort}&page={page}");
    let response: KomikuListResponse = network::get_json(&url)?;
    let entries = response
        .items
        .iter()
        .map(|c| manga_info_from_comic(c, BASE_URL))
        .map(MangaInfo::into_aidoku)
        .collect::<Vec<_>>();
    let has_next_page = page < response.total_pages;

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
    let mut updated = manga.clone();

    let detail_url = format!("{API_BASE_URL}/comics/{slug}");
    let comic: KomikuComic = network::get_json(&detail_url)?;

    if needs_details {
        let info = manga_info_from_comic(&comic, BASE_URL);
        updated = info.into_aidoku();
    }

    if needs_chapters {
        let chapters_url = format!("{API_BASE_URL}/comics/{}/chapters", comic.id);
        let items: Vec<KomikuChapter> = network::get_json(&chapters_url)?;
        let chapters = chapters_from_items(&items, comic.id, &slug, BASE_URL)
            .into_iter()
            .map(ChapterInfo::into_aidoku)
            .collect::<Vec<_>>();
        updated.chapters = Some(chapters);
    }

    Ok(updated)
}

/// Fetches the page list for a chapter.
pub fn page_list(_manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    page_list_inner(&chapter).map_err(|error| error::with_site(SITE, error))
}

fn page_list_inner(chapter: &Chapter) -> Result<Vec<Page>> {
    let key = &chapter.key;
    let (comic_id, chapter_id) = key
        .split_once(':')
        .ok_or_else(|| AidokuError::message("invalid komiku chapter key"))?;
    let url = format!("{API_BASE_URL}/comics/{comic_id}/chapters/id/{chapter_id}");
    let detail: KomikuChapterDetail = network::get_json(&url)?;
    Ok(pages_from_detail(&detail))
}

/// Builds an image request for Komiku.
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
    if let Some(url) = manga.url.as_deref() {
        let clean = url.trim_end_matches('/');
        if let Some(slug) = clean.rsplit('/').next() {
            if !slug.is_empty() {
                return Ok(String::from(slug));
            }
        }
    }
    Err(AidokuError::message("missing series slug"))
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
        let result = search(Some(String::from("solo")), 1, 0).expect("search");
        assert!(!result.entries.is_empty());
    }
}
