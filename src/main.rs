use anyhow::{Context, Result};
use clap::{command, Command};

use std::path::PathBuf;

#[cfg(not(debug_assertions))]
use directories::ProjectDirs;
#[cfg(not(debug_assertions))]
use std::fs;

use wtime::app::App;

#[cfg(not(debug_assertions))]
fn get_db_file() -> Result<PathBuf> {
    let dirs =
        ProjectDirs::from("", "", env!("CARGO_PKG_NAME")).context("Error getting data dir")?;
    let mut data_dir_path = PathBuf::from(dirs.data_dir());

    fs::create_dir_all(&data_dir_path).context("Error creating data dir")?;
    data_dir_path.push("prod.sqlite");
    Ok(data_dir_path)
}

#[cfg(debug_assertions)]
fn get_db_file() -> Result<PathBuf> {
    Ok(PathBuf::from("test.sqlite"))
}

fn main() -> Result<()> {
    // Build argument parser
    let matches = command!()
        .subcommand(Command::new("checkin").about("Start counting working time"))
        .subcommand(Command::new("checkout").about("Stop counting work time and display count"))
        .get_matches();

    // Create the app object
    let db_file = get_db_file()?;
    println!("Database file is {:?}", db_file);
    let app = App::new(db_file.as_path()).context("Open DB file")?;

    // Reacts on command
    match matches.subcommand() {
        Some(("checkin", _)) => app.do_checkin(),
        Some(("checkout", _)) => app.do_checkout(),
        None => app.do_list(),
        _ => unreachable!("Should never match none"),
    }
}
