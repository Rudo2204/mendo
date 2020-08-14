use anyhow::Result;
use directories::ProjectDirs;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::anilist::model::User;
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
        if self.token == "Leave this field." {
            false
        } else {
            true
        }
    }

    pub fn ready_to_auth(&mut self) -> bool {
        if self.id == 0 || self.secret == "Edit this!" || self.name == "Edit this!" {
            false
        } else {
            true
        }
    }
}

fn capitalize_word(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
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
        capitalize_word(&application),
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
        let result = request::query_user(&mut cfg)?;
        if let Some(viewer_resp) = result.data {
            viewer_resp.viewer.dump_user_info(&user_profile_path)?;
        }
    }
    debug!("Loading user profile...");
    let s = fs::read_to_string(&user_profile_path)?;
    let user: User = serde_yaml::from_str(&s)?;
    Ok(user.id)
}
