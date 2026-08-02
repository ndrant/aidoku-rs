use aidoku::alloc::string::ToString;
use aidoku::alloc::{String, Vec, vec};
use aidoku::prelude::*;
use aidoku::serde::Deserialize;
use aidoku::{Page, PageContent, Viewer};

use crate::models::{ChapterInfo, MangaInfo, Status};
use crate::utils;

/// Response envelope for `GET /series` (latest and search listings).
#[derive(Deserialize)]
pub struct SeriesListResponse {
    pub data: Vec<SeriesItem>,
    #[serde(default)]
    pub meta: Option<Meta>,
}

/// Pagination metadata returned alongside series listings.
#[derive(Deserialize)]
pub struct Meta {
    #[serde(rename = "lastPage")]
    pub last_page: i32,
}

/// A series item in a listing or detail response.
#[derive(Deserialize)]
pub struct SeriesItem {
    pub data: SeriesData,
}

/// The inner `data` object of a series item.
#[derive(Deserialize)]
pub struct SeriesData {
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub synopsis: Option<String>,
    #[serde(default, rename = "coverImage")]
    pub cover_image: Option<String>,
    #[serde(default)]
    pub genres: Option<Vec<GenreRef>>,
}

/// A genre reference, resolving through its `data` object to a name.
#[derive(Deserialize)]
pub struct GenreRef {
    pub data: GenreData,
}

#[derive(Deserialize)]
pub struct GenreData {
    pub name: String,
}

/// Response envelope for `GET /series/:slug`.
#[derive(Deserialize)]
pub struct DetailResponse {
    pub data: SeriesItem,
}

/// Response envelope for `GET /series/:slug/chapters`.
#[derive(Deserialize)]
pub struct ChapterListResponse {
    pub data: Vec<ChapterItem>,
}

/// A chapter item in the chapter list.
#[derive(Deserialize)]
pub struct ChapterItem {
    pub data: ChapterData,
    #[serde(default, rename = "createdAt")]
    pub created_at: Option<String>,
}

/// The inner `data` object of a chapter item.
#[derive(Deserialize)]
pub struct ChapterData {
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    pub index: f64,
}

/// Response envelope for `GET /series/:slug/chapters/:index`.
#[derive(Deserialize)]
pub struct ReaderResponse {
    pub data: ReaderData,
}

#[derive(Deserialize)]
pub struct ReaderData {
    pub data: ReaderInner,
}

#[derive(Deserialize)]
pub struct ReaderInner {
    pub images: Vec<String>,
}

/// Maps a series item to a [MangaInfo], skipping entries without a usable
/// slug or title.
pub fn manga_info_from_item(item: &SeriesItem, base_url: &str) -> Option<MangaInfo> {
    let data = &item.data;
    if data.slug.is_empty() || data.title.is_empty() {
        return None;
    }
    let tags = data.genres.as_ref().map(|genres| {
        genres
            .iter()
            .filter_map(|genre| {
                let name = genre.data.name.trim();
                if name.is_empty() {
                    None
                } else {
                    Some(String::from(name))
                }
            })
            .collect()
    });
    Some(MangaInfo {
        key: data.slug.clone(),
        title: data.title.clone(),
        cover: data.cover_image.clone().filter(|cover| !cover.is_empty()),
        authors: data.author.as_ref().map(|author| vec![author.clone()]),
        description: data
            .synopsis
            .clone()
            .filter(|synopsis| !synopsis.is_empty()),
        url: Some(format!("{base_url}/series/{}", data.slug)),
        tags,
        status: status_from_str(data.status.as_deref()),
        viewer: viewer_from_format(data.format.as_deref()),
        ..MangaInfo::default()
    })
}

/// Maps a chapter item to a [ChapterInfo]. The chapter key matches the
/// identifier the reader endpoint accepts: the chapter slug when present,
/// otherwise the numeric index.
pub fn chapter_info_from_item(
    item: &ChapterItem,
    manga_slug: &str,
    base_url: &str,
) -> Option<ChapterInfo> {
    let data = &item.data;
    let key = data
        .slug
        .as_ref()
        .filter(|slug| !slug.is_empty())
        .cloned()
        .unwrap_or_else(|| data.index.to_string());
    let url = format!("{base_url}/series/{manga_slug}/chapter/{key}");
    Some(ChapterInfo {
        key,
        title: data.title.clone().filter(|title| !title.is_empty()),
        chapter_number: Some(data.index as f32),
        date_uploaded: item.created_at.as_deref().and_then(utils::iso8601_seconds),
        url: Some(url),
        ..ChapterInfo::default()
    })
}

/// Maps a reader response to page image URLs.
pub fn pages_from_reader(reader: &ReaderResponse) -> Vec<Page> {
    reader
        .data
        .data
        .images
        .iter()
        .map(|url| Page {
            content: PageContent::url(url),
            ..Page::default()
        })
        .collect()
}

/// Maps a KomikCast status string to a [Status].
pub fn status_from_str(status: Option<&str>) -> Status {
    match status {
        Some("ongoing") => Status::Ongoing,
        Some("completed") => Status::Completed,
        Some("cancelled") | Some("canceled") => Status::Cancelled,
        Some("hiatus") => Status::Hiatus,
        _ => Status::Unknown,
    }
}

/// Maps a KomikCast format string to an Aidoku [Viewer].
pub fn viewer_from_format(format: Option<&str>) -> Viewer {
    match format {
        Some("manhwa") | Some("manhua") | Some("webtoon") => Viewer::Webtoon,
        Some("manga") => Viewer::RightToLeft,
        _ => Viewer::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidoku::MangaStatus;
    use aidoku_test::aidoku_test;

    const BASE: &str = "https://v3.komikcast.fit";

    fn parse<'a, T: Deserialize<'a>>(json: &'a str) -> T {
        serde_json::from_str(json).expect("fixture must deserialize")
    }

    #[aidoku_test]
    fn list_item_maps_to_manga_info() {
        let item: SeriesItem = parse(
            r#"{
			"data": {
				"slug": "human-table",
				"title": "Human Table",
				"author": "Kim Gyusam",
				"format": "manhwa",
				"status": "ongoing",
				"synopsis": "A survival story.",
				"coverImage": "https://minio/c/human.webp",
				"nativeTitle": "The Human Table",
				"genres": [{ "data": { "name": "Action" } }, { "data": { "name": "Horror" } }]
			}
		}"#,
        );
        let info = manga_info_from_item(&item, BASE).expect("valid item");
        assert_eq!(info.key, "human-table");
        assert_eq!(info.title, "Human Table");
        assert_eq!(info.authors, Some(vec![String::from("Kim Gyusam")]));
        assert_eq!(
            info.url,
            Some(String::from("https://v3.komikcast.fit/series/human-table"))
        );
        assert_eq!(
            info.tags,
            Some(vec![String::from("Action"), String::from("Horror")])
        );
        assert_eq!(info.status, Status::Ongoing);
        assert_eq!(info.viewer, Viewer::Webtoon);
        let manga = info.into_aidoku();
        assert_eq!(manga.status, MangaStatus::Ongoing);
    }

    #[aidoku_test]
    fn list_item_with_empty_slug_is_skipped() {
        let item: SeriesItem = parse(r#"{"data":{"slug":"","title":"Title"}}"#);
        assert!(manga_info_from_item(&item, BASE).is_none());
    }

    #[aidoku_test]
    fn chapter_item_maps_to_chapter_info() {
        let item: ChapterItem = parse(
            r#"{
			"data": { "slug": null, "title": null, "index": 5, "seriesId": 10332 },
			"createdAt": "2026-08-02T10:19:22.546+07:00"
		}"#,
        );
        let info = chapter_info_from_item(&item, "human-table", BASE).expect("valid item");
        assert_eq!(info.key, "5");
        assert_eq!(info.chapter_number, Some(5.0));
        assert_eq!(info.date_uploaded, Some(1785640762));
    }

    #[aidoku_test]
    fn chapter_item_uses_slug_key_when_present() {
        let item: ChapterItem = parse(
            r#"{
			"data": { "slug": "chapter-005", "title": "Side Story", "index": 5 },
			"createdAt": null
		}"#,
        );
        let info = chapter_info_from_item(&item, "human-table", BASE).expect("valid item");
        assert_eq!(info.key, "chapter-005");
        assert_eq!(info.title, Some(String::from("Side Story")));
        assert_eq!(info.date_uploaded, None);
    }

    #[aidoku_test]
    fn reader_maps_to_pages() {
        let reader: ReaderResponse = parse(
            r#"{
			"data": { "data": {
				"images": ["https://cdn/a/1.jpg", "https://cdn/a/2.jpg"]
			} }
		}"#,
        );
        let pages = pages_from_reader(&reader);
        assert_eq!(pages.len(), 2);
        assert_eq!(
            pages[0].content,
            PageContent::Url(String::from("https://cdn/a/1.jpg"), None)
        );
    }

    #[aidoku_test]
    fn status_and_viewer_helpers() {
        assert_eq!(status_from_str(Some("ongoing")), Status::Ongoing);
        assert_eq!(status_from_str(Some("completed")), Status::Completed);
        assert_eq!(status_from_str(Some("cancelled")), Status::Cancelled);
        assert_eq!(status_from_str(Some("hiatus")), Status::Hiatus);
        assert_eq!(status_from_str(Some("unknown-thing")), Status::Unknown);
        assert_eq!(viewer_from_format(Some("manhwa")), Viewer::Webtoon);
        assert_eq!(viewer_from_format(Some("manhua")), Viewer::Webtoon);
        assert_eq!(viewer_from_format(Some("manga")), Viewer::RightToLeft);
        assert_eq!(viewer_from_format(None), Viewer::Unknown);
    }
}
