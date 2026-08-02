use aidoku::alloc::{String, Vec};
use aidoku::imports::net::Request;
use aidoku::{Chapter, FilterValue, Manga, MangaPageResult, Page, PageContext, Result};

#[cfg(feature = "komikcast")]
use crate::sites::komikcast;
#[cfg(feature = "natsu")]
use crate::sites::natsu;
#[cfg(feature = "shinigami")]
use crate::sites::shinigami;

#[cfg(all(feature = "komikcast", feature = "natsu", feature = "shinigami"))]
pub const SITE_KOMIKCAST: &str = "komikcast";
#[cfg(all(feature = "komikcast", feature = "natsu", feature = "shinigami"))]
pub const SITE_NATSU: &str = "natsu";
#[cfg(all(feature = "komikcast", feature = "natsu", feature = "shinigami"))]
pub const SITE_SHIINIGAMI: &str = "shinigami";

#[cfg(all(feature = "komikcast", feature = "natsu", feature = "shinigami"))]
pub const SITE_FILTER_ID: &str = "site";
#[cfg(all(feature = "komikcast", feature = "natsu", feature = "shinigami"))]
pub const SITE_FILTER_VALUE_NATSU: &str = "natsu";
#[cfg(all(feature = "komikcast", feature = "natsu", feature = "shinigami"))]
pub const SITE_FILTER_VALUE_SHIINIGAMI: &str = "shinigami";

/// The id of the sort filter declared in the site manifests (`res/*.json`).
///
/// Option indices are shared by all sites: 0 = "Terbaru", 1 = "Populer".
pub const SORT_FILTER_ID: &str = "sort";

/// Dispatches a search to the Natsu-only build.
#[cfg(all(
    feature = "natsu",
    not(feature = "komikcast"),
    not(feature = "shinigami")
))]
pub fn search(
    query: Option<String>,
    page: i32,
    filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    natsu::search(query, page, selected_sort_index(&filters))
}

/// Dispatches a search to the KomikCast-only build.
#[cfg(all(
    feature = "komikcast",
    not(feature = "natsu"),
    not(feature = "shinigami")
))]
pub fn search(
    query: Option<String>,
    page: i32,
    filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    komikcast::search(query, page, selected_sort_index(&filters))
}

/// Dispatches a search to the Shinigami-only build.
#[cfg(all(
    feature = "shinigami",
    not(feature = "komikcast"),
    not(feature = "natsu")
))]
pub fn search(
    query: Option<String>,
    page: i32,
    filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    shinigami::search(query, page, selected_sort_index(&filters))
}

/// Dispatches a search to the site selected through filters, defaulting to
/// KomikCast when no selection is present (multi-site build).
#[cfg(all(feature = "komikcast", feature = "natsu", feature = "shinigami"))]
pub fn search(
    query: Option<String>,
    page: i32,
    filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    let sort = selected_sort_index(&filters);
    match selected_site(&filters) {
        SITE_NATSU => natsu::search(query, page, sort),
        SITE_SHIINIGAMI => shinigami::search(query, page, sort),
        _ => komikcast::search(query, page, sort),
    }
}

/// Refreshes manga details from the Natsu-only build.
#[cfg(all(
    feature = "natsu",
    not(feature = "komikcast"),
    not(feature = "shinigami")
))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    natsu::manga_update(manga, needs_details, needs_chapters)
}

/// Refreshes manga details from the KomikCast-only build.
#[cfg(all(
    feature = "komikcast",
    not(feature = "natsu"),
    not(feature = "shinigami")
))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    komikcast::manga_update(manga, needs_details, needs_chapters)
}

/// Refreshes manga details from the Shinigami-only build.
#[cfg(all(
    feature = "shinigami",
    not(feature = "komikcast"),
    not(feature = "natsu")
))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    shinigami::manga_update(manga, needs_details, needs_chapters)
}

/// Refreshes manga details from the site that owns the manga URL
/// (multi-site build).
#[cfg(all(feature = "komikcast", feature = "natsu", feature = "shinigami"))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    match site_from_url(manga.url.as_deref()) {
        SITE_NATSU => natsu::manga_update(manga, needs_details, needs_chapters),
        SITE_SHIINIGAMI => shinigami::manga_update(manga, needs_details, needs_chapters),
        _ => komikcast::manga_update(manga, needs_details, needs_chapters),
    }
}

/// Fetches page images from the Natsu-only build.
#[cfg(all(
    feature = "natsu",
    not(feature = "komikcast"),
    not(feature = "shinigami")
))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    natsu::page_list(manga, chapter)
}

/// Fetches page images from the KomikCast-only build.
#[cfg(all(
    feature = "komikcast",
    not(feature = "natsu"),
    not(feature = "shinigami")
))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    komikcast::page_list(manga, chapter)
}

/// Fetches page images from the Shinigami-only build.
#[cfg(all(
    feature = "shinigami",
    not(feature = "komikcast"),
    not(feature = "natsu")
))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    shinigami::page_list(manga, chapter)
}

/// Fetches page images from the site that owns the manga URL (multi-site build).
#[cfg(all(feature = "komikcast", feature = "natsu", feature = "shinigami"))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    match site_from_url(manga.url.as_deref()) {
        SITE_NATSU => natsu::page_list(manga, chapter),
        SITE_SHIINIGAMI => shinigami::page_list(manga, chapter),
        _ => komikcast::page_list(manga, chapter),
    }
}

/// Builds the request used to download an image in the Natsu-only build.
#[cfg(all(
    feature = "natsu",
    not(feature = "komikcast"),
    not(feature = "shinigami")
))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    natsu::image_request(url)
}

/// Builds the request used to download an image in the KomikCast-only build.
#[cfg(all(
    feature = "komikcast",
    not(feature = "natsu"),
    not(feature = "shinigami")
))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    komikcast::image_request(url)
}

/// Builds the request used to download an image in the Shinigami-only build.
#[cfg(all(
    feature = "shinigami",
    not(feature = "komikcast"),
    not(feature = "natsu")
))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    shinigami::image_request(url)
}

/// Builds the request used to download an image for whichever site owns it
/// (multi-site build).
#[cfg(all(feature = "komikcast", feature = "natsu", feature = "shinigami"))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    if url.contains("natsu") {
        natsu::image_request(url)
    } else if url.contains("shngm") || url.contains("shinigami") {
        shinigami::image_request(url)
    } else {
        komikcast::image_request(url)
    }
}

#[cfg(all(feature = "komikcast", feature = "natsu", feature = "shinigami"))]
fn selected_site(filters: &[FilterValue]) -> &'static str {
    for filter in filters {
        if let FilterValue::Select { id, value } = filter {
            if id == SITE_FILTER_ID {
                if value == SITE_FILTER_VALUE_NATSU {
                    return SITE_NATSU;
                }
                if value == SITE_FILTER_VALUE_SHIINIGAMI {
                    return SITE_SHIINIGAMI;
                }
                return SITE_KOMIKCAST;
            }
        }
    }
    SITE_KOMIKCAST
}

/// Returns the index of the selected sort option, defaulting to the first
/// option ("Terbaru") when no sort filter is present.
pub fn selected_sort_index(filters: &[FilterValue]) -> usize {
    filters
        .iter()
        .find_map(|filter| match filter {
            FilterValue::Sort { id, index, .. } if id == SORT_FILTER_ID => Some(*index as usize),
            _ => None,
        })
        .unwrap_or(0)
}

#[cfg(all(feature = "komikcast", feature = "natsu", feature = "shinigami"))]
fn site_from_url(url: Option<&str>) -> &'static str {
    match url {
        Some(url) if url.contains("natsu.one") => SITE_NATSU,
        Some(url) if url.contains("shinigami") || url.contains("shngm") => SITE_SHIINIGAMI,
        _ => SITE_KOMIKCAST,
    }
}
