use aidoku::alloc::{String, Vec};
use aidoku::prelude::*;
use aidoku::serde::Deserialize;
use aidoku::{Page, PageContent, Viewer};

use crate::models::{ChapterInfo, MangaInfo, Status};
use crate::utils;

/// Pagination metadata returned alongside every list endpoint.
#[derive(Deserialize)]
pub struct ListMeta {
    #[serde(default, rename = "total_page")]
    pub total_page: Option<i32>,
}

/// Response envelope for `GET /manga/list` (search, latest and popular).
#[derive(Deserialize)]
pub struct MangaListResponse {
    pub data: Vec<MangaItem>,
    pub meta: ListMeta,
}

/// A manga item in a listing, search or detail response.
#[derive(Deserialize)]
pub struct MangaItem {
    #[serde(rename = "manga_id")]
    pub manga_id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "cover_image_url")]
    pub cover_image_url: Option<String>,
    #[serde(default, rename = "cover_portrait_url")]
    pub cover_portrait_url: Option<String>,
    #[serde(default)]
    pub status: Option<i64>,
    #[serde(default)]
    pub taxonomy: Option<Taxonomy>,
}

/// Response envelope for `GET /manga/detail/:id`.
#[derive(Deserialize)]
pub struct DetailResponse {
    pub data: MangaItem,
}

/// The taxonomy object groups role names, keyed by their capitalized role.
#[derive(Deserialize)]
pub struct Taxonomy {
    #[serde(default, rename = "Artist")]
    pub artists: Vec<TaxonomyItem>,
    #[serde(default, rename = "Author")]
    pub authors: Vec<TaxonomyItem>,
    #[serde(default, rename = "Format")]
    pub formats: Vec<TaxonomyItem>,
    #[serde(default, rename = "Genre")]
    pub genres: Vec<TaxonomyItem>,
}

/// A named entry inside a taxonomy group.
#[derive(Deserialize)]
pub struct TaxonomyItem {
    pub name: String,
}

/// Response envelope for `GET /chapter/:manga_id/list`.
#[derive(Deserialize)]
pub struct ChapterListResponse {
    pub data: Vec<ChapterItem>,
    pub meta: ListMeta,
}

/// A chapter item in the chapter list.
#[derive(Deserialize)]
pub struct ChapterItem {
    #[serde(rename = "chapter_id")]
    pub chapter_id: String,
    #[serde(default, rename = "chapter_title")]
    pub chapter_title: Option<String>,
    #[serde(default, rename = "chapter_number")]
    pub chapter_number: Option<f64>,
    #[serde(default, rename = "release_date")]
    pub release_date: Option<String>,
}

/// Response envelope for `GET /chapter/detail/:id`.
#[derive(Deserialize)]
pub struct ChapterDetailResponse {
    pub data: ChapterDetail,
}

/// The chapter detail used to build page image URLs.
#[derive(Deserialize)]
pub struct ChapterDetail {
    #[serde(default, rename = "base_url")]
    pub base_url: Option<String>,
    #[serde(default)]
    pub chapter: Option<ChapterFiles>,
}

/// The `chapter` field of a chapter detail: a path prefix and page files.
#[derive(Deserialize)]
pub struct ChapterFiles {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub data: Vec<String>,
}

/// Maps a manga item to a [MangaInfo], skipping entries without an id.
pub fn manga_info_from_item(item: &MangaItem, base_url: &str) -> Option<MangaInfo> {
    if item.manga_id.is_empty() || item.title.is_empty() {
        return None;
    }
    let taxonomy = item.taxonomy.as_ref();
    Some(MangaInfo {
        key: item.manga_id.clone(),
        title: item.title.clone(),
        cover: item
            .cover_image_url
            .clone()
            .or_else(|| item.cover_portrait_url.clone())
            .filter(|cover| !cover.is_empty()),
        authors: taxonomy.and_then(|t| names(&t.authors)),
        artists: taxonomy.and_then(|t| names(&t.artists)),
        description: item
            .description
            .clone()
            .filter(|description| !description.is_empty()),
        url: Some(format!("{base_url}/series/{}", item.manga_id)),
        tags: taxonomy.and_then(|t| names(&t.genres)),
        status: status_from_num(item.status),
        viewer: taxonomy
            .and_then(|t| viewer_from_formats(&t.formats))
            .unwrap_or(Viewer::Unknown),
        ..MangaInfo::default()
    })
}

/// Collects the non-empty names of a taxonomy group.
fn names(items: &[TaxonomyItem]) -> Option<Vec<String>> {
    let names = items
        .iter()
        .map(|item| item.name.trim())
        .filter(|name| !name.is_empty())
        .map(String::from)
        .collect::<Vec<_>>();
    if names.is_empty() { None } else { Some(names) }
}

/// Maps the first recognized format to an Aidoku [Viewer].
pub fn viewer_from_formats(formats: &[TaxonomyItem]) -> Option<Viewer> {
    for format in formats {
        let format = format.name.trim().to_lowercase();
        match format.as_str() {
            "manhwa" | "manhua" | "webtoon" => return Some(Viewer::Webtoon),
            "manga" => return Some(Viewer::RightToLeft),
            _ => {}
        }
    }
    None
}

/// Maps a numeric Shinigami status to a [Status]: 1 ongoing, 2 completed,
/// 3 hiatus.
pub fn status_from_num(status: Option<i64>) -> Status {
    match status {
        Some(1) => Status::Ongoing,
        Some(2) => Status::Completed,
        Some(3) => Status::Hiatus,
        _ => Status::Unknown,
    }
}

/// Maps a chapter item to a [ChapterInfo]. The chapter key matches the id the
/// reader endpoint accepts.
pub fn chapter_info_from_item(item: &ChapterItem, base_url: &str) -> Option<ChapterInfo> {
    if item.chapter_id.is_empty() {
        return None;
    }
    Some(ChapterInfo {
        key: item.chapter_id.clone(),
        title: item.chapter_title.clone().filter(|title| !title.is_empty()),
        chapter_number: item.chapter_number.map(|number| number as f32),
        date_uploaded: item
            .release_date
            .as_deref()
            .and_then(utils::iso8601_seconds),
        url: Some(format!("{base_url}/chapter/{}", item.chapter_id)),
    })
}

/// Builds page image URLs from a chapter detail: `base_url` + `path` + file.
pub fn pages_from_detail(detail: &ChapterDetail) -> Vec<Page> {
    let mut pages = Vec::new();
    if let Some(files) = detail.chapter.as_ref() {
        if let Some(base_url) = detail.base_url.as_deref() {
            let prefix = format!("{base_url}{}", files.path.as_deref().unwrap_or(""));
            for file in &files.data {
                pages.push(Page {
                    content: PageContent::url(&format!("{prefix}{file}")),
                    ..Page::default()
                });
            }
        }
    }
    pages
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidoku::MangaStatus;
    use aidoku::alloc::vec;
    use aidoku_test::aidoku_test;

    const BASE: &str = "https://shinigami.asia";

    fn parse<'a, T: Deserialize<'a>>(json: &'a str) -> T {
        serde_json::from_str(json).expect("fixture must deserialize")
    }

    #[aidoku_test]
    fn manga_item_maps_to_manga_info() {
        let item: MangaItem = parse(
            r#"{
			"manga_id": "c0f1d049-ff7f-474d-8c6a-3a55e4c44147",
			"title": "Demonic Emperor",
			"description": "A story.",
			"cover_image_url": "https://assets.shngm.id/thumbnail/image/a.jpg",
			"cover_portrait_url": "https://assets.shngm.id/thumbnail/image/b.jpg",
			"status": 1,
			"taxonomy": {
				"Artist": [{ "name": "Wuer Manhua" }],
				"Author": [{ "name": "Wuer Manhua" }, { "name": "Ye Xiao" }],
				"Format": [{ "name": "Manhua" }],
				"Genre": [{ "name": "Action" }, { "name": "Fantasy" }],
				"Type": [{ "name": "Project" }]
			}
		}"#,
        );
        let info = manga_info_from_item(&item, BASE).expect("valid item");
        assert_eq!(info.key, "c0f1d049-ff7f-474d-8c6a-3a55e4c44147");
        assert_eq!(info.title, "Demonic Emperor");
        assert_eq!(
            info.cover.as_deref(),
            Some("https://assets.shngm.id/thumbnail/image/a.jpg")
        );
        assert_eq!(
            info.authors,
            Some(vec![String::from("Wuer Manhua"), String::from("Ye Xiao")])
        );
        assert_eq!(info.artists, Some(vec![String::from("Wuer Manhua")]));
        assert_eq!(
            info.tags,
            Some(vec![String::from("Action"), String::from("Fantasy")])
        );
        assert_eq!(info.status, Status::Ongoing);
        assert_eq!(info.viewer, Viewer::Webtoon);
        assert_eq!(
            info.url,
            Some(String::from(
                "https://shinigami.asia/series/c0f1d049-ff7f-474d-8c6a-3a55e4c44147"
            ))
        );
        let manga = info.into_aidoku();
        assert_eq!(manga.status, MangaStatus::Ongoing);
    }

    #[aidoku_test]
    fn manga_item_empty_id_is_skipped() {
        let item: MangaItem = parse(r#"{"manga_id": "", "title": "Title"}"#);
        assert!(manga_info_from_item(&item, BASE).is_none());
    }

    #[aidoku_test]
    fn manga_item_cover_falls_back_to_portrait() {
        let item: MangaItem = parse(
            r#"{"manga_id": "id-1", "title": "T", "cover_portrait_url": "https://assets.shngm.id/thumbnail/image/b.jpg"}"#,
        );
        let info = manga_info_from_item(&item, BASE).expect("valid item");
        assert_eq!(
            info.cover.as_deref(),
            Some("https://assets.shngm.id/thumbnail/image/b.jpg")
        );
        assert_eq!(info.authors, None);
        assert_eq!(info.status, Status::Unknown);
        assert_eq!(info.viewer, Viewer::Unknown);
    }

    #[aidoku_test]
    fn chapter_item_maps_to_chapter_info() {
        let item: ChapterItem = parse(
            r#"{
			"chapter_id": "825d0326-3dfe-47c8-8468-153725735068",
			"chapter_title": "",
			"chapter_number": 16.1,
			"release_date": "2026-08-02T00:24:32Z"
		}"#,
        );
        let info = chapter_info_from_item(&item, BASE).expect("valid item");
        assert_eq!(info.key, "825d0326-3dfe-47c8-8468-153725735068");
        assert_eq!(info.title, None);
        assert_eq!(info.chapter_number, Some(16.1));
        assert_eq!(
            info.date_uploaded,
            utils::iso8601_seconds("2026-08-02T00:24:32Z")
        );
        assert_eq!(
            info.url.as_deref(),
            Some("https://shinigami.asia/chapter/825d0326-3dfe-47c8-8468-153725735068")
        );
    }

    #[aidoku_test]
    fn chapter_item_empty_id_is_skipped() {
        let item: ChapterItem = parse(r#"{"chapter_id": ""}"#);
        assert!(chapter_info_from_item(&item, BASE).is_none());
    }

    #[aidoku_test]
    fn pages_built_from_base_path_and_files() {
        let detail: ChapterDetail = parse(
            r#"{
			"base_url": "https://assets.shngm.id",
			"chapter": {
				"path": "/chapter/manga_c0f1d049-ff7f-474d-8c6a-3a55e4c44147/chapter_825d0326-3dfe-47c8-8468-153725735068/",
				"data": ["00-46fb69.jpg", "01-0edd23.jpg", "02-81172c.jpg"]
			}
		}"#,
        );
        let pages = pages_from_detail(&detail);
        assert_eq!(pages.len(), 3);
        assert_eq!(
            pages[0].content,
            PageContent::Url(
                String::from(
                    "https://assets.shngm.id/chapter/manga_c0f1d049-ff7f-474d-8c6a-3a55e4c44147/chapter_825d0326-3dfe-47c8-8468-153725735068/00-46fb69.jpg"
                ),
                None
            )
        );
        assert_eq!(
            pages[2].content,
            PageContent::Url(
                String::from(
                    "https://assets.shngm.id/chapter/manga_c0f1d049-ff7f-474d-8c6a-3a55e4c44147/chapter_825d0326-3dfe-47c8-8468-153725735068/02-81172c.jpg"
                ),
                None
            )
        );
    }

    #[aidoku_test]
    fn pages_empty_without_files() {
        let detail: ChapterDetail = parse(r#"{"base_url": "https://assets.shngm.id"}"#);
        assert!(pages_from_detail(&detail).is_empty());
    }

    #[aidoku_test]
    fn status_mapping() {
        assert_eq!(status_from_num(Some(1)), Status::Ongoing);
        assert_eq!(status_from_num(Some(2)), Status::Completed);
        assert_eq!(status_from_num(Some(3)), Status::Hiatus);
        assert_eq!(status_from_num(Some(9)), Status::Unknown);
        assert_eq!(status_from_num(None), Status::Unknown);
    }

    #[aidoku_test]
    fn viewer_from_formats_recognizes_webtoon() {
        let formats: Vec<TaxonomyItem> = parse(r#"[{"name": "Manhua"}, {"name": "Manga"}]"#);
        assert_eq!(viewer_from_formats(&formats), Some(Viewer::Webtoon));
        let formats: Vec<TaxonomyItem> = parse(r#"[{"name": "Manga"}]"#);
        assert_eq!(viewer_from_formats(&formats), Some(Viewer::RightToLeft));
        let formats: Vec<TaxonomyItem> = parse(r#"[{"name": "Light Novel"}]"#);
        assert_eq!(viewer_from_formats(&formats), None);
    }
}
