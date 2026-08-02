#![no_std]
// Single-site packaging builds only compile their site; silence dead-code
// for the helpers used exclusively by the intentionally excluded modules.
#![cfg_attr(
    not(all(feature = "komikcast", feature = "natsu", feature = "shinigami")),
    allow(dead_code)
)]

mod error;
mod models;
mod network;
mod parser;
mod source;
mod utils;

mod sites;

use aidoku::alloc::{String, Vec};
use aidoku::imports::net::Request;
use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Manga, MangaPageResult, Page, PageContext, Result,
    Source, prelude::*,
};

pub struct ComicSource;

impl Source for ComicSource {
    fn new() -> Self {
        println!("[aidoku-rs] source initialized");
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

impl ImageRequestProvider for ComicSource {
    fn get_image_request(&self, url: String, context: Option<PageContext>) -> Result<Request> {
        source::image_request(url, context)
    }
}

register_source!(ComicSource, ImageRequestProvider);
