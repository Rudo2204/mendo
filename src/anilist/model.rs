use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    name: String,
    site_url: String,
    updated_at: i64, // unix timestamp
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
    pub user_preferred: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Media {
    pub title: Option<MediaTitle>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaListStatus {
    Current,
    Planning,
    Completed,
    Dropped,
    Paused,
    Repeating,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaList {
    pub id: i32,
    media_id: i32,
    pub status: Option<MediaListStatus>,
    pub progress: i32,
    pub progress_volumes: i32,
    pub media: Option<Media>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaListGroup {
    pub name: Option<String>,
    pub status: Option<MediaListStatus>,
    pub is_custom_list: Option<bool>,
    pub entries: Option<Vec<Option<MediaList>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaListCollection {
    pub lists: Option<Vec<Option<MediaListGroup>>>,
}
