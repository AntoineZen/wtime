use anyhow::{Context, Result};
use clap::{command, Arg, Command};
use wtime::app::App;

fn main() -> Result<()> {
    // Build argument parser
    let matches = command!()
        .arg(
            Arg::new("database")
                .long("db")
                .default_value("prod.sqlite")
                .help("Specify database to use"),
        )
        .subcommand(Command::new("checkin").about("Start counting working time"))
        .subcommand(Command::new("checkout").about("Stop counting work time and display count"))
        .get_matches();

    // Get database file from argument
    let db_file_name: String = matches
        .get_one::<String>("database")
        .map(|s| s.to_string())
        .context("Get DB file")?;

    // Create the app object
    let app = App::new(db_file_name).context("Open DB file")?;

    // Reacts on command
    match matches.subcommand() {
        Some(("checkin", _)) => app.do_checkin(),
        Some(("checkout", _)) => app.do_checkout(),
        None => app.do_list(),
        _ => unreachable!("Should never match none"),
    }
}
