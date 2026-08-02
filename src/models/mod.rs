use aidoku::alloc::{String, Vec};
use aidoku::{Chapter, ContentRating, Manga, MangaStatus, Viewer};

/// Publishing status of a manga, independent of any Aidoku type.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    #[default]
    Unknown,
    Ongoing,
    Completed,
    Cancelled,
    Hiatus,
}

impl Status {
    pub fn to_aidoku(self) -> MangaStatus {
        match self {
            Status::Unknown => MangaStatus::Unknown,
            Status::Ongoing => MangaStatus::Ongoing,
            Status::Completed => MangaStatus::Completed,
            Status::Cancelled => MangaStatus::Cancelled,
            Status::Hiatus => MangaStatus::Hiatus,
        }
    }
}

/// Parsed manga data before it is mapped to an Aidoku [Manga].
#[derive(Default, Clone, Debug, PartialEq)]
pub struct MangaInfo {
    pub key: String,
    pub title: String,
    pub cover: Option<String>,
    pub authors: Option<Vec<String>>,
    pub artists: Option<Vec<String>>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Status,
    pub viewer: Viewer,
}

impl MangaInfo {
    pub fn into_aidoku(self) -> Manga {
        Manga {
            key: self.key,
            title: self.title,
            cover: self.cover,
            artists: self.artists,
            authors: self.authors,
            description: self.description,
            url: self.url,
            tags: self.tags,
            status: self.status.to_aidoku(),
            content_rating: ContentRating::Safe,
            viewer: self.viewer,
            ..Manga::default()
        }
    }
}

/// Parsed chapter data before it is mapped to an Aidoku [Chapter].
#[derive(Default, Clone, Debug, PartialEq)]
pub struct ChapterInfo {
    pub key: String,
    pub title: Option<String>,
    pub chapter_number: Option<f32>,
    pub date_uploaded: Option<i64>,
    pub url: Option<String>,
}

impl ChapterInfo {
    pub fn into_aidoku(self) -> Chapter {
        Chapter {
            key: self.key,
            title: self.title,
            chapter_number: self.chapter_number,
            date_uploaded: self.date_uploaded,
            url: self.url,
            ..Chapter::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidoku::alloc::vec;
    use aidoku_test::aidoku_test;

    #[aidoku_test]
    fn manga_info_maps_to_aidoku() {
        let info = MangaInfo {
            key: String::from("k"),
            title: String::from("Title"),
            cover: Some(String::from("https://example.com/c.jpg")),
            authors: Some(vec![String::from("Author")]),
            status: Status::Ongoing,
            viewer: Viewer::RightToLeft,
            ..MangaInfo::default()
        };
        let manga = info.into_aidoku();
        assert_eq!(manga.key, "k");
        assert_eq!(manga.title, "Title");
        assert_eq!(manga.status, MangaStatus::Ongoing);
        assert_eq!(manga.viewer, Viewer::RightToLeft);
        assert_eq!(manga.content_rating, ContentRating::Safe);
    }

    #[aidoku_test]
    fn chapter_info_maps_to_aidoku() {
        let info = ChapterInfo {
            key: String::from("ch"),
            title: Some(String::from("Chapter 1")),
            chapter_number: Some(1.0),
            date_uploaded: Some(1700000000),
            url: Some(String::from("https://example.com/ch/1")),
        };
        let chapter = info.into_aidoku();
        assert_eq!(chapter.key, "ch");
        assert_eq!(chapter.chapter_number, Some(1.0));
        assert_eq!(chapter.date_uploaded, Some(1700000000));
    }
}
