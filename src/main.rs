use anyhow::{anyhow, Result};
use clap::{App, Arg};
use std::io;

use chrono::{Local, Utc};
use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, error, info, warn, LevelFilter};

mod anilist;
mod util;
use anilist::{oauth, request};
use util::MendoConfig;

const PROGRAM_NAME: &str = "mendo";

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
    if !mendo_cfg.ready_to_auth() {
        println!(
            "You need to edit information in the config file first before we can authorize you."
        );
        println!("Mendo's config file is located at {}", conf_file.display());
        error!("One or more fields in conf file has not been edited. Exiting...");
        return Err(anyhow!(
            "One or more fields in conf file has not been edited!"
        ));
    } else if !mendo_cfg.access_token_is_valid() {
        warn!("Access token is invalid. Starting authorization process...");
        println!("Starting authorization process...");
        let res_token = oauth::auth(&mut mendo_cfg).await?;
        mendo_cfg = util::cfg_save_token(PROGRAM_NAME, &mut mendo_cfg, &res_token)?;
        info!("Token is saved to config file!");
    }
    info!("Token from config file is valid. Let's get to work!");

    request::query_user(&mut mendo_cfg).await?;

    debug!("-----Everything is finished!-----");
    Ok(())
}
