use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use log::{debug, error, info};

#[derive(Serialize, Deserialize, Debug)]
pub struct AnilistToken<'a> {
    token_type: &'a str,
    expires_in: i32,
    pub access_token: &'a str,
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

pub fn create_proj_conf(qualifier: &str, organization: &str, application: &str) -> Result<()> {
    let proj_dirs = ProjectDirs::from(&qualifier, &organization, &application)
        .expect("Could not retrieve ProjectDirs, maybe you are using an unsupported OS");
    let conf_dir = proj_dirs.config_dir();

    match conf_dir.exists() && conf_dir.join(format!("{}.yml", &application)).exists() {
        true => {
            error!("This statement should never be reached!");
            info!(
                "{} configuration file exists. Will use these values...",
                capitalize_word(&application)
            );
        }
        false => {
            info!(
                "{} configuration file does not exist. I will now create a configuration file at {}",
                capitalize_word(&application),
                conf_dir.display()
            );
            confy::store(&application, MendoConfig::default())?;
        }
    }

    Ok(())
}

pub fn access_token_is_valid(cfg: &mut MendoConfig) -> Result<bool> {
    if cfg.token == "Leave this field." {
        Ok(false)
    } else {
        Ok(true)
    }
}

pub fn ready_to_auth(cfg: &mut MendoConfig) -> Result<bool> {
    if cfg.id == 0 || cfg.secret == "Edit this!" || cfg.name == "Edit this!" {
        Ok(false)
    } else {
        Ok(true)
    }
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
    info!("Configuration with access token is saved! Let's get to work!");

    Ok(cfg_with_token)
}
