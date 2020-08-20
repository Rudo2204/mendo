use anyhow::{anyhow, Result};
use clap::{crate_authors, crate_description, crate_version, App, AppSettings, Arg};
use fs2::FileExt;
use reqwest::blocking::Client;
use std::{fs::File, io};

use chrono::{Local, Utc};
use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, error, info, LevelFilter};

mod anilist;
mod util;
use anilist::{oauth, request};
use util::MendoConfig;

pub const PROGRAM_NAME: &str = "mendo";

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
            .level_for(PROGRAM_NAME, LevelFilter::Warn),
        1 => base_config
            .level(LevelFilter::Info)
            .level_for(PROGRAM_NAME, LevelFilter::Info),
        2 => base_config
            .level(LevelFilter::Info)
            .level_for(PROGRAM_NAME, LevelFilter::Debug),
        _3_or_more => base_config.level(LevelFilter::Trace),
    };

    // Separate file config so we can include year, month and day (UTC format) in file logs
    let log_file_path =
        util::get_data_dir("", "", PROGRAM_NAME)?.join(format!("{}.log", PROGRAM_NAME));
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
        .chain(fern::log_file(log_file_path)?);

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

fn main() -> Result<()> {
    let matches = App::new(PROGRAM_NAME)
        .setting(AppSettings::DisableHelpSubcommand)
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(
            App::new("auth")
                .arg(
                    Arg::with_name("force")
                        .short("f")
                        .long("force")
                        .help("force reauthorize flag"),
                )
                .about("Authorizes mendo to update progress"),
        )
        .subcommand(
            App::new("update")
                .about("Updates manga progress")
                .arg(
                    Arg::with_name("filename")
                        .help("the filename of manga archive")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("regexp")
                        .short("e")
                        .long("regexp")
                        .help("Overrides filename regex pattern")
                        .takes_value(true)
                        .default_value(r"^(.*) (v?|c?)\d+"),
                ),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Sets the level of debug information verbosity"),
        )
        .get_matches();

    let verbosity: u64 = matches.occurrences_of("verbose");
    let data_dir = util::get_data_dir("", "", PROGRAM_NAME)?;

    util::create_data_dir(&data_dir)?;
    setup_logging(verbosity)?;
    let log_file_path =
        util::get_data_dir("", "", PROGRAM_NAME)?.join(format!("{}.log", PROGRAM_NAME));
    let log_file = File::open(log_file_path)?;
    log_file.lock_exclusive()?;
    debug!("-----Logger is initialized. Starting main program!-----");

    let conf_file = util::get_conf_dir("", "", PROGRAM_NAME)?;
    while !conf_file.exists() {
        util::create_proj_conf("", "", PROGRAM_NAME)?;
    }

    let mut mendo_cfg: MendoConfig = confy::load(PROGRAM_NAME)?;
    if let Some(auth_matches) = matches.subcommand_matches("auth") {
        debug!("Config file loaded. Checking for auth status...");
        if !mendo_cfg.ready_to_auth() {
            println!(
            "You need to edit information in the config file first before we can authorize you."
            );
            println!("Mendo's config file is located at {}", conf_file.display());
            error!("One or more fields in conf file has not been edited. Exiting...");
            return Err(anyhow!(
                "One or more fields in conf file has not been edited!"
            ));
        } else if !mendo_cfg.access_token_is_valid() || auth_matches.is_present("force") {
            info!("Starting authorization process...");
            println!("Starting authorization process...");
            let res_token = oauth::auth(&mut mendo_cfg)?;
            util::cfg_save_token(PROGRAM_NAME, &mut mendo_cfg, &res_token)?;
            println!("Authorization process finished. Now you can use `update` subcommand!");
        }
    }

    if let Some(update_matches) = matches.subcommand_matches("update") {
        info!("Token from config file is valid. Let's get to work!");
        // Reuse a single client to take advantage of keep-alive connection pooling
        let client = Client::new();

        let filename_pattern = update_matches
            .value_of("regexp")
            .expect("Safe because of default value");
        let user_id = util::get_user_id(&mut mendo_cfg, &data_dir, &client)?;
        let filename = update_matches
            .value_of("filename")
            .expect("Safe because of clap handling");
        let media_id = util::get_media_id(
            &mut mendo_cfg,
            &data_dir,
            &filename,
            &filename_pattern,
            &client,
        )?;
        let (entry_id, progress) =
            util::get_eid_and_progress(&mut mendo_cfg, user_id, media_id, &client)?;

        request::update_media(&mut mendo_cfg, entry_id, progress + 1, &client)?;

        #[cfg(target_family = "unix")]
        util::notify_updated(&filename, &filename_pattern, progress + 1)?;
    }

    debug!("-----Everything is finished!-----");
    log_file.unlock()?;
    Ok(())
}
