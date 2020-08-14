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
    #[serde(rename(deserialize = "id"))]
    pub media_id: i32,
    pub status: MediaStatus,
    pub title: MediaTitle,
    pub synonyms: Vec<String>,
    pub chapters: Option<i32>,
    pub volumes: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaList {
    pub id: i32,
    pub status: MediaListStatus,
    pub progress: Option<i32>,
    pub progress_volumes: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaType {
    Anime,
    Manga,
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
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaStatus {
    Finished,
    Releasing,
    #[serde(rename = "NOT_YET_RELEASED")]
    NotYetReleased,
    Cancelled,
}

#[derive(Deserialize, Debug)]
pub struct QueryError {
    pub message: Option<String>,
    pub status: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct QueryResponse<R> {
    pub data: Option<R>,
    pub errors: Option<Vec<QueryError>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ViewerResponse {
    pub viewer: User,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MediaResponse {
    pub media: Media,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MediaListResponse {
    pub media_list: MediaList,
}
