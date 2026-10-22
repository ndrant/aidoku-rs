use aidoku::alloc::{String, Vec};
use aidoku::imports::net::Request;
use aidoku::{Chapter, FilterValue, Manga, MangaPageResult, Page, PageContext, Result};

#[cfg(feature = "kiryuu")]
use crate::sites::kiryuu;
#[cfg(feature = "komikcast")]
use crate::sites::komikcast;
#[cfg(feature = "komikindo")]
use crate::sites::komikindo;
#[cfg(feature = "komiku")]
use crate::sites::komiku;
#[cfg(feature = "manhwaindo")]
use crate::sites::manhwaindo;
#[cfg(feature = "natsu")]
use crate::sites::natsu;
#[cfg(feature = "shinigami")]
use crate::sites::shinigami;
#[cfg(feature = "westmanga")]
use crate::sites::westmanga;

#[cfg(all(
    feature = "komikcast",
    feature = "natsu",
    feature = "shinigami",
    feature = "kiryuu",
    feature = "komikindo",
    feature = "westmanga",
    feature = "manhwaindo",
    feature = "komiku"
))]
pub const SITE_KOMIKCAST: &str = "komikcast";
#[cfg(all(
    feature = "komikcast",
    feature = "natsu",
    feature = "shinigami",
    feature = "kiryuu",
    feature = "komikindo",
    feature = "westmanga",
    feature = "manhwaindo",
    feature = "komiku"
))]
pub const SITE_NATSU: &str = "natsu";
#[cfg(all(
    feature = "komikcast",
    feature = "natsu",
    feature = "shinigami",
    feature = "kiryuu",
    feature = "komikindo",
    feature = "westmanga",
    feature = "manhwaindo",
    feature = "komiku"
))]
pub const SITE_SHIINIGAMI: &str = "shinigami";
#[cfg(all(
    feature = "komikcast",
    feature = "natsu",
    feature = "shinigami",
    feature = "kiryuu",
    feature = "komikindo",
    feature = "westmanga",
    feature = "manhwaindo",
    feature = "komiku"
))]
pub const SITE_KIRYUU: &str = "kiryuu";
#[cfg(all(
    feature = "komikcast",
    feature = "natsu",
    feature = "shinigami",
    feature = "kiryuu",
    feature = "komikindo",
    feature = "westmanga",
    feature = "manhwaindo",
    feature = "komiku"
))]
pub const SITE_KOMIKINDO: &str = "komikindo";
#[cfg(all(
    feature = "komikcast",
    feature = "natsu",
    feature = "shinigami",
    feature = "kiryuu",
    feature = "komikindo",
    feature = "westmanga",
    feature = "manhwaindo",
    feature = "komiku"
))]
pub const SITE_WESTMANGA: &str = "westmanga";
#[cfg(all(
    feature = "komikcast",
    feature = "natsu",
    feature = "shinigami",
    feature = "kiryuu",
    feature = "komikindo",
    feature = "westmanga",
    feature = "manhwaindo",
    feature = "komiku"
))]
pub const SITE_MANHWAINDO: &str = "manhwaindo";
#[cfg(all(
    feature = "komikcast",
    feature = "natsu",
    feature = "shinigami",
    feature = "kiryuu",
    feature = "komikindo",
    feature = "westmanga",
    feature = "manhwaindo",
    feature = "komiku"
))]
pub const SITE_KOMIKU: &str = "komiku";

#[cfg(all(
    feature = "komikcast",
    feature = "natsu",
    feature = "shinigami",
    feature = "kiryuu",
    feature = "komikindo",
    feature = "westmanga",
    feature = "manhwaindo",
    feature = "komiku"
))]
pub const SITE_FILTER_ID: &str = "site";

/// The id of the sort filter declared in the site manifests (`res/*.json`).
/// Option indices: 0 = "Terbaru", 1 = "Populer".
pub const SORT_FILTER_ID: &str = "sort";

// --- SEARCH DISPATCH ---

#[cfg(all(
    feature = "natsu",
    not(feature = "komikcast"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn search(
    query: Option<String>,
    page: i32,
    filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    natsu::search(query, page, selected_sort_index(&filters))
}

#[cfg(all(
    feature = "komikcast",
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn search(
    query: Option<String>,
    page: i32,
    filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    komikcast::search(query, page, selected_sort_index(&filters))
}

#[cfg(all(
    feature = "shinigami",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn search(
    query: Option<String>,
    page: i32,
    filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    shinigami::search(query, page, selected_sort_index(&filters))
}

#[cfg(all(
    feature = "kiryuu",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn search(
    query: Option<String>,
    page: i32,
    filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    kiryuu::search(query, page, selected_sort_index(&filters))
}

#[cfg(all(
    feature = "komikindo",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn search(
    query: Option<String>,
    page: i32,
    filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    komikindo::search(query, page, selected_sort_index(&filters))
}

#[cfg(all(
    feature = "westmanga",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn search(
    query: Option<String>,
    page: i32,
    filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    westmanga::search(query, page, selected_sort_index(&filters))
}

#[cfg(all(
    feature = "manhwaindo",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "komiku")
))]
pub fn search(
    query: Option<String>,
    page: i32,
    filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    manhwaindo::search(query, page, selected_sort_index(&filters))
}

#[cfg(all(
    feature = "komiku",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo")
))]
pub fn search(
    query: Option<String>,
    page: i32,
    filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    komiku::search(query, page, selected_sort_index(&filters))
}

#[cfg(all(
    feature = "komikcast",
    feature = "natsu",
    feature = "shinigami",
    feature = "kiryuu",
    feature = "komikindo",
    feature = "westmanga",
    feature = "manhwaindo",
    feature = "komiku"
))]
pub fn search(
    query: Option<String>,
    page: i32,
    filters: Vec<FilterValue>,
) -> Result<MangaPageResult> {
    let sort = selected_sort_index(&filters);
    match selected_site(&filters) {
        SITE_NATSU => natsu::search(query, page, sort),
        SITE_SHIINIGAMI => shinigami::search(query, page, sort),
        SITE_KIRYUU => kiryuu::search(query, page, sort),
        SITE_KOMIKINDO => komikindo::search(query, page, sort),
        SITE_WESTMANGA => westmanga::search(query, page, sort),
        SITE_MANHWAINDO => manhwaindo::search(query, page, sort),
        SITE_KOMIKU => komiku::search(query, page, sort),
        _ => komikcast::search(query, page, sort),
    }
}

// --- MANGA UPDATE DISPATCH ---

#[cfg(all(
    feature = "natsu",
    not(feature = "komikcast"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    natsu::manga_update(manga, needs_details, needs_chapters)
}

#[cfg(all(
    feature = "komikcast",
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    komikcast::manga_update(manga, needs_details, needs_chapters)
}

#[cfg(all(
    feature = "shinigami",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    shinigami::manga_update(manga, needs_details, needs_chapters)
}

#[cfg(all(
    feature = "kiryuu",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    kiryuu::manga_update(manga, needs_details, needs_chapters)
}

#[cfg(all(
    feature = "komikindo",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    komikindo::manga_update(manga, needs_details, needs_chapters)
}

#[cfg(all(
    feature = "westmanga",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    westmanga::manga_update(manga, needs_details, needs_chapters)
}

#[cfg(all(
    feature = "manhwaindo",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "komiku")
))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    manhwaindo::manga_update(manga, needs_details, needs_chapters)
}

#[cfg(all(
    feature = "komiku",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo")
))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    komiku::manga_update(manga, needs_details, needs_chapters)
}

#[cfg(all(
    feature = "komikcast",
    feature = "natsu",
    feature = "shinigami",
    feature = "kiryuu",
    feature = "komikindo",
    feature = "westmanga",
    feature = "manhwaindo",
    feature = "komiku"
))]
pub fn manga_update(manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
    match site_from_url(manga.url.as_deref()) {
        SITE_NATSU => natsu::manga_update(manga, needs_details, needs_chapters),
        SITE_SHIINIGAMI => shinigami::manga_update(manga, needs_details, needs_chapters),
        SITE_KIRYUU => kiryuu::manga_update(manga, needs_details, needs_chapters),
        SITE_KOMIKINDO => komikindo::manga_update(manga, needs_details, needs_chapters),
        SITE_WESTMANGA => westmanga::manga_update(manga, needs_details, needs_chapters),
        SITE_MANHWAINDO => manhwaindo::manga_update(manga, needs_details, needs_chapters),
        SITE_KOMIKU => komiku::manga_update(manga, needs_details, needs_chapters),
        _ => komikcast::manga_update(manga, needs_details, needs_chapters),
    }
}

// --- PAGE LIST DISPATCH ---

#[cfg(all(
    feature = "natsu",
    not(feature = "komikcast"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    natsu::page_list(manga, chapter)
}

#[cfg(all(
    feature = "komikcast",
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    komikcast::page_list(manga, chapter)
}

#[cfg(all(
    feature = "shinigami",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    shinigami::page_list(manga, chapter)
}

#[cfg(all(
    feature = "kiryuu",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    kiryuu::page_list(manga, chapter)
}

#[cfg(all(
    feature = "komikindo",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    komikindo::page_list(manga, chapter)
}

#[cfg(all(
    feature = "westmanga",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    westmanga::page_list(manga, chapter)
}

#[cfg(all(
    feature = "manhwaindo",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "komiku")
))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    manhwaindo::page_list(manga, chapter)
}

#[cfg(all(
    feature = "komiku",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo")
))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    komiku::page_list(manga, chapter)
}

#[cfg(all(
    feature = "komikcast",
    feature = "natsu",
    feature = "shinigami",
    feature = "kiryuu",
    feature = "komikindo",
    feature = "westmanga",
    feature = "manhwaindo",
    feature = "komiku"
))]
pub fn page_list(manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
    match site_from_url(manga.url.as_deref()) {
        SITE_NATSU => natsu::page_list(manga, chapter),
        SITE_SHIINIGAMI => shinigami::page_list(manga, chapter),
        SITE_KIRYUU => kiryuu::page_list(manga, chapter),
        SITE_KOMIKINDO => komikindo::page_list(manga, chapter),
        SITE_WESTMANGA => westmanga::page_list(manga, chapter),
        SITE_MANHWAINDO => manhwaindo::page_list(manga, chapter),
        SITE_KOMIKU => komiku::page_list(manga, chapter),
        _ => komikcast::page_list(manga, chapter),
    }
}

// --- IMAGE REQUEST DISPATCH ---

#[cfg(all(
    feature = "natsu",
    not(feature = "komikcast"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    natsu::image_request(url)
}

#[cfg(all(
    feature = "komikcast",
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    komikcast::image_request(url)
}

#[cfg(all(
    feature = "shinigami",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    shinigami::image_request(url)
}

#[cfg(all(
    feature = "kiryuu",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    kiryuu::image_request(url)
}

#[cfg(all(
    feature = "komikindo",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    komikindo::image_request(url)
}

#[cfg(all(
    feature = "westmanga",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "manhwaindo"),
    not(feature = "komiku")
))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    westmanga::image_request(url)
}

#[cfg(all(
    feature = "manhwaindo",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "komiku")
))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    manhwaindo::image_request(url)
}

#[cfg(all(
    feature = "komiku",
    not(feature = "komikcast"),
    not(feature = "natsu"),
    not(feature = "shinigami"),
    not(feature = "kiryuu"),
    not(feature = "komikindo"),
    not(feature = "westmanga"),
    not(feature = "manhwaindo")
))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    komiku::image_request(url)
}

#[cfg(all(
    feature = "komikcast",
    feature = "natsu",
    feature = "shinigami",
    feature = "kiryuu",
    feature = "komikindo",
    feature = "westmanga",
    feature = "manhwaindo",
    feature = "komiku"
))]
pub fn image_request(url: String, _context: Option<PageContext>) -> Result<Request> {
    if url.contains("natsu") {
        natsu::image_request(url)
    } else if url.contains("shngm") || url.contains("shinigami") {
        shinigami::image_request(url)
    } else if url.contains("kiryuu") || url.contains("yuucdn") {
        kiryuu::image_request(url)
    } else if url.contains("komikindo")
        || url.contains("amatipolanyatirudesainnya")
        || url.contains("bukansiapasiapa")
    {
        komikindo::image_request(url)
    } else if url.contains("westmanga") {
        westmanga::image_request(url)
    } else if url.contains("manhwaindo") || url.contains("gmbr.pro") {
        manhwaindo::image_request(url)
    } else if url.contains("komiku") {
        komiku::image_request(url)
    } else {
        komikcast::image_request(url)
    }
}

#[cfg(all(
    feature = "komikcast",
    feature = "natsu",
    feature = "shinigami",
    feature = "kiryuu",
    feature = "komikindo",
    feature = "westmanga",
    feature = "manhwaindo",
    feature = "komiku"
))]
fn selected_site(filters: &[FilterValue]) -> &'static str {
    for filter in filters {
        if let FilterValue::Select { id, value } = filter {
            if id == SITE_FILTER_ID {
                return match value.as_str() {
                    SITE_NATSU => SITE_NATSU,
                    SITE_SHIINIGAMI => SITE_SHIINIGAMI,
                    SITE_KIRYUU => SITE_KIRYUU,
                    SITE_KOMIKINDO => SITE_KOMIKINDO,
                    SITE_WESTMANGA => SITE_WESTMANGA,
                    SITE_MANHWAINDO => SITE_MANHWAINDO,
                    SITE_KOMIKU => SITE_KOMIKU,
                    _ => SITE_KOMIKCAST,
                };
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

#[cfg(all(
    feature = "komikcast",
    feature = "natsu",
    feature = "shinigami",
    feature = "kiryuu",
    feature = "komikindo",
    feature = "westmanga",
    feature = "manhwaindo",
    feature = "komiku"
))]
fn site_from_url(url: Option<&str>) -> &'static str {
    match url {
        Some(url) if url.contains("natsu.one") => SITE_NATSU,
        Some(url) if url.contains("shinigami") || url.contains("shngm") => SITE_SHIINIGAMI,
        Some(url) if url.contains("kiryuu") => SITE_KIRYUU,
        Some(url) if url.contains("komikindo") => SITE_KOMIKINDO,
        Some(url) if url.contains("westmanga") => SITE_WESTMANGA,
        Some(url) if url.contains("manhwaindo") => SITE_MANHWAINDO,
        Some(url) if url.contains("komiku") => SITE_KOMIKU,
        _ => SITE_KOMIKCAST,
    }
}
