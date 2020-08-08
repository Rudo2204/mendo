use anyhow::{anyhow, Result};
use clap::{App, Arg};
use reqwest::{self, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io;
use tokio::fs;

use chrono::{Local, Utc};
use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, error, info, warn, LevelFilter};

mod anilist;
mod util;
use anilist::oauth;
use util::*;

const PROGRAM_NAME: &str = "mendo";
const ANILIST_API_URL: &str = "https://graphql.anilist.co";

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

fn setup_logging(verbosity: u64) -> Result<()> {
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Blue)
        .trace(Color::BrightBlack); // this is the same as the background color

    let mut base_config = fern::Dispatch::new();

    base_config = match verbosity {
        0 => base_config
            .level(LevelFilter::Info)
            .level_for("mendo", LevelFilter::Warn),
        1 => base_config
            .level(LevelFilter::Info)
            .level_for("mendo", LevelFilter::Info),
        2 => base_config
            .level(LevelFilter::Info)
            .level_for("mendo", LevelFilter::Debug),
        _3_or_more => base_config.level(LevelFilter::Debug),
    };

    // Separate file config so we can include year, month and day (UTC format) in file logs
    let file_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{date} {colored_level} {colored_target} > {colored_message}",
                date = Utc::now().format("%Y-%m-%dT%H:%M:%SUTC"),
                colored_level = format_args!(
                    "\x1B[{}m{}\x1B[0m",
                    colors_line.get_color(&record.level()).to_fg_str(),
                    record.level()
                ),
                colored_target = format_args!("\x1B[95m{}\x1B[0m", record.target()),
                colored_message = format_args!(
                    "\x1B[{}m{}\x1B[0m",
                    colors_line.get_color(&record.level()).to_fg_str(),
                    message
                ),
            ))
        })
        .chain(fern::log_file("program.log")?);

    // For stdout output we will just output local %H:%M:%S
    let stdout_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{date} {colored_level} {colored_target} > {colored_message}",
                date = Local::now().format("%H:%M:%S"),
                colored_level = format_args!(
                    "\x1B[{}m{}\x1B[0m",
                    colors_line.get_color(&record.level()).to_fg_str(),
                    record.level()
                ),
                colored_target = format_args!("\x1B[95m{}\x1B[0m", record.target()),
                colored_message = format_args!(
                    "\x1B[{}m{}\x1B[0m",
                    colors_line.get_color(&record.level()).to_fg_str(),
                    message
                ),
            ))
        })
        .chain(io::stdout());

    base_config
        .chain(file_config)
        .chain(stdout_config)
        .apply()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cmd_arguments = App::new(PROGRAM_NAME)
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Increases debug verbosity"),
        )
        .get_matches();

    let verbosity: u64 = cmd_arguments.occurrences_of("verbose");
    setup_logging(verbosity)?;
    debug!("-----Logger is initialized. Starting main program!-----");

    let conf_file = util::get_conf_dir("", "", PROGRAM_NAME)?;
    while !conf_file.exists() {
        util::create_proj_conf("", "", PROGRAM_NAME)?;
    }

    let mut mendo_cfg: MendoConfig = confy::load(PROGRAM_NAME)?;
    debug!("Config file loaded. Checking for auth status...");

    // TODO: Better auth checking
    if !util::ready_to_auth(&mut mendo_cfg)? {
        println!(
            "You need to edit information in the config file first before we can authorize you."
        );
        println!("Mendo's config file is located at {}", conf_file.display());
        error!("One or more fields in conf file has not been edited. Exiting...");
        return Err(anyhow!(
            "One or more fields in conf file has not been edited!"
        ));
    } else if !util::access_token_is_valid(&mut mendo_cfg)? {
        warn!("Access token is invalid. Starting authorization process...");
        println!("Starting authorization process...");
        let res_token = oauth::auth(&mut mendo_cfg).await?;
        mendo_cfg = util::cfg_save_token(PROGRAM_NAME, &mut mendo_cfg, &res_token)?;
        info!("Token is saved to config file!");
    }
    let token = mendo_cfg.token;
    info!("Token from config file loaded. Let's get to work!");

    let content = fs::read_to_string("graphql/query_user.gql").await?;

    //let var = json!({ "id": 84994881, "progress": 109});
    //let query = json!({ "query": content, "variables": var });
    let query = json!({ "query": content });

    let client = reqwest::Client::new();
    debug!("Sending POST request...");
    let res = client
        .post(ANILIST_API_URL)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&query)
        .send()
        .await?;

    let res_status = res.status();
    debug!("Anilist returned code `{}'", res.status());

    match res_status {
        StatusCode::OK => {
            println!("{}", res.text().await?);
            //let test = res.json::<QueryResponse<QueryID>>().await?;
            //println!("{:#?}", test);
            info!("Anilist handled the sent request!");
        }
        StatusCode::TOO_MANY_REQUESTS => {
            error!("Too many requests!");
            return Err(anyhow!("Too many requests!"));
        }
        StatusCode::NOT_FOUND => {
            error!("Anilist returned code `404 Not Found'!");
            return Err(anyhow!("Anilist returned code `404 Not Found'!"));
        }
        _ => {
            error!("Anilist returned an unimplemented StatusCode!");
            return Err(anyhow!("Anilist returned an unimplemented StatusCode!"));
        }
    }

    debug!("-----Everything is finished!-----");
    Ok(())
}
