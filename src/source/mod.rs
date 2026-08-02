use aidoku::alloc::{String, Vec};
use aidoku::imports::net::Request;
use aidoku::{Chapter, FilterValue, Manga, MangaPageResult, Page, PageContext, Result};

#[cfg(feature = "komikcast")]
use crate::sites::komikcast;
#[cfg(feature = "natsu")]
use crate::sites::natsu;

#[cfg(all(feature = "natsu", feature = "komikcast"))]
pub const SITE_KOMIKCAST: &str = "komikcast";
#[cfg(all(feature = "natsu", feature = "komikcast"))]
pub const SITE_NATSU: &str = "natsu";

#[cfg(all(feature = "natsu", feature = "komikcast"))]
pub const SITE_FILTER_ID: &str = "site";
#[cfg(all(feature = "natsu", feature = "komikcast"))]
pub const SITE_FILTER_VALUE_NATSU: &str = "natsu";

/// Dispatches a search to the Natsu-only build.
#[cfg(all(feature = "natsu", not(feature = "komikcast")))]
pub fn search(
    query: Option<String>,
    page: i32,
    _filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    natsu::search(query, page)
}

/// Dispatches a search to the KomikCast-only build.
#[cfg(all(feature = "komikcast", not(feature = "natsu")))]
pub fn search(
    query: Option<String>,
    page: i32,
    _filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    komikcast::search(query, page)
}

/// Dispatches a search to the site selected through filters, defaulting to
/// KomikCast when no selection is present (multi-site build).
#[cfg(all(feature = "natsu", feature = "komikcast"))]
pub fn search(
    query: Option<String>,
    page: i32,
    filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    if selected_site(&filters) == SITE_NATSU {
        natsu::search(query, page)
    } else {
        komikcast::search(query, page)
    }
}

/// Refreshes manga details from the Natsu-only build.
#[cfg(all(feature = "natsu", not(feature = "komikcast")))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    natsu::manga_update(manga, needs_details, needs_chapters)
}

/// Refreshes manga details from the KomikCast-only build.
#[cfg(all(feature = "komikcast", not(feature = "natsu")))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    komikcast::manga_update(manga, needs_details, needs_chapters)
}

/// Refreshes manga details from the site that owns the manga URL
/// (multi-site build).
#[cfg(all(feature = "natsu", feature = "komikcast"))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    if site_from_url(manga.url.as_deref()) == SITE_NATSU {
        natsu::manga_update(manga, needs_details, needs_chapters)
    } else {
        komikcast::manga_update(manga, needs_details, needs_chapters)
    }
}

/// Fetches page images from the Natsu-only build.
#[cfg(all(feature = "natsu", not(feature = "komikcast")))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    natsu::page_list(manga, chapter)
}

/// Builds the request used to download an image in the Natsu-only build.
#[cfg(all(feature = "natsu", not(feature = "komikcast")))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    natsu::image_request(url)
}

/// Builds the request used to download an image in the KomikCast-only build.
#[cfg(all(feature = "komikcast", not(feature = "natsu")))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    komikcast::image_request(url)
}

/// Builds the request used to download an image for whichever site owns it
/// (multi-site build).
#[cfg(all(feature = "natsu", feature = "komikcast"))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    if url.contains("natsu") {
        natsu::image_request(url)
    } else {
        komikcast::image_request(url)
    }
}

/// Fetches page images from the KomikCast-only build.
#[cfg(all(feature = "komikcast", not(feature = "natsu")))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    komikcast::page_list(manga, chapter)
}

/// Fetches page images from the site that owns the manga URL (multi-site build).
#[cfg(all(feature = "natsu", feature = "komikcast"))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    if site_from_url(manga.url.as_deref()) == SITE_NATSU {
        natsu::page_list(manga, chapter)
    } else {
        komikcast::page_list(manga, chapter)
    }
}

#[cfg(all(feature = "natsu", feature = "komikcast"))]
fn selected_site(filters: &[FilterValue]) -> &'static str {
    for filter in filters {
        if let FilterValue::Select { id, value } = filter {
            if id == SITE_FILTER_ID {
                return if value == SITE_FILTER_VALUE_NATSU {
                    SITE_NATSU
                } else {
                    SITE_KOMIKCAST
                };
            }
        }
    }
    SITE_KOMIKCAST
}

#[cfg(all(feature = "natsu", feature = "komikcast"))]
fn site_from_url(url: Option<&str>) -> &'static str {
    match url {
        Some(url) if url.contains("natsu.one") => SITE_NATSU,
        _ => SITE_KOMIKCAST,
    }
}
