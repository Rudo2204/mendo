use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use yaml_rust::{YamlEmitter, YamlLoader};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    name: String,
    site_url: String,
    updated_at: i64, // unix timestamp
}

impl User {
    pub fn dump_user_info(&self, path: &PathBuf) -> Result<()> {
        let s = serde_yaml::to_string(&self)?;
        let docs = YamlLoader::load_from_str(&s)?;
        let doc = &docs[0];
        let mut out_str = String::new();
        {
            let mut emitter = YamlEmitter::new(&mut out_str);
            emitter.dump(doc)?;
        }
        let mut output = File::create(path)?;
        write!(output, "{}", out_str)?;
        debug!("User profile dumped to {}", path.display());
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: String,
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

impl Media {
    pub fn append_local_data(&self, path: &PathBuf) -> Result<()> {
        let mut file = OpenOptions::new().append(true).open(&path)?;
        let data = format!(
            "{} - mediaId: {}",
            &self.title.english.as_ref().unwrap(),
            self.media_id
        );
        writeln!(file, "{}", data)?;
        debug!(
            "Appended received data to local media data at {}",
            &path.display()
        );
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaList {
    #[serde(rename(deserialize = "id"))]
    pub entry_id: i32,
    pub status: MediaListStatus,
    pub progress: i32,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SaveMediaListEntry {
    pub id: Option<i32>,
    pub media_id: Option<i32>,
    pub status: Option<MediaListStatus>,
    pub progress: Option<i32>,
}
