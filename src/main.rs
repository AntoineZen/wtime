use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use anyhow::{Context, Result, anyhow};
use clap::{command, Command};
use now::DateTimeNow;
use wtime::db::{InOut, Stamp};

fn main() -> Result<()> {
    let matches = command!()
        .subcommand(Command::new("checkin").about("Start counting working time"))
        .subcommand(Command::new("checkout").about("Stop counting work time and display count"))
        .get_matches();

    match matches.subcommand() {
        Some(("checkin", _)) => do_checkin(),
        Some(("checkout", _)) => do_checkout(),
        None => do_list(),
        _ => unreachable!("Should never match none"),
    }
}

fn open_db() -> Result<sqlite::Connection> {
    let db_file = std::path::Path::new("prod.sqlite");
    let must_init = !db_file.exists();
    let conn = sqlite::open(db_file)?;

    if must_init {
        Stamp::create(&conn).context("Crate Stamp table")?;
    }
    Ok(conn)
}

fn do_checkin() -> Result<()> {
    let c = open_db().context("Opening database")?;

    // check that we are actually out
    if let Some(last_stamp) = Stamp::last(&c) {
        if last_stamp.in_out == InOut::In {
            return Err(anyhow!("Already checked in ! (Do you meant to check-out ?)"));
        }
    }

    // Creat teh checking stamp
    let mut stamp = Stamp::check_in();
    stamp.insert(&c).context("Inserting new stamp")?;

    println!("Checked in at {}", stamp.date.format("%H:%M"));
    Ok(())
}

fn do_checkout() -> Result<()> {
    let c = open_db().context("Opening database")?;

    // Check that last stamp is check-in
    if let Some(last_stamp) = Stamp::last(&c) {
        if last_stamp.in_out == InOut::Out {
            return Err(anyhow!("Already checked out ! (Do you meant to check-in ?)"));
        }
    }

    // Create the checkout stamps
    let mut stamp = Stamp::check_out();
    stamp.insert(&c).context("Inserting new stamp")?;

    println!("Checked out at {}", stamp.date.format("%H:%M"));

    // Print worked time
    let now = Utc::now();
    let begin_of_week = now.beginning_of_week();
    let begin_of_day = now.beginning_of_day();

    if let Some(checkin) = stamp.previous(&c) {
        let work_time = checkin.delta(&stamp);
        println!(
            "You worked {} hours, {} minutes and {} seconds.",
            work_time.num_hours(),
            work_time.num_minutes(),
            work_time.num_seconds()
        );
    }

    Ok(())
}

fn do_list() -> Result<()> {
    Ok(())
}
