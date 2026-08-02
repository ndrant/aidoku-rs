use aidoku::alloc::{String, Vec};
use aidoku::{Chapter, FilterValue, Manga, MangaPageResult, Page, Result};

use crate::sites::{komikcast, natsu};

pub const SITE_KOMIKCAST: &str = "komikcast";
pub const SITE_NATSU: &str = "natsu";

pub const SITE_FILTER_ID: &str = "site";
pub const SITE_FILTER_VALUE_NATSU: &str = "natsu";

/// Dispatches a search to the site selected through filters, defaulting to
/// KomikCast when no selection is present.
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

/// Dispatches a manga update to the site that owns the manga URL.
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    if site_from_url(manga.url.as_deref()) == SITE_NATSU {
        natsu::manga_update(manga, needs_details, needs_chapters)
    } else {
        komikcast::manga_update(manga, needs_details, needs_chapters)
    }
}

/// Dispatches a page list request to the site that owns the manga URL.
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    if site_from_url(manga.url.as_deref()) == SITE_NATSU {
        natsu::page_list(manga, chapter)
    } else {
        komikcast::page_list(manga, chapter)
    }
}

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

fn site_from_url(url: Option<&str>) -> &'static str {
    match url {
        Some(url) if url.contains("natsu.one") => SITE_NATSU,
        _ => SITE_KOMIKCAST,
    }
}
