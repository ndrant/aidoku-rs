#![no_std]
// Single-site packaging builds only compile their site; silence dead-code
// for the helpers used exclusively by the intentionally excluded modules.
#![cfg_attr(not(all(feature = "komikcast", feature = "natsu")), allow(dead_code))]

mod error;
mod models;
mod network;
mod parser;
mod source;
mod utils;

mod sites;

use aidoku::alloc::{String, Vec};
use aidoku::{Chapter, FilterValue, Manga, MangaPageResult, Page, Result, Source, prelude::*};

pub struct ComicSource;

impl Source for ComicSource {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        source::search(query, page, filters)
    }

    fn get_manga_update(
        &self,
        manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        source::manga_update(manga, needs_details, needs_chapters)
    }

    fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        source::page_list(manga, chapter)
    }
}

register_source!(ComicSource);
