use anyhow::Result;
use reqwest::{self, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs;

use log::{debug, error, info, warn};
use pretty_env_logger;

mod anilist;
mod util;
use anilist::oauth;
use util::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryResponse<T> {
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
    pub user_preferred: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Media {
    pub id: Option<i32>,
    pub title: Option<MediaTitle>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct QueryID {
    pub media: Media,
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let conf_file = util::get_conf_dir("", "", "mendo")?;
    while !conf_file.exists() {
        util::create_proj_conf("", "", "mendo")?;
    }

    let mut mendo_cfg: MendoConfig = confy::load("mendo")?;

    if !util::ready_to_auth(&mut mendo_cfg)? {
        error!("One or more fields in conf file has not been edited");
        println!(
            "You need to edit information in the config file first before we can authorize you."
        );
        println!("Mendo's config file is located at {}", conf_file.display());
    } else if !util::access_token_is_valid(&mut mendo_cfg)? {
        warn!("Access token is invalid. Starting authorization process...");
        println!("Starting authorization process...");
        let res_token = oauth::auth(&mut mendo_cfg).await?;
        mendo_cfg = util::cfg_save_token("mendo", &mut mendo_cfg, &res_token)?;
    } else {
        info!("Token is valid, let's get to work!");
    }
    let token = mendo_cfg.token;

    let content = fs::read_to_string("graphql/update_media.gql").await?;
    let url = "https://graphql.anilist.co";
    let var = json!({ "id": 84994881, "progress": 109});

    let client = reqwest::Client::new();
    let query = json!({ "query": content, "variables": var });
    //let query = json!({ "query": content });
    let res = client
        .post(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&query)
        .send()
        .await?;

    let res_status = res.status();
    debug!("Anilist returned `{}'", res.status());

    match res_status {
        StatusCode::OK => {
            println!("{}", res.text().await?);
            //let test = res.json::<QueryResponse<QueryID>>().await?;
            //println!("{:#?}", test);
        }
        StatusCode::TOO_MANY_REQUESTS => {
            eprintln!("Too many requests!");
        }
        StatusCode::NOT_FOUND => {
            println!("{}", res.text().await?);
        }
        _ => panic!("shouldn't panic but hey!"),
    }

    Ok(())
}
