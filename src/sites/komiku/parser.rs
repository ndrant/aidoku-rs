use aidoku::alloc::{String, Vec, format, vec};
use aidoku::serde::Deserialize;
use aidoku::{Page, PageContent, Viewer};

use crate::models::{ChapterInfo, MangaInfo, Status};

#[derive(Deserialize, Debug)]
pub struct KomikuComic {
    pub id: i64,
    pub slug: String,
    pub title: String,
    #[serde(rename = "type")]
    pub comic_type: Option<String>,
    pub genres: Option<Vec<String>>,
    pub status: Option<String>,
    pub author: Option<String>,
    pub artist: Option<String>,
    pub synopsis: Option<String>,
    #[serde(rename = "coverUrl")]
    pub cover_url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct KomikuListResponse {
    pub items: Vec<KomikuComic>,
    #[serde(rename = "totalPages")]
    pub total_pages: i32,
}

#[derive(Deserialize, Debug)]
pub struct KomikuChapter {
    pub id: i64,
    pub n: f32,
    pub title: Option<String>,
    #[serde(rename = "releasedAt")]
    pub released_at: Option<i64>,
}

#[derive(Deserialize, Debug)]
pub struct KomikuPageItem {
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct KomikuChapterDetail {
    pub pages: Vec<KomikuPageItem>,
}

/// Converts a [KomikuComic] into a [MangaInfo] record.
pub fn manga_info_from_comic(comic: &KomikuComic, base_url: &str) -> MangaInfo {
    let key = comic.slug.clone();
    let url = format!("{base_url}/manga/{}", comic.slug);
    let status = match comic.status.as_deref().map(str::to_lowercase).as_deref() {
        Some("ongoing") => Status::Ongoing,
        Some("completed") => Status::Completed,
        Some("hiatus") => Status::Hiatus,
        _ => Status::Unknown,
    };
    let viewer = match comic
        .comic_type
        .as_deref()
        .map(str::to_lowercase)
        .as_deref()
    {
        Some("manhwa") | Some("webtoon") | Some("manhua") => Viewer::Webtoon,
        _ => Viewer::RightToLeft,
    };
    let authors = comic
        .author
        .clone()
        .filter(|a| !a.is_empty())
        .map(|a| vec![a]);
    let artists = comic
        .artist
        .clone()
        .filter(|a| !a.is_empty())
        .map(|a| vec![a]);

    MangaInfo {
        key,
        title: comic.title.clone(),
        cover: comic.cover_url.clone(),
        description: comic.synopsis.clone(),
        authors,
        artists,
        tags: comic.genres.clone(),
        status,
        viewer,
        url: Some(url),
    }
}

/// Converts a [KomikuChapter] list into [ChapterInfo] records.
pub fn chapters_from_items(
    chapters: &[KomikuChapter],
    comic_id: i64,
    slug: &str,
    base_url: &str,
) -> Vec<ChapterInfo> {
    chapters
        .iter()
        .map(|ch| {
            // Encode both comicId and chapterId into the key so page_list has both
            let key = format!("{comic_id}:{}", ch.id);
            let url = format!("{base_url}/read/id/{slug}/ch{}-{}", ch.n, ch.id);
            let date_uploaded = ch.released_at.map(|ms| ms / 1000);
            ChapterInfo {
                key,
                title: ch
                    .title
                    .clone()
                    .or_else(|| Some(format!("Chapter {}", ch.n))),
                chapter_number: Some(ch.n),
                date_uploaded,
                url: Some(url),
            }
        })
        .collect()
}

/// Converts a [KomikuChapterDetail] into a vector of [Page] records.
pub fn pages_from_detail(detail: &KomikuChapterDetail) -> Vec<Page> {
    detail
        .pages
        .iter()
        .map(|item| Page {
            content: PageContent::url(&item.url),
            ..Page::default()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidoku_test::aidoku_test;

    #[aidoku_test]
    fn comic_converts_to_manga_info() {
        let comic = KomikuComic {
            id: 85049,
            slug: String::from("nano-machine"),
            title: String::from("Nano Machine"),
            comic_type: Some(String::from("Manhwa")),
            genres: Some(vec![String::from("Action")]),
            status: Some(String::from("Ongoing")),
            author: Some(String::from("Author")),
            artist: None,
            synopsis: Some(String::from("Synopsis")),
            cover_url: Some(String::from("https://cdn.example/cover.jpg")),
        };
        let info = manga_info_from_comic(&comic, "https://01.komiku.asia");
        assert_eq!(info.key, "nano-machine");
        assert_eq!(info.viewer, Viewer::Webtoon);
        assert_eq!(info.status, Status::Ongoing);
    }
}
