use anyhow::{anyhow, Result};
use log::{debug, error, info};
use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Map, Value};
use tokio::time;

use super::model::{MediaListCollection, User};
use super::query::QUERY_USER;
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
    pub viewer: Option<User>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MediaListCollectionResponse {
    pub media_list_collection: Option<Box<MediaListCollection>>,
}

pub async fn query_graphql<R>(
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
        debug!("Sending POST request...");
        let res = client
            .post(ANILIST_API_URL)
            .header("ContentType", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .json(&query)
            .send()
            .await?;

        let res_status = res.status();
        debug!("Anilist returned code `{}'", res.status());

        match res_status {
            StatusCode::TOO_MANY_REQUESTS => {
                let secs;
                let retry = res.headers().get("Retry-After");
                if let Some(val) = retry {
                    let header = String::from_utf8_lossy(val.as_bytes());
                    secs = header.parse::<u64>().unwrap_or(60);
                } else {
                    secs = 60;
                }
                time::delay_for(time::Duration::from_secs(secs)).await;
            }
            StatusCode::UNAUTHORIZED => {
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
                error!("Unthorized! Run the program again to reauthorize!");
                return Err(anyhow!("Unthorized! Run the program again to reauthorize!"));
            }
            StatusCode::OK | _ => {
                let response: QueryResponse<R> = res.json().await?;
                debug!("Response=\n{:#?}", response);
                info!("Request is handled by Anilist!");
                return Ok(response);
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

pub async fn query_user(cfg: &mut MendoConfig) -> Result<QueryResponse<ViewerResponse>> {
    debug!("Querying currently authenticated user...");
    query_graphql(QUERY_USER, &None, cfg).await
}
