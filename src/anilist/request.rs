use anyhow::{anyhow, Result};
use log::{debug, error, info};
use reqwest::{blocking::Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Map, Value};
use std::{thread, time};

use super::model::{Media, MediaList, MediaListStatus, MediaStatus, MediaType, User};
use super::query::{QUERY_MEDIA_LIST, QUERY_USER, SEARCH_MEDIA};
use crate::util::MendoConfig;
use crate::PROGRAM_NAME;

const ANILIST_API_URL: &str = "https://graphql.anilist.co";

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

pub fn query_graphql<R>(
    query_str: &str,
    variables: &Option<Map<String, Value>>,
    cfg: &mut MendoConfig,
) -> Result<QueryResponse<R>>
where
    R: DeserializeOwned + std::fmt::Debug,
    // Need DeserializeOwned because of how reqwest deserializes the response.
{
    let query = if let Some(vars) = &variables {
        json!({ "query": query_str, "variables": vars })
    } else {
        json!({ "query": query_str })
    };

    let token = &cfg.token;
    let local_rate_limit_count: u8 = 3;

    for _ in 0..local_rate_limit_count {
        let client = Client::new();
        debug!("Sending POST request with query = \n{:#?}", query);
        let res = client
            .post(ANILIST_API_URL)
            .header("ContentType", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .json(&query)
            .send()?;

        let res_status = res.status();

        match res_status {
            StatusCode::TOO_MANY_REQUESTS => {
                debug!("Anilist returned code `{}'", res_status);
                let secs;
                let retry = res.headers().get("Retry-After");
                if let Some(val) = retry {
                    let header = String::from_utf8_lossy(val.as_bytes());
                    secs = header.parse::<u64>().unwrap_or(60);
                } else {
                    secs = 60;
                }
                thread::sleep(time::Duration::from_secs(secs));
            }
            StatusCode::UNAUTHORIZED => {
                error!("Anilist returned code `{}'. Unauthorized!", res_status);
                let response: QueryResponse<R> = res.json()?;
                debug!("Response =\n{:#?}", response);
                debug!("Deleting the existing token to force user to reauth...");
                confy::store(
                    PROGRAM_NAME,
                    MendoConfig {
                        id: cfg.id,
                        secret: cfg.secret.to_string(),
                        name: cfg.name.to_string(),
                        url: cfg.url.to_string(),
                        token: "Leave this field.".to_string(),
                    },
                )?;
                return Err(anyhow!("Unthorized! Run the program again to reauthorize!"));
            }
            StatusCode::OK => {
                info!("Anilist returned `{}'!", res.status());
                let response: QueryResponse<R> = res.json()?;
                debug!("Response =\n{:#?}", response);
                return Ok(response);
            }
            _ => {
                error!("Anilist returned an unimplemented code `{}'!", res_status);
                let response: QueryResponse<R> = res.json()?;
                debug!("Response =\n{:#?}", response);
                return Err(anyhow!("Anilist returned an unimplemented response code!"));
            }
        }
    }

    error!(
        "Exceeded the local rate limit count ({})",
        local_rate_limit_count
    );
    Err(anyhow!(
        "Exceeded the rate limit count ({})",
        local_rate_limit_count
    ))
}

pub fn query_user(cfg: &mut MendoConfig) -> Result<QueryResponse<ViewerResponse>> {
    info!("Querying currently authenticated user...");
    query_graphql(QUERY_USER, &None, cfg)
}

pub fn search_media(
    cfg: &mut MendoConfig,
    search_string: &str,
    media_type: MediaType,
) -> Result<QueryResponse<MediaResponse>> {
    let variables = json!({
        "search": search_string,
        "type": media_type,
        "status_not": MediaStatus::NotYetReleased,
    });

    if let serde_json::Value::Object(variables) = variables {
        info!(
            "Searching Media using name: {}, type: {:?}...",
            search_string, media_type
        );
        query_graphql(SEARCH_MEDIA, &Some(variables), cfg)
    } else {
        error!("Media list query variables is not a json object");
        Err(anyhow!("Media list query variables is not a json object"))
    }
}

pub fn query_media_list(
    cfg: &mut MendoConfig,
    user_id: i32,
    media_id: i32,
    media_type: MediaType,
) -> Result<QueryResponse<MediaListResponse>> {
    let variables = json!({
        "userId": user_id,
        "mediaId": media_id,
        "type": media_type,
        "status_not": MediaListStatus::Dropped,
    });

    if let serde_json::Value::Object(variables) = variables {
        info!(
            "Querying MediaList for progress using media ID: {}, type: {:?} of user...",
            media_id, media_type
        );
        query_graphql(QUERY_MEDIA_LIST, &Some(variables), cfg)
    } else {
        error!("Media list query variables is not a json object");
        Err(anyhow!("Media list query variables is not a json object"))
    }
}
