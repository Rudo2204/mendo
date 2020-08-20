use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use reqwest::{blocking::Client, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::{json, Map, Value};
use std::{thread, time};

use super::model::{
    MediaListResponse, MediaListStatus, MediaResponse, MediaStatus, MediaType, QueryResponse,
    SaveMediaListEntry, ViewerResponse,
};
use super::query::{QUERY_MEDIA_LIST, QUERY_USER, SEARCH_MEDIA, UPDATE_MEDIA};
use crate::util::MendoConfig;
use crate::PROGRAM_NAME;

const ANILIST_API_URL: &str = "https://graphql.anilist.co";

pub fn query_graphql<R>(
    query_str: &str,
    variables: &Option<Map<String, Value>>,
    cfg: &mut MendoConfig,
    client: &Client,
    use_token: bool,
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
    debug!("Sending POST request with query = \n{:#?}", query);

    let token = &cfg.token;
    let local_rate_limit_count: u8 = 3;

    for i in 0..local_rate_limit_count {
        if i > 0 {
            warn!("Retrying {}...", i);
        }
        let mut cl = client
            .post(ANILIST_API_URL)
            .header("ContentType", "application/json")
            .header("Accept", "application/json");

        if use_token {
            cl = cl.header("Authorization", format!("Bearer {}", token));
        }

        let res = cl.json(&query).send()?;

        let res_status = res.status();

        match res_status {
            StatusCode::TOO_MANY_REQUESTS => {
                debug!("Anilist returned code `{}'", res_status);
                let retry = res.headers().get("Retry-After");
                let secs = if let Some(val) = retry {
                    let header = String::from_utf8_lossy(val.as_bytes());
                    header.parse::<u64>().unwrap_or(60)
                } else {
                    60
                };
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
            // This could happen in two situations:
            // 1. The user has not created entry for this title.
            // 2. The input filename is just garbage, could not search for that title
            StatusCode::NOT_FOUND => {
                warn!("Anilist returned `{}'!", res.status());
                if let Some(vars) = &variables {
                    // This is the 1st situation AKA when using query_media_list
                    if vars.contains_key("mediaId") {
                        debug!("It seems like user has not created entry for this title!");
                        let media_id = vars
                            .get("mediaId")
                            .expect("Safe because of the above check")
                            .as_u64()
                            .expect("Safe because of how mendo defined media_id")
                            as i32;
                        debug!("Got media_id `{}` from variables", media_id);
                        create_new_entry(cfg, media_id, MediaListStatus::Current, 0, client)?;
                        info!("Will now retry to query MediaList...");
                        return Ok(query_graphql(
                            QUERY_MEDIA_LIST,
                            &variables,
                            cfg,
                            &client,
                            false,
                        )?);
                    // This is the 2ns situation AKA when using search_media
                    } else {
                        error!("The API did not return any result! Maybe recheck your archive filename?");
                        return Err(anyhow!("The API did not return any result! Maybe recheck your archive filename?"));
                    }
                }
            }
            StatusCode::OK => {
                info!("Anilist returned `{}'!", res.status());
                let response: QueryResponse<R> = res.json()?;
                debug!("Response =\n{:#?}", response);
                return Ok(response);
            }
            _ => {
                error!("Anilist returned an unimplemented code `{}'!", res_status);
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

pub fn query_user(cfg: &mut MendoConfig, client: &Client) -> Result<QueryResponse<ViewerResponse>> {
    info!("Querying for info of currently authenticated user...");
    query_graphql(QUERY_USER, &None, cfg, &client, true)
}

pub fn search_media(
    cfg: &mut MendoConfig,
    search_string: &str,
    media_type: MediaType,
    client: &Client,
) -> Result<QueryResponse<MediaResponse>> {
    let variables = json!({
        "search": search_string,
        "type": media_type,
        "status_not": MediaStatus::NotYetReleased,
    });

    if let serde_json::Value::Object(variables) = variables {
        info!(
            "Searching Media using name: `{}`, type: `{:?}`...",
            search_string, media_type
        );
        query_graphql(SEARCH_MEDIA, &Some(variables), cfg, &client, false)
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
    client: &Client,
) -> Result<QueryResponse<MediaListResponse>> {
    let variables = json!({
        "userId": user_id,
        "mediaId": media_id,
        "type": media_type,
        "status_not": MediaListStatus::Dropped,
    });

    if let serde_json::Value::Object(variables) = variables {
        info!(
            "Querying MediaList for progress using media ID: `{}`, type: `{:?}` of user...",
            media_id, media_type
        );
        query_graphql(QUERY_MEDIA_LIST, &Some(variables), cfg, &client, false)
    } else {
        error!("Media list query variables is not a json object");
        Err(anyhow!("Media list query variables is not a json object"))
    }
}

pub fn update_media(
    cfg: &mut MendoConfig,
    entry_id: i32,
    progress: i32,
    client: &Client,
) -> Result<QueryResponse<SaveMediaListEntry>> {
    let variables = json!({
        "id": entry_id,
        "progress": progress,
    });

    if let serde_json::Value::Object(variables) = variables {
        info!(
            "Updating progress of title which has entry ID: `{}` with: progress `{}` for user...",
            entry_id, progress
        );
        query_graphql(UPDATE_MEDIA, &Some(variables), cfg, &client, true)
    } else {
        error!("Media list query variables is not a json object");
        Err(anyhow!("Media list query variables is not a json object"))
    }
}

pub fn create_new_entry(
    cfg: &mut MendoConfig,
    media_id: i32,
    status: MediaListStatus,
    progress: i32,
    client: &Client,
) -> Result<QueryResponse<SaveMediaListEntry>> {
    //the Result here has None fields, aka useless
    let variables = json!({
        "mediaId": media_id,
        "status": status,
        "progress": progress,
    });

    if let serde_json::Value::Object(variables) = variables {
        info!("Creating entry for title which has media ID: `{}` with: status `{:?}`, progress `{}` for user...",
            media_id, status, progress
        );
        query_graphql(UPDATE_MEDIA, &Some(variables), cfg, &client, true)
    } else {
        error!("Media list query variables is not a json object");
        Err(anyhow!("Media list query variables is not a json object"))
    }
}
