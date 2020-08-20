use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use log::{debug, error, info};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::anilist::model::{MediaType, User};
use crate::anilist::request;

#[derive(Serialize, Deserialize, Debug)]
struct AnilistToken<'a> {
    token_type: &'a str,
    expires_in: i32,
    access_token: &'a str,
    refresh_token: &'a str,
}

// have to use String here because of how Confy serdes the structs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MendoConfig {
    pub id: i32,
    pub secret: String,
    pub name: String,
    pub url: String,
    pub token: String,
}

impl Default for MendoConfig {
    fn default() -> Self {
        MendoConfig {
            id: 0,
            secret: "Edit this!".to_string(),
            name: "Edit this!".to_string(),
            url: "http://localhost:8080/callback".to_string(),
            token: "Leave this field.".to_string(),
        }
    }
}

impl MendoConfig {
    pub fn access_token_is_valid(&mut self) -> bool {
        self.token != "Leave this field."
    }

    pub fn ready_to_auth(&mut self) -> bool {
        !(self.id == 0 || self.secret == "Edit this!" || self.name == "Edit this!")
    }
}

pub fn get_conf_dir(qualifier: &str, organization: &str, application: &str) -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from(&qualifier, &organization, &application)
        .expect("Could not retrieve ProjectDirs, maybe you are using an unsupported OS");
    Ok(proj_dirs.config_dir().to_path_buf())
}

pub fn get_data_dir(qualifier: &str, organization: &str, application: &str) -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from(&qualifier, &organization, &application)
        .expect("Could not retrieve ProjectDirs, maybe you are using an unsupported OS");
    Ok(proj_dirs.data_dir().to_path_buf())
}

pub fn create_data_dir(data_dir: &PathBuf) -> Result<()> {
    if !data_dir.exists() {
        debug!("Project data dir does not exist, creating them...");
        std::fs::create_dir_all(data_dir)?;
        debug!("Successfully created data dirs");
    }
    Ok(())
}

pub fn create_proj_conf(qualifier: &str, organization: &str, application: &str) -> Result<()> {
    let proj_dirs = ProjectDirs::from(&qualifier, &organization, &application)
        .expect("Could not retrieve ProjectDirs, maybe you are using an unsupported OS");
    let conf_dir = proj_dirs.config_dir();

    debug!(
        "{} configuration file does not exist. I will now create a configuration file at {}",
        &application,
        conf_dir.display()
    );
    confy::store(&application, MendoConfig::default())?;
    debug!("Default config file saved.");

    Ok(())
}

pub fn cfg_save_token(
    application: &str,
    cfg: &mut MendoConfig,
    res_token: &str,
) -> Result<MendoConfig> {
    let anilist_token: AnilistToken = serde_json::from_str(&res_token)?;
    debug!("Deserialized anilist token:\n{:#?}\n", anilist_token);

    let client_secret = &cfg.secret;
    let client_name = &cfg.name;

    let cfg_with_token = MendoConfig {
        id: cfg.id,
        secret: client_secret.to_string(),
        name: client_name.to_string(),
        url: "http://localhost:8080/callback".to_string(),
        token: anilist_token.access_token.to_string(),
    };
    confy::store(&application, cfg_with_token.clone())?;

    info!("Configuration with access token is saved!");
    Ok(cfg_with_token)
}

pub fn get_user_id(mut cfg: &mut MendoConfig, data_dir: &PathBuf) -> Result<i32> {
    let user_profile_path = data_dir.join("user.yml");
    if !user_profile_path.exists() {
        debug!("Local user profile does not exist. Querying to create one...");
        let query_result = request::query_user(&mut cfg)?;
        if let Some(viewer_resp) = query_result.data {
            viewer_resp.viewer.dump_user_info(&user_profile_path)?;
        }
    }
    debug!("Loading user profile...");
    let s = fs::read_to_string(&user_profile_path)?;
    let user: User = serde_yaml::from_str(&s)?;
    let user_id = user.id;
    debug!("Got user_id `{}` of authenticated user!", user_id);
    Ok(user_id)
}

pub fn get_media_id(mut cfg: &mut MendoConfig, data_dir: &PathBuf, filename: &str) -> Result<i32> {
    let local_media_data = data_dir.join("media_data.txt");
    let name_re = Regex::new(r"^(.*) v?(\d+)")?;
    let caps = name_re
        .captures(filename)
        .expect("Safe because of append_local_data");
    let name = caps.get(1).map_or_else(|| "", |m| m.as_str());
    debug!("Got manga name: `{}` using regex", &name);

    if !local_media_data.exists() {
        debug!(
            "Local media data does not exist, creating one at {}",
            &local_media_data.display()
        );
        File::create(&local_media_data)?;
    }

    let local_data = fs::read_to_string(&local_media_data)?;
    debug!(
        "Attempting to find media_id of manga `{}` from local media data...",
        &name
    );
    let file_re = Regex::new(format!("{} - mediaId: (\\d+)", &name).as_str())?;
    match file_re.captures(&local_data) {
        Some(caps) => {
            let media_id: i32 = caps
                .get(1)
                .expect("Safe because of append_local_data")
                .as_str()
                .parse()
                .expect("Could not parse media_id from str to i32");
            debug!(
                "Found media_id: `{}` of manga `{}` from local media data!",
                media_id, &name
            );
            Ok(media_id)
        }
        None => {
            debug!("Did not find media_id from local media data. Will now query for it.");
            let query_result = request::search_media(&mut cfg, &name, MediaType::Manga)?;
            match query_result.data {
                Some(media_resp) => {
                    let media_id = media_resp.media.media_id;
                    debug!(
                        "Found media_id: `{}` of manga `{}` from querying the API!",
                        media_id, &name
                    );
                    append_local_data(&local_media_data, name, media_id)?;
                    Ok(media_id)
                }
                None => xkcd_unreachable::xkcd_unreachable!(),
            }
        }
    }
}

fn append_local_data(path: &PathBuf, name: &str, media_id: i32) -> Result<()> {
    let mut file = OpenOptions::new().append(true).open(&path)?;
    writeln!(file, "{} - mediaId: {}", name, media_id)?;
    debug!(
        "Appended received data to local media data at {}",
        &path.display()
    );
    Ok(())
}

pub fn get_eid_and_progress(
    mut cfg: &mut MendoConfig,
    user_id: i32,
    media_id: i32,
) -> Result<(i32, i32)> {
    let query_result = request::query_media_list(&mut cfg, user_id, media_id, MediaType::Manga)?;

    match query_result.data {
        Some(media_list_resp) => Ok((
            media_list_resp.media_list.entry_id,
            media_list_resp.media_list.progress,
        )),
        None => xkcd_unreachable::xkcd_unreachable!(),
    }
}
