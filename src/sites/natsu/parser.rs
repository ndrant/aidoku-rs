use aidoku::alloc::{String, Vec, vec};
use aidoku::imports::html::{Document, Element};
use aidoku::prelude::*;
use aidoku::{Page, PageContent, Viewer};

use crate::models::{ChapterInfo, MangaInfo, Status};
use crate::parser as html;
use crate::utils;

pub const MANGA_LINK_SELECTOR: &str = "a[href*='/manga/']";
pub const FORMAT_BADGE_SELECTOR: &str = "img[src*='/static/svg/']";
const CHAPTER_MARKER: &str = "/chapter-";

/// Extracts the manga slug from a manga page URL such as
/// `https://natsu.one/manga/one-piece/`.
pub fn slug_from_url(url: &str) -> Option<String> {
    let marker = "/manga/";
    let start = url.find(marker)? + marker.len();
    let rest = &url[start..];
    let end = rest.find('/').unwrap_or(rest.len());
    let slug = rest[..end].trim();
    if slug.is_empty() {
        None
    } else {
        Some(String::from(slug))
    }
}

/// Extracts the chapter identifier from a chapter URL such as
/// `https://natsu.one/manga/one-piece/chapter-1189.391109/`.
pub fn chapter_key_from_url(url: &str) -> Option<String> {
    let start = url.find(CHAPTER_MARKER)? + CHAPTER_MARKER.len();
    let rest = &url[start..];
    let end = rest.find('/').unwrap_or(rest.len());
    let key = rest[..end].trim();
    if key.is_empty() {
        None
    } else {
        Some(String::from(key))
    }
}

/// Parses the server-rendered manga grid on `/latest-update/` pages.
pub fn manga_info_from_listing(document: &Document, base_url: &str) -> Vec<MangaInfo> {
    let mut entries = Vec::new();
    if let Some(cards) = document.select("#search-results > div") {
        for card in cards {
            if let Some(info) = manga_info_from_card(&card, base_url) {
                entries.push(info);
            }
        }
    }
    entries
}

/// Parses a single manga card from the latest-update grid.
pub fn manga_info_from_card(card: &Element, base_url: &str) -> Option<MangaInfo> {
    let url = card
        .select_first(MANGA_LINK_SELECTOR)
        .and_then(|link| html::attr(&link, "href"))
        .and_then(|href| html::resolve_url(&href, base_url))?;
    let key = slug_from_url(&url)?;
    let title = card
        .select_first(format!("{MANGA_LINK_SELECTOR} h1"))
        .and_then(|title| html::text(&title))?;
    let cover = card
        .select_first(format!("{MANGA_LINK_SELECTOR} img"))
        .and_then(|img| html::attr(&img, "src"))
        .map(|src| utils::remove_size_suffix(&src));
    let status = card
        .select_first("p.font-normal.text-xs")
        .and_then(|p| html::text(&p))
        .map(|text| status_from_str(Some(&text)))
        .unwrap_or(Status::Unknown);
    Some(MangaInfo {
        key,
        title,
        cover,
        url: Some(url),
        status,
        viewer: viewer_from_badge(card),
        ..MangaInfo::default()
    })
}

/// Parses the `<a>` results returned by the AJAX search endpoint.
pub fn manga_info_from_search_results(document: &Document, base_url: &str) -> Vec<MangaInfo> {
    let mut entries = Vec::new();
    if let Some(results) = document.select("#searchResults > a") {
        for result in results {
            if let Some(info) = manga_info_from_search_result(&result, base_url) {
                entries.push(info);
            }
        }
    }
    entries
}

/// Parses the manga cards returned by the AJAX advanced-search endpoint,
/// where each card is a top-level `<div>` in the response fragment. The card
/// structure matches the latest-update grid, so cards share [manga_info_from_card].
pub fn manga_info_from_advanced_results(document: &Document, base_url: &str) -> Vec<MangaInfo> {
    let mut entries = Vec::new();
    if let Some(cards) = document.select("body > div") {
        for card in cards {
            if let Some(info) = manga_info_from_card(&card, base_url) {
                entries.push(info);
            }
        }
    }
    entries
}

fn manga_info_from_search_result(result: &Element, base_url: &str) -> Option<MangaInfo> {
    let url = html::attr(result, "href").and_then(|href| html::resolve_url(&href, base_url))?;
    let key = slug_from_url(&url)?;
    let title = result.select_first("h3").and_then(|h3| html::text(&h3))?;
    let cover = result
        .select_first("img")
        .and_then(|img| html::attr(&img, "src"));
    let description = result.select_first("p").and_then(|p| html::text(&p));
    Some(MangaInfo {
        key,
        title,
        cover,
        description,
        url: Some(url),
        ..MangaInfo::default()
    })
}

/// Parses a manga detail page into a [MangaInfo]. `slug` supplies the key.
pub fn manga_info_from_detail(
    document: &Document,
    base_url: &str,
    slug: &str,
) -> Option<MangaInfo> {
    let title = document
        .select_first("h1[itemprop='name']")
        .and_then(|h1| html::text(&h1))?;
    let cover = document
        .select_first("meta[property='og:image']")
        .and_then(|meta| html::attr(&meta, "content"))
        .map(|cover| utils::remove_size_suffix(&cover));
    let description = document
        .select_first("div[itemprop='description'][data-show='false']")
        .and_then(|desc| html::text(&desc))
        .or_else(|| {
            document
                .select_first("div[itemprop='description']")
                .and_then(|desc| html::text(&desc))
        });
    let ld = json_ld_info(document);
    let tags = ld.genres.or_else(|| {
        document.select("a[itemprop='genre']").map(|links| {
            links
                .into_iter()
                .filter_map(|link| html::text(&link))
                .collect::<Vec<_>>()
        })
    });
    Some(MangaInfo {
        key: String::from(slug),
        title,
        cover,
        authors: ld.authors,
        description,
        url: Some(format!("{base_url}/manga/{slug}/")),
        tags,
        status: ld.status,
        viewer: viewer_from_document(document),
        ..MangaInfo::default()
    })
}

/// Extracted data from the page's JSON-LD (schema.org) scripts.
pub struct JsonLdData {
    pub authors: Option<Vec<String>>,
    pub genres: Option<Vec<String>>,
    pub status: Status,
}

/// Reads author, genre and status from `application/ld+json` scripts,
/// preferring the ComicSeries node that carries an `author` field.
pub fn json_ld_info(document: &Document) -> JsonLdData {
    let mut data = JsonLdData {
        authors: None,
        genres: None,
        status: Status::Unknown,
    };
    if let Some(scripts) = document.select("script[type='application/ld+json']") {
        for script in scripts {
            let Some(raw) = script.html() else { continue };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
                continue;
            };
            if value.get("author").is_none() && value.get("creativeWorkStatus").is_none() {
                continue;
            }
            if data.authors.is_none() {
                data.authors = value.get("author").and_then(ld_names);
            }
            if data.genres.is_none() {
                data.genres = value.get("genre").and_then(ld_genres);
            }
            if data.status == Status::Unknown {
                data.status = value
                    .get("creativeWorkStatus")
                    .and_then(|status| status.as_str())
                    .map(|status| status_from_str(Some(status)))
                    .unwrap_or(Status::Unknown);
            }
            if data.authors.is_some() && data.genres.is_some() && data.status != Status::Unknown {
                break;
            }
        }
    }
    data
}

/// Reads a `name` from a JSON-LD Person or list of Persons.
fn ld_names(value: &serde_json::Value) -> Option<Vec<String>> {
    let names = match value {
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|item| item.get("name").and_then(|name| name.as_str()))
            .map(String::from)
            .collect(),
        _ => vec![
            value
                .get("name")
                .and_then(|name| name.as_str())
                .map(String::from)
                .unwrap_or_default(),
        ],
    };
    let names = names
        .into_iter()
        .filter(|name| !name.is_empty())
        .collect::<Vec<_>>();
    if names.is_empty() { None } else { Some(names) }
}

/// Reads a JSON-LD `genre` value (single string or list of strings).
fn ld_genres(value: &serde_json::Value) -> Option<Vec<String>> {
    match value {
        serde_json::Value::String(genre) => Some(vec![genre.clone()]),
        serde_json::Value::Array(items) => {
            let genres = items
                .iter()
                .filter_map(|item| item.as_str())
                .map(String::from)
                .collect::<Vec<_>>();
            if genres.is_empty() {
                None
            } else {
                Some(genres)
            }
        }
        _ => None,
    }
}

/// Parses the server-rendered chapter list into [ChapterInfo] values.
pub fn chapters_from_document(document: &Document, base_url: &str) -> Vec<ChapterInfo> {
    let mut chapters = Vec::new();
    if let Some(rows) = document.select("#chapter-list [data-chapter-number]") {
        for row in rows {
            if let Some(chapter) = chapter_from_row(&row, base_url) {
                chapters.push(chapter);
            }
        }
    }
    chapters
}

fn chapter_from_row(row: &Element, base_url: &str) -> Option<ChapterInfo> {
    let url = row
        .select_first("a[href*='/chapter-']")
        .and_then(|link| html::attr(&link, "href"))
        .and_then(|href| html::resolve_url(&href, base_url))?;
    let key = chapter_key_from_url(&url)?;
    let title = row
        .select_first("a[href*='/chapter-'] span")
        .and_then(|span| html::text(&span));
    let date_uploaded = row
        .select_first("time")
        .and_then(|time| html::attr(&time, "datetime"))
        .as_deref()
        .and_then(utils::iso8601_seconds);
    Some(ChapterInfo {
        key,
        title: title.clone(),
        chapter_number: chapter_number_from_title(title.as_deref()),
        date_uploaded,
        url: Some(url),
    })
}

/// Parses a chapter title such as "Chapter 1189" into its number.
fn chapter_number_from_title(title: Option<&str>) -> Option<f32> {
    let rest = title?.trim().to_lowercase();
    let rest = rest.strip_prefix("chapter ")?;
    let digits: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    digits.trim_end_matches('.').parse::<f32>().ok()
}

/// Reads the server-rendered page images of a chapter reader page.
pub fn pages_from_document(document: &Document) -> Vec<Page> {
    let mut pages = Vec::new();
    if let Some(sections) = document.select("section[data-image-data]") {
        for section in sections {
            if let Some(images) = section.select("img") {
                for image in images {
                    if let Some(src) = html::attr(&image, "src") {
                        pages.push(Page {
                            content: PageContent::url(&src),
                            ..Page::default()
                        });
                    }
                }
            }
        }
    }
    if pages.is_empty()
        && let Some(src) = document
            .select_first("meta[property='og:image']")
            .and_then(|meta| html::attr(&meta, "content"))
    {
        pages.push(Page {
            content: PageContent::url(&src),
            ..Page::default()
        });
    }
    pages
}

/// Derives the reading direction from the format badge shown on cards and
/// detail pages (`/static/svg/manhua.svg`, `alt="manhua"`, ...).
pub fn viewer_from_badge(root: &Element) -> Viewer {
    root.select_first(FORMAT_BADGE_SELECTOR)
        .map(|badge| viewer_from_format_badge(&badge))
        .unwrap_or(Viewer::Unknown)
}

/// Same as [viewer_from_badge], but for a whole [Document] such as a
/// manga detail page.
pub fn viewer_from_document(document: &Document) -> Viewer {
    document
        .select_first(FORMAT_BADGE_SELECTOR)
        .map(|badge| viewer_from_format_badge(&badge))
        .unwrap_or(Viewer::Unknown)
}

fn viewer_from_format_badge(badge: &Element) -> Viewer {
    let format = html::attr(badge, "alt")
        .or_else(|| html::attr(badge, "src"))
        .map(|value| format_from_svg_src(&value).to_lowercase());
    viewer_from_format(format.as_deref())
}

/// Maps a format string to an Aidoku [Viewer].
pub fn viewer_from_format(format: Option<&str>) -> Viewer {
    match format {
        Some("manhwa") | Some("manhua") | Some("webtoon") => Viewer::Webtoon,
        Some("manga") => Viewer::RightToLeft,
        _ => Viewer::Unknown,
    }
}

/// Maps a Natsu status string to a [Status].
pub fn status_from_str(status: Option<&str>) -> Status {
    match status.map(str::trim).map(str::to_lowercase).as_deref() {
        Some("ongoing") | Some("berlangsung") => Status::Ongoing,
        Some("completed") | Some("selesai") | Some("complete") => Status::Completed,
        Some("cancelled") | Some("canceled") => Status::Cancelled,
        Some("hiatus") | Some("on hiatus") => Status::Hiatus,
        _ => Status::Unknown,
    }
}

/// Extracts the format name from a badge `src` or `alt` value, e.g.
/// `.../static/svg/manhua.svg` -> `manhua`.
fn format_from_svg_src(value: &str) -> String {
    let name = value.rsplit('/').next().unwrap_or(value);
    let name = name.strip_suffix(".svg").unwrap_or(name);
    String::from(name)
}

/// Extracts the `nonce` query value from an `hx-post` attribute.
pub fn extract_nonce(attr: &str) -> Option<String> {
    let key = "nonce=";
    let start = attr.find(key)? + key.len();
    let rest = &attr[start..];
    let end = rest.find('&').unwrap_or(rest.len());
    let nonce = rest[..end].trim();
    if nonce.is_empty() {
        None
    } else {
        Some(String::from(nonce))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidoku::MangaStatus;
    use aidoku::imports::html::Html;
    use aidoku_test::aidoku_test;

    const BASE: &str = "https://natsu.one";

    fn parse(html: &str) -> Document {
        Html::parse(html).expect("fixture must parse")
    }

    #[aidoku_test]
    fn slug_from_url_extracts_slug() {
        assert_eq!(
            slug_from_url("https://natsu.one/manga/one-piece/"),
            Some(String::from("one-piece"))
        );
        assert_eq!(slug_from_url("https://natsu.one/"), None);
    }

    #[aidoku_test]
    fn chapter_key_from_url_extracts_id() {
        assert_eq!(
            chapter_key_from_url("https://natsu.one/manga/one-piece/chapter-1189.391109/"),
            Some(String::from("1189.391109"))
        );
    }

    #[aidoku_test]
    fn listing_cards_map_to_manga_info() {
        let document = parse(
            r#"<div id="search-results">
				<div>
					<a href="https://natsu.one/manga/one-piece/"><img src="https://natsu.one/wp-content/uploads/2025/09/47c0ee13-320x427.png"></a>
					<span class="absolute"><img src="https://natsu.one/wp-content/themes/natsu_id/static/svg/manhua.svg" alt="manhua"></span>
					<div class="flex"><p class="font-normal text-xs">Ongoing</p></div>
					<a href="https://natsu.one/manga/one-piece/"><h1 class="text-[15px]">One Piece</h1></a>
				</div>
				<div>
					<a href="https://natsu.one/manga/dungeon-hunter/"><img src="https://natsu.one/wp-content/uploads/2025/09/x-768x1196.jpg"></a>
					<span><img src="https://natsu.one/wp-content/themes/natsu_id/static/svg/manga.svg" alt=""></span>
					<div><p class="font-normal text-xs">Completed</p></div>
					<a href="https://natsu.one/manga/dungeon-hunter/"><h1>Dungeon Hunter</h1></a>
				</div>
			</div>"#,
        );
        let entries = manga_info_from_listing(&document, BASE);
        assert_eq!(entries.len(), 2);

        let first = &entries[0];
        assert_eq!(first.key, "one-piece");
        assert_eq!(first.title, "One Piece");
        assert_eq!(
            first.cover.as_deref(),
            Some("https://natsu.one/wp-content/uploads/2025/09/47c0ee13.png")
        );
        assert_eq!(first.status, Status::Ongoing);
        assert_eq!(first.viewer, Viewer::Webtoon);
        assert_eq!(
            first.url.as_deref(),
            Some("https://natsu.one/manga/one-piece/")
        );

        let second = &entries[1];
        assert_eq!(second.viewer, Viewer::RightToLeft);
        assert_eq!(second.status, Status::Completed);
    }

    #[aidoku_test]
    fn search_results_map_to_manga_info() {
        let document = parse(
            r#"<div id="searchResults">
				<a href="https://natsu.one/manga/one-piece/">
					<img src="https://natsu.one/wp-content/uploads/2025/09/47c0ee13.jpg" alt="One Piece">
					<div><h3>One Piece</h3><p>Pirates and treasure.</p></div>
				</a>
			</div>"#,
        );
        let entries = manga_info_from_search_results(&document, BASE);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "one-piece");
        assert_eq!(
            entries[0].description.as_deref(),
            Some("Pirates and treasure.")
        );
    }

    #[aidoku_test]
    fn advanced_results_map_to_manga_info() {
        let document = parse(
            r#"<div>
				<div class="flex rounded-lg overflow-hidden h-46 group-data-[mode=vertical]:hidden">
					<a href="https://natsu.one/manga/martial-peak/" class="min-w-[120px] w-23 h-full relative">
						<img src="https://natsu.one/wp-content/uploads/2025/09/51a42f3f-320x452.png">
					</a>
					<a href="https://natsu.one/manga/martial-peak/" class="text-base font-medium text-white">Martial Peak</a>
					<span class="bg-accent text-xs px-2 py-0.5 rounded-lg">Ongoing</span>
				</div>
				<div class="group-data-[mode=horizontal]:hidden">
					<a href="https://natsu.one/manga/martial-peak/"><img src="https://natsu.one/wp-content/uploads/2025/09/51a42f3f-320x452.png"></a>
					<span><img src="https://natsu.one/wp-content/themes/natsu_id/static/svg/manhua.svg" alt="manhua"></span>
					<div><p class="font-normal text-xs">Ongoing</p></div>
					<a href="https://natsu.one/manga/martial-peak/"><h1>Martial Peak</h1></a>
				</div>
			</div>
			<div>
				<div class="group-data-[mode=horizontal]:hidden">
					<a href="https://natsu.one/manga/one-piece/"><img src="https://natsu.one/wp-content/uploads/2025/09/47c0ee13.png"></a>
					<span><img src="https://natsu.one/wp-content/themes/natsu_id/static/svg/manga.svg" alt=""></span>
					<div><p class="font-normal text-xs">Completed</p></div>
					<a href="https://natsu.one/manga/one-piece/"><h1>One Piece</h1></a>
				</div>
			</div>
			<div class="flex justify-center my-8 col-span-full">
				<button>1</button><button>2</button>
			</div>"#,
        );
        let entries = manga_info_from_advanced_results(&document, BASE);
        assert_eq!(entries.len(), 2);

        assert_eq!(entries[0].key, "martial-peak");
        assert_eq!(entries[0].title, "Martial Peak");
        assert_eq!(entries[0].status, Status::Ongoing);
        assert_eq!(entries[0].viewer, Viewer::Webtoon);
        assert_eq!(
            entries[0].cover.as_deref(),
            Some("https://natsu.one/wp-content/uploads/2025/09/51a42f3f.png")
        );
        assert_eq!(
            entries[0].url.as_deref(),
            Some("https://natsu.one/manga/martial-peak/")
        );

        assert_eq!(entries[1].key, "one-piece");
        assert_eq!(entries[1].viewer, Viewer::RightToLeft);
        assert_eq!(entries[1].status, Status::Completed);
    }

    #[aidoku_test]
    fn detail_page_maps_to_manga_info() {
        let document = parse(
            r#"<html>
				<head>
					<meta property="og:image" content="https://natsu.one/wp-content/uploads/2025/09/47c0ee13-e6d5-40c0-bfb9-57b26b2b1d2a-768x1196.jpg">
					<meta property="og:url" content="https://natsu.one/manga/one-piece/">
					<script type="application/ld+json">{"@type":["Book","ComicSeries"],"name":"One Piece","author":{"@type":"Person","name":"ODA Eiichiro"},"genre":["Action","Adventure"],"creativeWorkStatus":"Ongoing"}</script>
				</head>
				<body>
					<h1 itemprop="name">One Piece</h1>
					<img src="https://natsu.one/wp-content/themes/natsu_id/static/svg/manga.svg" alt="">
					<div itemprop="description" data-show="false">Synopsis here.</div>
					<a itemprop="genre" href="https://natsu.one/genre/action/">Action</a>
				</body>
			</html>"#,
        );
        let info = manga_info_from_detail(&document, BASE, "one-piece").expect("detail");
        assert_eq!(info.key, "one-piece");
        assert_eq!(info.title, "One Piece");
        assert_eq!(
            info.cover.as_deref(),
            Some(
                "https://natsu.one/wp-content/uploads/2025/09/47c0ee13-e6d5-40c0-bfb9-57b26b2b1d2a.jpg"
            )
        );
        assert_eq!(info.description.as_deref(), Some("Synopsis here."));
        assert_eq!(info.authors, Some(vec![String::from("ODA Eiichiro")]));
        assert_eq!(
            info.tags,
            Some(vec![String::from("Action"), String::from("Adventure")])
        );
        assert_eq!(info.status, Status::Ongoing);
        assert_eq!(info.viewer, Viewer::RightToLeft);
    }

    #[aidoku_test]
    fn json_ld_missing_script_returns_defaults() {
        let document = parse(r#"<html><body><p>no json</p></body></html>"#);
        let data = json_ld_info(&document);
        assert_eq!(data.authors, None);
        assert_eq!(data.genres, None);
        assert_eq!(data.status, Status::Unknown);
    }

    #[aidoku_test]
    fn ld_script_selector_finds_script() {
        let document = parse(
            r#"<html><head><script type="application/ld+json">{"author":{"name":"A"}}</script></head></html>"#,
        );
        let script = document
            .select_first("script[type='application/ld+json']")
            .expect("script");
        let value = script
            .html()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok());
        assert!(value.is_some(), "json parses");
        let author = value
            .as_ref()
            .and_then(|v| v.get("author").and_then(ld_names));
        assert_eq!(author, Some(vec![String::from("A")]));
    }

    #[aidoku_test]
    fn chapters_map_from_rows() {
        let document = parse(
            r#"<div id="chapter-list">
				<div data-chapter-number="1189" class="flex">
					<a href="https://natsu.one/manga/one-piece/chapter-1189.391109/">
						<span>Chapter 1189</span>
						<time datetime="2026-07-25T01:56:20Z">8 days ago</time>
					</a>
				</div>
				<div data-chapter-number="1188" class="flex">
					<a href="https://natsu.one/manga/one-piece/chapter-1188.391099/">
						<span>Chapter 5.5</span>
						<time datetime="2026-07-24T01:00:00Z"></time>
					</a>
				</div>
			</div>"#,
        );
        let chapters = chapters_from_document(&document, BASE);
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].key, "1189.391109");
        assert_eq!(chapters[0].chapter_number, Some(1189.0));
        assert_eq!(
            chapters[0].date_uploaded,
            utils::iso8601_seconds("2026-07-25T01:56:20Z")
        );
        assert_eq!(chapters[1].chapter_number, Some(5.5));
        assert_eq!(
            chapters[1].url.as_deref(),
            Some("https://natsu.one/manga/one-piece/chapter-1188.391099/")
        );
    }

    #[aidoku_test]
    fn pages_read_from_reader_sections() {
        let document = parse(
            r#"<section data-image-data="1"><img src="https://cdn.natsu.id/img/o/one-piece/0/1.jpg"><img src="https://cdn.natsu.id/img/o/one-piece/0/2.jpg"><img src="https://cdn.natsu.id/img/o/one-piece/0/3.jpg"></section>"#,
        );
        let pages = pages_from_document(&document);
        assert_eq!(pages.len(), 3);
        assert_eq!(
            pages[0].content,
            PageContent::Url(
                String::from("https://cdn.natsu.id/img/o/one-piece/0/1.jpg"),
                None
            )
        );
        assert_eq!(
            pages[2].content,
            PageContent::Url(
                String::from("https://cdn.natsu.id/img/o/one-piece/0/3.jpg"),
                None
            )
        );
    }

    #[aidoku_test]
    fn pages_fall_back_to_og_image() {
        let document = parse(
            r#"<meta property="og:image" content="https://cdn.natsu.id/img/o/one-piece/0/1.jpg">"#,
        );
        let pages = pages_from_document(&document);
        assert_eq!(pages.len(), 1);
    }

    #[aidoku_test]
    fn nonce_extracted_from_hx_post() {
        assert_eq!(
            extract_nonce(
                "https://natsu.one/wp-admin/admin-ajax.php?nonce=e6d65736eb&action=search"
            ),
            Some(String::from("e6d65736eb"))
        );
        assert_eq!(extract_nonce("no nonce here"), None);
    }

    #[aidoku_test]
    fn status_and_viewer_helpers() {
        assert_eq!(status_from_str(Some("Ongoing")), Status::Ongoing);
        assert_eq!(status_from_str(Some("Completed")), Status::Completed);
        assert_eq!(status_from_str(Some("Hiatus")), Status::Hiatus);
        assert_eq!(status_from_str(Some("Cancelled")), Status::Cancelled);
        assert_eq!(status_from_str(Some("mystery")), Status::Unknown);
        assert_eq!(viewer_from_format(Some("manhwa")), Viewer::Webtoon);
        assert_eq!(viewer_from_format(Some("manga")), Viewer::RightToLeft);
        assert_eq!(viewer_from_format(None), Viewer::Unknown);
    }

    #[aidoku_test]
    fn badge_viewer_reads_alt_then_src() {
        let by_alt = parse(
			r#"<img src="https://natsu.one/wp-content/themes/natsu_id/static/svg/manhua.svg" alt="manhua">"#,
		)
		.select_first("img")
		.expect("badge");
        assert_eq!(viewer_from_badge(&by_alt), Viewer::Webtoon);

        let by_src = parse(
			r#"<img src="https://natsu.one/wp-content/themes/natsu_id/static/svg/manga.svg" alt="">"#,
		)
		.select_first("img")
		.expect("badge");
        assert_eq!(viewer_from_badge(&by_src), Viewer::RightToLeft);
    }

    #[aidoku_test]
    fn mangastatus_of_listing_card() {
        let document = parse(
            r#"<div id="search-results"><div><a href="https://natsu.one/manga/a/"><h1>A</h1></a>
			<div><p class="font-normal text-xs">Ongoing</p></div></div></div>"#,
        );
        let entries = manga_info_from_listing(&document, BASE);
        assert_eq!(entries[0].status.to_aidoku(), MangaStatus::Ongoing);
    }
}
