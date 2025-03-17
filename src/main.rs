use std::{path::PathBuf, process::exit};

use crate::config::Config;
use anyhow::{bail, Context};
use clap::{arg, command, value_parser};
use utils::search_common_paths;

mod config;
mod error;
mod utils;

#[rustfmt::skip::macros(arg)]
fn main() -> anyhow::Result<()> {
    let matches = command!()
        .arg(
            arg!(-c --config <VALUE>)
                .value_parser(value_parser!(PathBuf))
                .required(false)
                .help("the file used as base for the generated links"),
        )
        .arg(arg!(-g --generate).help("generates files based on the template"))
        .arg(arg!(-n --nocheck).help("skips the checks of the config file for validity"))
        .arg(
            arg!(-m --vanitymap <VALUE>)
                .default_value(None)
                .help("generate a vanitymap at <VALUE>"),
        )
        .arg(
            arg!(-r --redirectfile <VALUE>)
                .default_value(None)
                .help("generate a redirectfile at <VALUE>"),
        )
        .arg(
            arg!(-o --output <VALUE>)
                .default_value("output")
                .help("the base directory to populate"),
        )
        .get_matches();

    let config_file = if let Some(config_file) = matches.get_one::<PathBuf>("config") {
        if !config_file.exists() {
            bail!("Specified config doesn't exist.");
        }
        config_file.clone()
    } else if let Some(path) = search_common_paths("kurzlink.yml") {
        path
    } else {
        bail!("Config not specified and no config found in default locations.");
    };

    // unwrapping is okay since clap inserts safe defaults
    let nocheck_flag = matches.get_one::<bool>("nocheck").unwrap();
    let generate_flag = matches.get_one::<bool>("generate").unwrap();
    let output_path = matches.get_one::<String>("output").unwrap();
    let vanity_opt_path = matches.get_one::<String>("vanitymap");
    let redirect_opt_path = matches.get_one::<String>("redirectfile");

    // get the links
    let mut config = Config::new(config_file)
        .with_context(|| "config couldn't be initialized".to_string())
        .unwrap_or_else(|err| {
            println!("Error: {}", err.root_cause());
            exit(1);
        });

    if !*nocheck_flag {
        config.validate().unwrap_or_else(|err| {
            println!("Error: {}", err.root_cause());
            exit(2);
        });
    } else {
        config.network.check = false
    }

    if *generate_flag {
        config
            .render_files(output_path)
            .with_context(|| "Rendering files failed".to_string())?
    }

    if let Some(vanity_path) = vanity_opt_path {
        config
            .write_vanity(vanity_path)
            .with_context(|| "Writing the vanitymap failed".to_string())?;
    };

    if let Some(redirect_path) = redirect_opt_path {
        config
            .write_redirect_list(redirect_path)
            .with_context(|| "Writing the rewrite list failed".to_string())?;
    };
    anyhow::Ok(())
}
